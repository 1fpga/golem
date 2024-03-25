use crate::utils::JsVec;
use boa_engine::object::builtins::JsArray;
use boa_engine::value::TryFromJs;
use boa_engine::{
    js_string, Context, JsArgs, JsError, JsObject, JsResult, JsString, JsValue, Module,
};
use diesel::connection::LoadConnection;
use diesel::deserialize::FromSql;
use diesel::query_builder::{BoxedSqlQuery, SqlQuery};
use diesel::row::{Field, Row};
use diesel::sqlite::SqliteType;
use diesel::SqliteConnection;
use std::cell::RefCell;
use std::rc::Rc;

fn build_query_<'a>(
    args: &[JsValue],
    ctx: &mut Context,
) -> JsResult<BoxedSqlQuery<'a, diesel::sqlite::Sqlite, SqlQuery>> {
    let query = args
        .get_or_undefined(0)
        .to_string(ctx)?
        .to_std_string_escaped();
    let bindings = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| JsArray::new(ctx).into());
    let bindings = JsVec::<JsValue>::try_from_js(&bindings, ctx)?;

    let mut q = diesel::sql_query(&query).into_boxed();

    for b in bindings.iter() {
        q = match b {
            JsValue::String(s) => q.bind::<diesel::sql_types::Text, _>(s.to_std_string_escaped()),
            JsValue::Integer(i) => q.bind::<diesel::sql_types::Integer, _>(*i),
            JsValue::Boolean(b) => q.bind::<diesel::sql_types::Bool, _>(*b),
            JsValue::Rational(f) => q.bind::<diesel::sql_types::Double, _>(*f),
            JsValue::Null => {
                q.bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(None::<String>)
            }
            JsValue::Undefined => {
                q.bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(None::<String>)
            }
            JsValue::BigInt(_) => {
                return Err(JsError::from_opaque(
                    JsString::from("BigInt bindings are not supported").into(),
                ))
            }
            JsValue::Object(_) => {
                return Err(JsError::from_opaque(
                    JsString::from("Object bindings are not supported").into(),
                ))
            }
            JsValue::Symbol(_) => {
                return Err(JsError::from_opaque(
                    JsString::from("Symbol bindings are not supported").into(),
                ))
            }
        }
    }

    Ok(q)
}

fn query(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
    app: &mut golem_ui::application::GoLEmApp,
) -> JsResult<JsValue> {
    let query = build_query_(args, ctx)?;

    let db = app.database();
    let mut db = db.lock().unwrap();
    let rows: <SqliteConnection as LoadConnection<_>>::Cursor<'_, '_> = db
        .load(query)
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;

    let result = JsArray::new(ctx);
    for row in rows {
        let row = row.map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;
        let row_result = JsObject::with_null_proto();

        for i in 0..row.field_count() {
            let Some(field) = row.get(i) else {
                continue;
            };
            let name = field.field_name();

            let Some(value) = field.value() else {
                row_result.set(i, JsValue::undefined(), false, ctx)?;
                continue;
            };

            let value = match value.value_type() {
                Some(SqliteType::Binary) => {
                    let i = bool::from_sql(value)
                        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;
                    i.into()
                }
                Some(SqliteType::SmallInt) => {
                    let i = i16::from_sql(value)
                        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;
                    i.into()
                }
                Some(SqliteType::Integer) => {
                    let i = i32::from_sql(value)
                        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;
                    i.into()
                }
                Some(SqliteType::Long) => {
                    let i = i64::from_sql(value)
                        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;
                    i.into()
                }
                Some(SqliteType::Float) => {
                    let f = f32::from_sql(value)
                        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;
                    f.into()
                }
                Some(SqliteType::Double) => {
                    let f = f64::from_sql(value)
                        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;
                    f.into()
                }
                Some(SqliteType::Text) => {
                    let s: String = FromSql::<diesel::sql_types::Text, _>::from_sql(value)
                        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;
                    JsString::from(s).into()
                }
                _ => JsValue::undefined(),
            };

            if let Some(name) = name {
                row_result.set(JsString::from(name), value, false, ctx)?;
            }
        }

        result.push(row_result, ctx)?;
    }

    Ok(result.into())
}

pub fn create_module(
    context: &mut Context,
    app: Rc<RefCell<golem_ui::application::GoLEmApp>>,
) -> JsResult<(JsString, Module)> {
    let menu = boa_engine::object::FunctionObjectBuilder::new(
        context.realm(),
        boa_engine::NativeFunction::from_copy_closure({
            let app = Rc::downgrade(&app).as_ptr();
            move |_this, args, ctx| query(_this, args, ctx, unsafe { &mut (*app).borrow_mut() })
        }),
    )
    .name(js_string!("query"))
    .build();

    Ok((
        js_string!("db"),
        Module::synthetic(
            // Make sure to list all exports beforehand.
            &[js_string!("query")],
            // The initializer is evaluated every time a module imports this synthetic module,
            // so we avoid creating duplicate objects by capturing and cloning them instead.
            boa_engine::module::SyntheticModuleInitializer::from_copy_closure_with_captures(
                |module, fns, _| {
                    module.set_export(&js_string!("query"), fns.0.clone().into())?;

                    Ok(())
                },
                (menu,),
            ),
            None,
            context,
        ),
    ))
}

use crate::utils::JsVec;
use boa_engine::object::builtins::JsArray;
use boa_engine::{js_string, Context, JsArgs, JsError, JsResult, JsString, JsValue, Module};
use diesel::connection::LoadConnection;
use diesel::deserialize::FromSql;
use diesel::row::{Field, Row};
use diesel::sqlite::SqliteType;
use diesel::{QueryDsl, SqliteConnection};
use std::cell::RefCell;
use std::rc::Rc;

fn query(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
    app: &mut golem_ui::application::GoLEmApp,
) -> JsResult<JsValue> {
    let query = args
        .get_or_undefined(0)
        .to_string(ctx)?
        .to_std_string_escaped();
    let bindings = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| JsArray::new(ctx).into());
    let vec: JsVec<JsValue> = bindings.try_js_into(ctx)?;
    let bindings = &vec.0;

    let mut q = diesel::sql_query(&query).into_boxed();
    // for b in bindings {
    //     q = match b {
    //         JsValue::String(s) => q
    //             .bind::<diesel::sql_types::Text, _>(s.to_std_string_escaped())
    //             .into_boxed(),
    //         JsValue::Integer(i) => q.bind::<diesel::sql_types::Integer, _>(i).into_boxed(),
    //         JsValue::Boolean(b) => q.bind::<diesel::sql_types::Bool, _>(b).into_boxed(),
    //         JsValue::Rational(f) => q.bind::<diesel::sql_types::Double, _>(f).into_boxed(),
    //         JsValue::Null => q
    //             .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(None)
    //             .into_boxed(),
    //         _ => return Err(JsError::new_type_error("Unsupported type for SQL binding")),
    //     }
    // }

    let db = app.database();
    let mut db = db.lock().unwrap();
    let rows: <SqliteConnection as LoadConnection<_>>::Cursor<'_, '_> = db
        .load(q)
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;

    let mut result = JsArray::new(ctx);
    for row in rows {
        let row = row.map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;
        let mut row_result = JsArray::new(ctx);

        for i in 0..row.field_count() {
            let Some(field) = row.get(i) else {
                row_result.push(JsValue::undefined(), ctx)?;
                continue;
            };

            let Some(value) = field.value() else {
                row_result.push(JsValue::undefined(), ctx)?;
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

            row_result.push(value, ctx)?;
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

use std::cell::RefCell;
use std::rc::Rc;

use boa_engine::{
    Context, js_string, JsError, JsObject, JsResult, JsString, JsValue, Module,
};
use boa_engine::object::builtins::JsArray;
use boa_engine::value::TryFromJs;
use boa_interop::{IntoJsFunctionUnsafe, IntoJsModule};
use diesel::{Connection, SqliteConnection};
use diesel::connection::LoadConnection;
use diesel::deserialize::FromSql;
use diesel::query_builder::{BoxedSqlQuery, SqlQuery};
use diesel::row::{Field, Row};
use diesel::sqlite::SqliteType;

fn build_query_<'a>(
    query: String,
    bindings: Option<JsArray>,
    ctx: &mut Context,
) -> JsResult<BoxedSqlQuery<'a, diesel::sqlite::Sqlite, SqlQuery>> {
    let bindings = bindings.unwrap_or_else(|| JsArray::new(ctx));
    let bindings = Vec::<JsValue>::try_from_js(&bindings.into(), ctx);
    let bindings = bindings?;

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
                ));
            }
            JsValue::Object(_) => {
                return Err(JsError::from_opaque(
                    JsString::from("Object bindings are not supported").into(),
                ));
            }
            JsValue::Symbol(_) => {
                return Err(JsError::from_opaque(
                    JsString::from("Symbol bindings are not supported").into(),
                ));
            }
        }
    }

    Ok(q)
}

fn create_row_object<'a>(
    row: impl Row<'a, diesel::sqlite::Sqlite>,
    ctx: &mut Context,
) -> JsResult<JsObject> {
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

    Ok(row_result)
}

fn query_(
    query: String,
    bindings: Option<JsArray>,
    ctx: &mut Context,
    app: &mut golem_ui::application::GoLEmApp,
) -> JsResult<JsValue> {
    let query = build_query_(query, bindings, ctx)?;

    let db = app.database();
    let mut db = db.lock().unwrap();
    let rows: <SqliteConnection as LoadConnection<_>>::Cursor<'_, '_> = db
        .load(query)
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;

    let result = JsArray::new(ctx);
    for row in rows {
        let row = row.map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;

        let row_result = create_row_object(row, ctx)?;

        result.push(row_result, ctx)?;
    }

    Ok(result.into())
}

fn execute(
    query: String,
    bindings: Option<JsArray>,
    ctx: &mut Context,
    app: &mut golem_ui::application::GoLEmApp,
) -> JsResult<JsValue> {
    let query = build_query_(query, bindings, ctx)?;

    let db = app.database();
    let mut db = db.lock().unwrap();
    let result = db
        .execute_returning_count(&query)
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;

    Ok(JsValue::from(result))
}

fn get(
    query: String,
    bindings: Option<JsArray>,
    ctx: &mut Context,
    app: &mut golem_ui::application::GoLEmApp,
) -> JsResult<JsValue> {
    let query = build_query_(query, bindings, ctx)?;

    let db = app.database();
    let mut db = db.lock().unwrap();
    let mut row: <SqliteConnection as LoadConnection<_>>::Cursor<'_, '_> =
        db.load(query).map_err(|err| {
            JsError::from_opaque(
                JsString::from(format!("Database error: {}", err.to_string())).into(),
            )
        })?;

    let result = match row.next() {
        Some(Ok(row)) => Ok(JsValue::from(create_row_object(row, ctx)?)),
        Some(Err(err)) => Err(JsError::from_opaque(
            JsString::from(format!("Database error: {}", err.to_string())).into(),
        )),
        None => Ok(JsValue::undefined()),
    };

    result
}

pub fn create_module(
    context: &mut Context,
    app: Rc<RefCell<golem_ui::application::GoLEmApp>>,
) -> JsResult<(JsString, Module)> {
    unsafe {
        let execute = {
            let app = app.clone();
            move |query: JsString, bindings| {
                execute(query.to_std_string_escaped(), bindings, context, &mut app.borrow_mut()).unwrap()
            }
        }.into_js_function_unsafe(context);

        let get = {
            let app = app.clone();
            move |query: JsString, bindings| {
                get(query.to_std_string_escaped(), bindings, context, &mut app.borrow_mut()).unwrap()
            }
        }.into_js_function_unsafe(context);

        let query = {
            let app = app.clone();
            move |query: JsString, bindings| {
                query_(query.to_std_string_escaped(), bindings, context, &mut app.borrow_mut()).unwrap()
            }
        }.into_js_function_unsafe(context);

        let module = [
            (js_string!("execute"), execute),
            (js_string!("get"), get),
            (js_string!("query"), query),
        ].into_js_module(context);

        Ok((js_string!("db"), module))
    }
}

use boa_engine::object::builtins::JsArray;
use boa_engine::value::TryFromJs;
use boa_engine::{js_string, Context, JsError, JsObject, JsResult, JsString, JsValue, Module};
use boa_interop::{ContextData, IntoJsFunctionCopied, IntoJsModule};
use diesel::connection::LoadConnection;
use diesel::deserialize::FromSql;
use diesel::query_builder::{BoxedSqlQuery, SqlQuery};
use diesel::row::{Field, Row};
use diesel::sqlite::{Sqlite, SqliteType, SqliteValue};
use diesel::{Connection, SqliteConnection};

use crate::HostData;

/// A value in a column in SQL, compatible with JavaScript.
pub enum SqlValue {
    String(String),
    Number(f64),
    Integer(i32),
    Boolean(bool),
    Null,
}

impl SqlValue {
    pub fn bind(self, query: BoxedSqlQuery<Sqlite, SqlQuery>) -> BoxedSqlQuery<Sqlite, SqlQuery> {
        match self {
            SqlValue::String(s) => query.bind::<diesel::sql_types::Text, _>(s.to_string()),
            SqlValue::Integer(i) => query.bind::<diesel::sql_types::Integer, _>(i),
            SqlValue::Boolean(b) => query.bind::<diesel::sql_types::Bool, _>(b),
            SqlValue::Number(f) => query.bind::<diesel::sql_types::Double, _>(f),
            SqlValue::Null => query
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(None::<String>),
        }
    }
}

impl TryFromJs for SqlValue {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::Null => Ok(Self::Null),
            JsValue::Boolean(b) => Ok(Self::Boolean(*b)),
            JsValue::String(s) => Ok(Self::String(s.to_std_string_escaped())),
            JsValue::Rational(r) => Ok(Self::Number(*r)),
            JsValue::Integer(i) => Ok(Self::Integer(*i)),
            JsValue::BigInt(_) | JsValue::Undefined | JsValue::Object(_) | JsValue::Symbol(_) => {
                Err(JsError::from_opaque(
                    js_string!(format!(
                        "Invalid value type {}, cannot convert to SQL.",
                        value.type_of()
                    ))
                    .into(),
                ))
            }
        }
    }
}

impl TryFrom<SqliteValue<'_, '_, '_>> for SqlValue {
    type Error = JsError;

    fn try_from(value: SqliteValue) -> Result<Self, Self::Error> {
        match value.value_type() {
            Some(SqliteType::Binary) => {
                let i = bool::from_sql(value)
                    .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;
                Ok(SqlValue::Boolean(i))
            }
            Some(SqliteType::SmallInt) => {
                let i = i16::from_sql(value)
                    .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;
                Ok(SqlValue::Integer(i.into()))
            }
            Some(SqliteType::Integer) => {
                let i = i32::from_sql(value)
                    .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;
                Ok(SqlValue::Integer(i))
            }
            Some(SqliteType::Long) => {
                let i = i64::from_sql(value)
                    .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;
                Ok(SqlValue::Integer(i as i32))
            }
            Some(SqliteType::Float) => {
                let f = f32::from_sql(value)
                    .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;
                Ok(SqlValue::Number(f as f64))
            }
            Some(SqliteType::Double) => {
                let f = f64::from_sql(value)
                    .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;
                Ok(SqlValue::Number(f))
            }
            Some(SqliteType::Text) => {
                let s: String = FromSql::<diesel::sql_types::Text, _>::from_sql(value)
                    .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;
                Ok(SqlValue::String(s))
            }
            _ => Err(JsError::from_opaque(
                js_string!("Unsupported SQL type").into(),
            )),
        }
    }
}

impl Into<JsValue> for SqlValue {
    fn into(self) -> JsValue {
        match self {
            SqlValue::String(s) => JsString::from(s).into(),
            SqlValue::Number(f) => f.into(),
            SqlValue::Integer(i) => i.into(),
            SqlValue::Boolean(b) => b.into(),
            SqlValue::Null => JsValue::null(),
        }
    }
}

fn build_query_<'a>(
    query: String,
    bindings: Option<Vec<SqlValue>>,
    _ctx: &mut Context,
) -> JsResult<BoxedSqlQuery<'a, Sqlite, SqlQuery>> {
    let bindings = bindings.unwrap_or_else(|| vec![]);
    let mut q = diesel::sql_query(&query).into_boxed();

    for b in bindings.into_iter() {
        q = b.bind(q);
    }

    Ok(q)
}

fn create_row_object<'a>(row: impl Row<'a, Sqlite>, ctx: &mut Context) -> JsResult<JsObject> {
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

        if let Some(name) = name {
            row_result.set(JsString::from(name), SqlValue::try_from(value)?, false, ctx)?;
        }
    }

    Ok(row_result)
}

fn query_(
    query: String,
    bindings: Option<Vec<SqlValue>>,
    ContextData(host_defined): ContextData<HostData>,
    ctx: &mut Context,
) -> JsResult<JsValue> {
    let query = build_query_(query, bindings, ctx)?;
    let app = host_defined.app_mut();

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

fn execute_(
    query: String,
    bindings: Option<Vec<SqlValue>>,
    ContextData(host_defined): ContextData<HostData>,
    ctx: &mut Context,
) -> JsResult<JsValue> {
    let query = build_query_(query, bindings, ctx)?;
    let app = host_defined.app_mut();

    let db = app.database();
    let mut db = db.lock().unwrap();
    let result = db
        .execute_returning_count(&query)
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;

    Ok(JsValue::from(result))
}

fn query_one_(
    query: String,
    bindings: Option<Vec<SqlValue>>,
    ContextData(host_defined): ContextData<HostData>,
    ctx: &mut Context,
) -> JsResult<JsValue> {
    let query = build_query_(query, bindings, ctx)?;
    let app = host_defined.app_mut();

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

pub fn create_module(context: &mut Context) -> JsResult<(JsString, Module)> {
    let module = [
        (
            js_string!("execute"),
            execute_.into_js_function_copied(context),
        ),
        (
            js_string!("queryOne"),
            query_one_.into_js_function_copied(context),
        ),
        (js_string!("query"), query_.into_js_function_copied(context)),
    ]
    .into_js_module(context);

    Ok((js_string!("db"), module))
}

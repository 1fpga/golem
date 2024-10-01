use boa_engine::builtins::typed_array::TypedArray;
use boa_engine::object::builtins::{JsArray, JsPromise, JsUint8Array};
use boa_engine::value::TryFromJs;
use boa_engine::{js_error, Context, JsObject, JsResult, JsString, JsValue};
use boa_interop::{js_class, JsClass};
use boa_macros::{Finalize, JsData, Trace};
use std::path::PathBuf;
use tracing::trace;

/// A value in a column in SQL, compatible with JavaScript.
pub enum SqlValue {
    Binary(Vec<u8>),
    String(String),
    Number(f64),
    Integer(i64),
    Null,
}

impl SqlValue {
    pub fn into_js_value(self, context: &mut Context) -> JsResult<JsValue> {
        Ok(match self {
            SqlValue::String(s) => JsString::from(s).into(),
            SqlValue::Number(f) => f.into(),
            SqlValue::Integer(i) => i.into(),
            SqlValue::Null => JsValue::null(),
            SqlValue::Binary(b) => JsUint8Array::from_iter(b.into_iter(), context)?.into(),
        })
    }
}

impl TryFromJs for SqlValue {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::Null => Ok(Self::Null),
            JsValue::Boolean(b) => Ok(Self::Integer(*b as i64)),
            JsValue::String(s) => Ok(Self::String(s.to_std_string_escaped())),
            JsValue::Rational(r) => Ok(Self::Number(*r)),
            JsValue::Integer(i) => Ok(Self::Integer(*i as i64)),
            JsValue::Object(o) if o.is::<TypedArray>() => {
                let array = JsUint8Array::from_object(o.clone())?;
                Ok(Self::Binary(array.iter(context).collect()))
            }
            _ => Err(js_error!(
                "Invalid value type {}, cannot convert to SQL.",
                value.type_of()
            )
            .into()),
        }
    }
}

impl TryFrom<sqlite::Value> for SqlValue {
    type Error = sqlite::Error;

    fn try_from(value: sqlite::Value) -> Result<Self, Self::Error> {
        match value {
            sqlite::Value::Binary(b) => Ok(SqlValue::Binary(b)),
            sqlite::Value::Integer(i) => Ok(SqlValue::Integer(i.into())),
            sqlite::Value::Float(f) => Ok(SqlValue::Number(f)),
            sqlite::Value::String(s) => Ok(SqlValue::String(s)),
            sqlite::Value::Null => Ok(SqlValue::Null),
        }
    }
}

impl TryFrom<&sqlite::Value> for SqlValue {
    type Error = sqlite::Error;

    fn try_from(value: &sqlite::Value) -> Result<Self, Self::Error> {
        match value {
            sqlite::Value::Binary(b) => Ok(SqlValue::Binary(b.clone())),
            sqlite::Value::Integer(i) => Ok(SqlValue::Integer(*i)),
            sqlite::Value::Float(f) => Ok(SqlValue::Number(*f)),
            sqlite::Value::String(s) => Ok(SqlValue::String(s.clone())),
            sqlite::Value::Null => Ok(SqlValue::Null),
        }
    }
}

impl From<SqlValue> for sqlite::Value {
    fn from(value: SqlValue) -> Self {
        match value {
            SqlValue::Binary(b) => sqlite::Value::Binary(b),
            SqlValue::Integer(i) => sqlite::Value::Integer(i as i64),
            SqlValue::Number(f) => sqlite::Value::Float(f),
            SqlValue::String(s) => sqlite::Value::String(s),
            SqlValue::Null => sqlite::Value::Null,
        }
    }
}

fn build_query_<'a>(
    connection: &'a sqlite::Connection,
    query: String,
    bindings: Option<Vec<SqlValue>>,
    _ctx: &mut Context,
) -> JsResult<sqlite::Statement<'a>> {
    let mut q = connection
        .prepare(query)
        .map_err(|e| js_error!("SQL Error: {}", e))?;
    if let Some(bindings) = bindings {
        let bindings = bindings
            .into_iter()
            .map(SqlValue::into)
            .collect::<Vec<sqlite::Value>>();
        q.bind(bindings.as_slice())
            .map_err(|e| js_error!("SQL Error: {}", e))?;
    }

    Ok(q)
}

fn create_row_object<'a>(
    row: sqlite::Row,
    mappings: &[String],
    ctx: &mut Context,
) -> JsResult<JsObject> {
    let row_result = JsObject::with_null_proto();
    for name in mappings {
        let value: SqlValue = row.read(name.as_str());

        row_result.set(
            JsString::from(name.as_str()),
            value.into_js_value(ctx)?,
            true,
            ctx,
        )?;
    }
    Ok(row_result)
}

fn create_row_object_array<'a>(
    mut statement: sqlite::Statement,
    ctx: &mut Context,
) -> JsResult<JsArray> {
    let mappings = statement.column_names().to_vec();
    let row_results = JsArray::new(ctx);

    for row in statement.iter() {
        let row = row.map_err(|e| js_error!("SQL Error: {}", e))?;
        row_results.push(create_row_object(row, &mappings, ctx)?, ctx)?;
    }

    Ok(row_results)
}

fn db_root() -> PathBuf {
    if cfg!(feature = "platform_de10") {
        PathBuf::from("/media/fat/golem")
    } else {
        let d = directories::BaseDirs::new()
            .expect("Could not find BaseDirs")
            .config_local_dir()
            .join("golem");
        if !d.exists() {
            std::fs::create_dir_all(&d).unwrap();
        }
        d
    }
}

#[derive(Trace, Finalize, JsData)]
pub struct JsDb {
    #[unsafe_ignore_trace]
    connection: sqlite::Connection,
}

impl JsDb {
    pub fn new(name: &str) -> JsResult<Self> {
        let path = db_root().join(format!("{}.sqlite", name));
        trace!("Opening database at {:?}", path);
        let connection =
            sqlite::Connection::open(path).map_err(|e| js_error!("SQL Error: {}", e))?;

        Ok(Self { connection })
    }

    pub fn query(
        &self,
        query: String,
        bindings: Option<Vec<SqlValue>>,
        context: &mut Context,
    ) -> JsPromise {
        JsPromise::new(
            move |fns, context| {
                let statement = build_query_(&self.connection, query, bindings, context)?;

                let rows = create_row_object_array(statement, context)?;
                let result = JsObject::with_null_proto();
                result.set(JsString::from("rows"), rows, true, context)?;
                fns.resolve
                    .call(&JsValue::null(), &[result.into()], context)?;
                Ok(JsValue::undefined())
            },
            context,
        )
    }

    pub fn query_one(
        &self,
        query: String,
        bindings: Option<Vec<SqlValue>>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let mut statement = build_query_(&self.connection, query, bindings, context)?;
        let mappings = statement.column_names().to_vec();
        let first_row = statement
            .iter()
            .next()
            .transpose()
            .map_err(|e| js_error!("SQL Error: {}", e))?;

        if let Some(first_row) = first_row {
            let result = create_row_object(first_row, &mappings, context)?;
            Ok(result.into())
        } else {
            Ok(JsValue::null())
        }
    }

    pub fn execute(
        &self,
        query: String,
        bindings: Option<Vec<SqlValue>>,
        context: &mut Context,
    ) -> JsResult<usize> {
        let mut statement = build_query_(&self.connection, query, bindings, context)?;

        if statement
            .next()
            .map_err(|e| js_error!(Error: "SQL Error: {}", e))?
            != sqlite::State::Done
        {
            return Err(js_error!(Error: "SQL Error: query did not complete"));
        }

        Ok(self.connection.change_count())
    }

    pub fn execute_raw(&self, query: String) -> JsResult<()> {
        self.connection
            .execute(query)
            .map_err(|e| js_error!("SQL Error: {}", e))?;
        Ok(())
    }
}

js_class! {
    class JsDb as "Db" {
        constructor(name: JsString) {
            JsDb::new(name.to_std_string().map_err(|_| js_error!("Invalid db name"))?.as_str())
        }

        fn query(this: JsClass<JsDb>, query: String, bindings: Option<Vec<SqlValue>>, context: &mut Context) -> JsPromise {
            this.borrow().query(query, bindings, context)
        }

        fn query_one as "queryOne"(this: JsClass<JsDb>, query: String, bindings: Option<Vec<SqlValue>>, context: &mut Context) -> JsResult<JsValue> {
            this.borrow().query_one(query, bindings, context)
        }

        fn execute(this: JsClass<JsDb>, query: String, bindings: Option<Vec<SqlValue>>, context: &mut Context) -> JsResult<usize> {
            this.borrow().execute(query, bindings, context)
        }

        fn execute_raw as "executeRaw"(this: JsClass<JsDb>, query: String) -> JsResult<()> {
            this.borrow().execute_raw(query)
        }
    }
}

use boa_engine::builtins::typed_array::TypedArray;
use boa_engine::class::Class;
use boa_engine::object::builtins::{JsArray, JsPromise, JsUint8Array};
use boa_engine::value::TryFromJs;
use boa_engine::{js_error, Context, JsObject, JsResult, JsString, JsValue};
use boa_interop::{js_class, JsClass};
use boa_macros::{Finalize, JsData, Trace};
use ouroboros::self_referencing;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, Value, ValueRef};
use rusqlite::{Connection, Row, Statement};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use tracing::trace;

/// A value in a column in SQL, compatible with JavaScript.
pub enum SqlValue {
    Json(serde_json::Value),
    Binary(Vec<u8>),
    String(JsString),
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
            SqlValue::Json(json) => JsValue::from_json(&json, context)?,
        })
    }
}

impl FromSql for SqlValue {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Null => Ok(SqlValue::Null),
            ValueRef::Integer(i) => Ok(SqlValue::Integer(i)),
            ValueRef::Real(f) => Ok(SqlValue::Number(f)),
            ValueRef::Text(s) => Ok(SqlValue::String(JsString::from(
                std::str::from_utf8(s).map_err(|_| FromSqlError::InvalidType)?,
            ))),
            ValueRef::Blob(b) => Ok(SqlValue::Binary(b.to_vec())),
        }
    }
}

impl TryFromJs for SqlValue {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::Null => Ok(Self::Null),
            JsValue::Boolean(b) => Ok(Self::Integer(*b as i64)),
            JsValue::String(s) => Ok(Self::String(s.clone())),
            JsValue::Rational(r) => Ok(Self::Number(*r)),
            JsValue::Integer(i) => Ok(Self::Integer(*i as i64)),
            JsValue::Object(o) if o.is::<TypedArray>() => {
                let array = JsUint8Array::from_object(o.clone())?;
                Ok(Self::Binary(array.iter(context).collect()))
            }
            o => Ok(Self::Json(o.to_json(context)?)),
        }
    }
}

impl TryFrom<&Value> for SqlValue {
    type Error = rusqlite::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Blob(b) => Ok(SqlValue::Binary(b.clone())),
            Value::Integer(i) => Ok(SqlValue::Integer(*i)),
            Value::Real(f) => Ok(SqlValue::Number(*f)),
            Value::Text(s) => Ok(SqlValue::String(JsString::from(s.as_str()))),
            Value::Null => Ok(SqlValue::Null),
        }
    }
}

impl From<SqlValue> for Value {
    fn from(value: SqlValue) -> Self {
        match value {
            SqlValue::Binary(b) => Value::Blob(b),
            SqlValue::Integer(i) => Value::Integer(i as i64),
            SqlValue::Number(f) => Value::Real(f),
            SqlValue::String(s) => Value::Text(s.to_std_string_escaped()),
            SqlValue::Null => Value::Null,
            SqlValue::Json(json) => Value::Text(json.to_string()),
        }
    }
}

fn build_query_<'a>(
    connection: &'a Connection,
    query: &String,
    bindings: Option<Vec<SqlValue>>,
    _ctx: &mut Context,
) -> JsResult<Statement<'a>> {
    let mut q = connection
        .prepare(query)
        .map_err(|e| js_error!("SQL Error: {}", e))?;

    if let Some(bindings) = bindings {
        for (i, b) in bindings
            .into_iter()
            .map(|v| -> Value { v.into() })
            .enumerate()
        {
            q.raw_bind_parameter(i + 1, b)
                .map_err(|e| js_error!("SQL Error: {}", e))?;
        }
    }

    Ok(q)
}

fn create_row_object<'a>(row: &Row, mappings: &[String], ctx: &mut Context) -> JsResult<JsObject> {
    let row_result = JsObject::with_null_proto();
    for name in mappings {
        let value: SqlValue = row
            .get(name.as_str())
            .map_err(|e| js_error!("SQL Error: {}", e))?;

        row_result.set(
            JsString::from(name.as_str()),
            value.into_js_value(ctx)?,
            true,
            ctx,
        )?;
    }
    Ok(row_result)
}

fn create_row_object_array<'a>(mut statement: Statement, ctx: &mut Context) -> JsResult<JsArray> {
    let mappings = statement
        .column_names()
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
    let row_results = JsArray::new(ctx);

    let mut rows = statement.raw_query();
    while let Some(row) = rows.next().map_err(|e| js_error!("SQL Error: {}", e))? {
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

fn first_row(mut statement: Statement, context: &mut Context) -> JsResult<JsValue> {
    let mappings = statement
        .column_names()
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
    let mut rows = statement.raw_query();

    if let Some(first_row) = rows
        .next()
        .map_err(|e| js_error!(Error: "SQL Error: {}", e))?
    {
        let result = create_row_object(first_row, &mappings, context)?;
        Ok(result.into())
    } else {
        Ok(JsValue::null())
    }
}

#[self_referencing]
struct JsDbInner {
    next_id: u32,

    connection: Connection,

    #[borrows(connection)]
    #[covariant]
    transactions: HashMap<u32, rusqlite::Transaction<'this>>,
}

impl JsDbInner {
    fn connection(&self, tx: Option<u32>) -> JsResult<&Connection> {
        if let Some(tx) = tx {
            Ok(self
                .borrow_transactions()
                .get(&tx)
                .ok_or_else(|| js_error!("Transaction {} not found", tx))?)
        } else {
            Ok(self.borrow_connection())
        }
    }

    pub fn create_transaction(&mut self) -> JsResult<u32> {
        self.with_mut(|this| {
            *this.next_id += 1;
            let id = *this.next_id;

            this.transactions.insert(
                id,
                this.connection
                    .unchecked_transaction()
                    .map_err(|e| js_error!("Could not create transaction: {}", e))?,
            );
            Ok(id)
        })
    }

    pub fn query(
        &self,
        tx: Option<u32>,
        query: &String,
        bindings: Option<Vec<SqlValue>>,
        context: &mut Context,
    ) -> JsPromise {
        JsPromise::new(
            move |fns, context| {
                let connection = self.connection(tx)?;
                let statement = build_query_(&*connection, query, bindings, context)?;
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
        tx: Option<u32>,
        query: &String,
        bindings: Option<Vec<SqlValue>>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let connection = self.connection(tx)?;
        let statement = build_query_(&*connection, query, bindings, context)?;
        let result = first_row(statement, context);

        result
    }

    pub fn execute(
        &self,
        tx: Option<u32>,
        query: &String,
        bindings: Option<Vec<SqlValue>>,
        context: &mut Context,
    ) -> JsResult<usize> {
        let connection = self.connection(tx)?;
        let mut statement = build_query_(&*connection, query, bindings, context)?;
        let count = statement
            .raw_execute()
            .map_err(|e| js_error!(Error: "SQL Error: {}", e))?;

        Ok(count)
    }

    pub fn execute_raw(&self, tx: Option<u32>, query: String) -> JsResult<()> {
        let connection = self.connection(tx)?;
        let result = connection
            .execute_batch(&query)
            .map_err(|e| js_error!("SQL Error: {}", e));
        result
    }

    pub fn execute_many(
        &self,
        tx: Option<u32>,
        query: &String,
        bindings: Vec<Vec<SqlValue>>,
        _context: &mut Context,
    ) -> JsResult<usize> {
        let connection = self.connection(tx)?;
        let mut statement = connection
            .prepare(query)
            .map_err(|e| js_error!("SQL Error: {}", e))?;
        let mut total = 0;

        for binding in bindings {
            for (i, b) in binding
                .into_iter()
                .map(|v| -> Value { v.into() })
                .enumerate()
            {
                statement
                    .raw_bind_parameter(i + 1, b)
                    .map_err(|e| js_error!("SQL Error: {}", e))?;
            }

            total += statement
                .raw_execute()
                .map_err(|e| js_error!("SQL Error: {}", e))?;
        }

        Ok(total)
    }
}

#[derive(Trace, Finalize, JsData)]
pub struct JsDb {
    #[unsafe_ignore_trace]
    inner: Rc<RefCell<JsDbInner>>,
}

impl JsDb {
    pub fn new(name: &str) -> JsResult<Self> {
        let path = db_root().join(format!("{}.sqlite", name));
        trace!("Opening database at {:?}", path);
        let connection = Connection::open(path).map_err(|e| js_error!("SQL Error: {}", e))?;

        Ok(Self {
            inner: Rc::new(RefCell::new(JsDbInner::new(0, connection, |_| {
                HashMap::new()
            }))),
        })
    }

    pub fn query(
        &self,
        query: &String,
        bindings: Option<Vec<SqlValue>>,
        context: &mut Context,
    ) -> JsPromise {
        self.inner.borrow().query(None, query, bindings, context)
    }

    pub fn query_one(
        &self,
        query: &String,
        bindings: Option<Vec<SqlValue>>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        self.inner
            .borrow()
            .query_one(None, query, bindings, context)
    }

    pub fn execute(
        &self,
        query: &String,
        bindings: Option<Vec<SqlValue>>,
        context: &mut Context,
    ) -> JsResult<usize> {
        self.inner.borrow().execute(None, query, bindings, context)
    }

    pub fn execute_raw(&self, query: String) -> JsResult<()> {
        self.inner.borrow().execute_raw(None, query)
    }

    pub fn execute_many(
        &self,
        query: &String,
        bindings: Vec<Vec<SqlValue>>,
        context: &mut Context,
    ) -> JsResult<usize> {
        self.inner
            .borrow()
            .execute_many(None, query, bindings, context)
    }

    pub fn begin_transaction(&self, context: &mut Context) -> JsPromise {
        JsPromise::new(
            move |fns, context| {
                let tx_id = self.inner.borrow_mut().create_transaction()?;
                let inner = self.inner.clone();

                let transaction = JsDbTransaction { inner, tx_id };

                fns.resolve.call(
                    &JsValue::null(),
                    &[JsDbTransaction::from_data(transaction, context)?.into()],
                    context,
                )?;
                Ok(JsValue::undefined())
            },
            context,
        )
    }
}

js_class! {
    class JsDb as "Db" {
        constructor(name: JsString) {
            JsDb::new(name.to_std_string().map_err(|_| js_error!("Invalid db name"))?.as_str())
        }

        fn begin_transaction as "beginTransaction"(this: JsClass<JsDb>, context: &mut Context) -> JsPromise {
            this.borrow_mut().begin_transaction(context)
        }

        fn query(this: JsClass<JsDb>, query: String, bindings: Option<Vec<SqlValue>>, context: &mut Context) -> JsPromise {
            this.borrow().query(&query, bindings, context)
        }

        fn query_one as "queryOne"(this: JsClass<JsDb>, query: String, bindings: Option<Vec<SqlValue>>, context: &mut Context) -> JsResult<JsValue> {
            this.borrow().query_one(&query, bindings, context)
        }

        fn execute(this: JsClass<JsDb>, query: String, bindings: Option<Vec<SqlValue>>, context: &mut Context) -> JsResult<usize> {
            this.borrow().execute(&query, bindings, context)
        }

        fn execute_raw as "executeRaw"(this: JsClass<JsDb>, query: String) -> JsResult<()> {
            this.borrow().execute_raw(query)
        }

        fn execute_many as "executeMany"(this: JsClass<JsDb>, query: String, bindings: Vec<Vec<SqlValue>>, context: &mut Context) -> JsResult<usize> {
            this.borrow().execute_many(&query, bindings, context)
        }
    }
}

#[derive(Trace, Finalize, JsData)]
pub struct JsDbTransaction {
    #[unsafe_ignore_trace]
    inner: Rc<RefCell<JsDbInner>>,

    tx_id: u32,
}

impl JsDbTransaction {
    pub fn query(
        &self,
        query: &String,
        bindings: Option<Vec<SqlValue>>,
        context: &mut Context,
    ) -> JsPromise {
        self.inner
            .borrow()
            .query(Some(self.tx_id), query, bindings, context)
    }

    pub fn query_one(
        &self,
        query: &String,
        bindings: Option<Vec<SqlValue>>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        self.inner
            .borrow()
            .query_one(Some(self.tx_id), query, bindings, context)
    }

    pub fn execute(
        &self,
        query: &String,
        bindings: Option<Vec<SqlValue>>,
        context: &mut Context,
    ) -> JsResult<usize> {
        self.inner
            .borrow()
            .execute(Some(self.tx_id), query, bindings, context)
    }

    pub fn execute_many(
        &self,
        query: &String,
        bindings: Vec<Vec<SqlValue>>,
        context: &mut Context,
    ) -> JsResult<usize> {
        self.inner
            .borrow()
            .execute_many(Some(self.tx_id), query, bindings, context)
    }

    pub fn execute_raw(&self, query: String) -> JsResult<()> {
        self.inner.borrow().execute_raw(Some(self.tx_id), query)
    }

    pub fn commit(&mut self, context: &mut Context) -> JsPromise {
        JsPromise::new(
            move |fns, context| {
                self.inner
                    .borrow_mut()
                    .with_transactions_mut(|txs| -> JsResult<()> {
                        txs.remove(&self.tx_id)
                            .ok_or_else(|| js_error!("Transaction {} not found", self.tx_id))?
                            .commit()
                            .map_err(|e| js_error!("SQL Error: {}", e))
                    })?;

                fns.resolve.call(&JsValue::null(), &[], context)?;
                Ok(JsValue::undefined())
            },
            context,
        )
    }

    pub fn rollback(&mut self, context: &mut Context) -> JsPromise {
        JsPromise::new(
            move |fns, context| {
                self.inner
                    .borrow_mut()
                    .with_transactions_mut(|txs| -> JsResult<()> {
                        txs.remove(&self.tx_id)
                            .ok_or_else(|| js_error!("Transaction {} not found", self.tx_id))?
                            .rollback()
                            .map_err(|e| js_error!("SQL Error: {}", e))
                    })?;

                fns.resolve.call(&JsValue::null(), &[], context)?;
                Ok(JsValue::undefined())
            },
            context,
        )
    }
}

js_class! {
    class JsDbTransaction as "DbTransaction" {
        constructor() {
            Err(js_error!("Cannot construct DbTransaction directly"))
        }

        fn query(this: JsClass<JsDbTransaction>, query: String, bindings: Option<Vec<SqlValue>>, context: &mut Context) -> JsPromise {
            this.borrow().query(&query, bindings, context)
        }

        fn query_one as "queryOne"(this: JsClass<JsDbTransaction>, query: String, bindings: Option<Vec<SqlValue>>, context: &mut Context) -> JsResult<JsValue> {
            this.borrow().query_one(&query, bindings, context)
        }

        fn execute(this: JsClass<JsDbTransaction>, query: String, bindings: Option<Vec<SqlValue>>, context: &mut Context) -> JsResult<usize> {
            this.borrow().execute(&query, bindings, context)
        }

        fn execute_many as "executeMany"(this: JsClass<JsDbTransaction>, query: String, bindings: Vec<Vec<SqlValue>>, context: &mut Context) -> JsResult<usize> {
            this.borrow().execute_many(&query, bindings, context)
        }

        fn execute_raw as "executeRaw"(this: JsClass<JsDbTransaction>, query: String) -> JsResult<()> {
            this.borrow().execute_raw(query)
        }

        fn commit(this: JsClass<JsDbTransaction>, context: &mut Context) -> JsPromise {
            this.borrow_mut().commit(context)
        }

        fn rollback(this: JsClass<JsDbTransaction>, context: &mut Context) -> JsPromise {
            this.borrow_mut().rollback(context)
        }
    }
}

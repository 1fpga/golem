use boa_engine::{
    js_error, js_string, Context, JsError, JsObject, JsResult, JsString, JsValue, Module,
};
use boa_interop::{IntoJsFunctionCopied, IntoJsModule};
use boa_macros::{Finalize, JsData, Trace};
use std::cell::RefCell;
use std::rc::Rc;
use url::Url;
use valico::json_schema::Scope;

#[derive(Clone, Debug, JsData, Trace, Finalize)]
struct SchemaScope {
    #[unsafe_ignore_trace]
    scope: Rc<RefCell<Scope>>,
}

fn validate_(value: JsValue, schema: JsObject, context: &mut Context) -> JsResult<bool> {
    let schema = JsValue::Object(schema);
    let binding = context
        .get_data::<SchemaScope>()
        .ok_or(js_error!("Context missing schema scope"))?;
    let scope = binding.scope.clone();

    let mut schema_binding = scope.borrow_mut();
    let schema = schema_binding
        .compile_and_return(schema.to_json(context)?, true)
        .map_err(JsError::from_rust)?;

    let value = value.to_json(context)?;
    let mut result = schema.validate(&value);
    // Take the top errors. No need to add more (there could be thousands).
    result.errors = result.errors.drain(..).take(20).collect();
    result.missing = result.missing.drain(..).take(20).collect();

    if result.is_valid() {
        Ok(true)
    } else {
        let errors = serde_json::to_value(result).map_err(JsError::from_rust)?;
        Err(js_error!(JsValue::from_json(&errors, context)?))
    }
}

fn validate_with_id_(value: JsValue, schema: String, context: &mut Context) -> JsResult<bool> {
    let binding = context
        .get_data::<SchemaScope>()
        .ok_or(js_error!("Context missing schema scope"))?;
    let scope = binding.scope.clone();

    let schema_binding = scope.borrow_mut();
    let schema = schema_binding
        .resolve(&Url::parse(&schema).map_err(JsError::from_rust)?)
        .ok_or(js_error!("Schema not found"))?;

    let value = value.to_json(context)?;
    let mut result = schema.validate(&value);
    // Take the top errors. No need to add more (there could be thousands).
    result.errors = result.errors.drain(..).take(20).collect();
    result.missing = result.missing.drain(..).take(20).collect();

    if result.is_valid() {
        Ok(true)
    } else {
        let errors = serde_json::to_value(result).map_err(JsError::from_rust)?;
        Err(js_error!(JsValue::from_json(&errors, context)?))
    }
}

fn add_schema_(schema: JsObject, context: &mut Context) -> JsResult<JsString> {
    let binding = context
        .get_data::<SchemaScope>()
        .ok_or(js_error!("Context missing schema scope"))?;
    let scope = binding.scope.clone();

    let id = scope
        .borrow_mut()
        .compile(JsValue::Object(schema).to_json(context)?, false)
        .map_err(JsError::from_rust)?;

    Ok(JsString::from(id.as_str()))
}

pub fn create_module(context: &mut Context) -> JsResult<(JsString, Module)> {
    context.insert_data(SchemaScope {
        scope: Rc::new(RefCell::new(Scope::new())),
    });

    Ok((
        js_string!("schema"),
        [
            (
                js_string!("validate"),
                validate_.into_js_function_copied(context),
            ),
            (
                js_string!("validateWithId"),
                validate_with_id_.into_js_function_copied(context),
            ),
            (
                js_string!("addSchema"),
                add_schema_.into_js_function_copied(context),
            ),
        ]
        .into_js_module(context),
    ))
}

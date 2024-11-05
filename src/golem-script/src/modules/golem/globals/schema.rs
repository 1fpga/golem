use boa_engine::{js_error, js_string, Context, JsObject, JsResult, JsValue};
use boa_interop::{js_class, IntoJsFunctionCopied};
use boa_macros::{Finalize, JsData, Trace};
use valico::json_schema::schema::ScopedSchema;
use valico::json_schema::Scope;

/// A JSON Schema that can validate JsValues.
#[derive(Trace, Finalize, JsData)]
pub struct JsJsonSchema {
    #[unsafe_ignore_trace]
    schema: ScopedSchema<'static>,
}

impl JsJsonSchema {
    /// Validate a value from a schema and an object.
    pub fn js_validate_or_throw(
        schema: JsObject,
        value: JsValue,
        context: &mut Context,
    ) -> JsResult<bool> {
        let mut scope = Scope::new();
        let object = JsValue::Object(schema).to_json(context)?;
        let schema = scope
            .compile_and_return(object, false)
            .map_err(|e| js_error!("{}", e))?;

        let state = schema.validate(&value.to_json(context)?);
        if state.is_valid() {
            Ok(true)
        } else {
            let mut error = String::new();
            for e in state.errors {
                error.push_str(&format!("{}\n", e));
            }
            Err(js_error!("{}", error))
        }
    }
}

js_class! {
    class JsJsonSchema as "JsonSchema" {
        constructor(schema: JsObject) {
            Err(js_error!("Cannot construct JsJsonSchema."))
        }

        init(class: &mut ClassBuilder) -> JsResult<()> {
            let validate_or_throw = Self::js_validate_or_throw.into_js_function_copied(class.context());

            class.static_method(js_string!("validateOrThrow"), 2, validate_or_throw);

            Ok(())
        }
    }
}

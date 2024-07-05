use boa_engine::{js_string, Context, JsError, JsResult, JsString, JsValue, Module};
use boa_interop::{ContextData, IntoJsFunctionCopied, IntoJsModule};

use crate::HostData;

fn get_settings_(
    ContextData(data): ContextData<HostData>,
    context: &mut Context,
) -> JsResult<JsValue> {
    let json = data.app().settings().as_json_value();
    JsValue::from_json(&json, context)
}

fn update_settings_(
    ContextData(data): ContextData<HostData>,
    settings: JsValue,
    context: &mut Context,
) -> JsResult<()> {
    let settings = settings.to_json(context)?;

    data.app()
        .settings()
        .update_from_json(settings)
        .map_err(|e| JsError::from_opaque(JsString::from(e).into()))?;
    Ok(())
}

pub fn create_module(context: &mut Context) -> JsResult<(JsString, Module)> {
    Ok((
        js_string!("settings"),
        [
            (
                js_string!("getSettings"),
                get_settings_.into_js_function_copied(context),
            ),
            (
                js_string!("updateSettings"),
                update_settings_.into_js_function_copied(context),
            ),
        ]
        .into_js_module(context),
    ))
}

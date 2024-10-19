use crate::HostData;
use boa_engine::value::TryFromJs;
use boa_engine::{js_error, js_string, Context, JsError, JsResult, JsString, JsValue, Module};
use boa_interop::{ContextData, IntoJsFunctionCopied, IntoJsModule};
use boa_macros::js_str;

#[derive(Debug, Clone, Copy)]
enum FontSize {
    Small,
    Medium,
    Large,
}

impl TryFromJs for FontSize {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        const s = value.to_string(context)?;
        if s == js_str!("small") {
            Ok(Self::Small)
        } else if s == js_str!("medium") {
            Ok(Self::Medium)
        } else if s == js_str!("large") {
            Ok(Self::Large)
        } else {
            Err(js_error!("Invalid font size"))
        }
    }
}

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

fn set_font_size_(ContextData(data): ContextData<HostData>, size: FontSize) -> JsResult<()> {
    data.app().settings().set_font_size(size );
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
            (
                js_string!("setFontSize"),
                set_font_size_.into_js_function_copied(context),
            ),
        ]
        .into_js_module(context),
    ))
}

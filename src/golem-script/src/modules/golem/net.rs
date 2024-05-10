use boa_engine::{js_string, Context, JsError, JsResult, JsString, JsValue, Module};
use boa_interop::{IntoJsFunctionCopied, IntoJsModule};
use std::str::FromStr;

fn fetch_json_(url: String, ctx: &mut Context) -> JsResult<JsValue> {
    reqwest::blocking::get(&url)
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?
        .text()
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))
        .and_then(|text| {
            JsValue::from_json(
                &serde_json::Value::from_str(&text)
                    .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?,
                ctx,
            )
        })
}

fn download_file_(url: String, path: String) -> JsResult<()> {
    let mut response = reqwest::blocking::get(&url)
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;

    let mut file = std::fs::File::create(path)
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;

    std::io::copy(&mut response, &mut file)
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;

    Ok(())
}

pub fn create_module(context: &mut Context) -> JsResult<(JsString, Module)> {
    Ok((
        js_string!("net"),
        [
            (
                js_string!("fetchJson"),
                fetch_json_.into_js_function_copied(context),
            ),
            (
                js_string!("downloadFile"),
                download_file_.into_js_function_copied(context),
            ),
        ]
        .into_js_module(context),
    ))
}

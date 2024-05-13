use boa_engine::{js_string, Context, JsError, JsResult, JsString, JsValue, Module};
use boa_interop::{IntoJsFunctionCopied, IntoJsModule};
use reqwest::header::CONTENT_DISPOSITION;
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

fn download_file_(url: String) -> JsResult<JsString> {
    let mut response = reqwest::blocking::get(&url)
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;

    let file_name = response
        .headers()
        .get(CONTENT_DISPOSITION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            let parts: Vec<&str> = header.split(';').collect();
            parts.iter().find_map(|part| {
                if part.trim().starts_with("filename=") {
                    Some(
                        part.trim_start_matches("filename=")
                            .trim_matches('"')
                            .to_string(),
                    )
                } else {
                    None
                }
            })
        })
        .unwrap_or_else(|| url.split('/').last().unwrap().to_string());

    let temp_dir = tempdir::TempDir::new("golem")
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;
    let path = temp_dir.path().join(file_name);

    let mut file = std::fs::File::create(path.clone())
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;

    std::io::copy(&mut response, &mut file)
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;

    Ok(JsString::from(path.display().to_string()))
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

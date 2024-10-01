use boa_engine::object::builtins::JsPromise;
use boa_engine::{js_string, Context, JsError, JsResult, JsString, JsValue, Module};
use boa_interop::{IntoJsFunctionCopied, IntoJsModule};
use reqwest::header::CONTENT_DISPOSITION;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

fn fetch_json_(url: String, ctx: &mut Context) -> JsResult<JsPromise> {
    let result = reqwest::blocking::get(&url)
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?
        .text()
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))
        .and_then(|text| {
            JsValue::from_json(
                &serde_json::Value::from_str(&text)
                    .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?,
                ctx,
            )
        });
    Ok(match result {
        Ok(v) => JsPromise::resolve(v, ctx),
        Err(e) => JsPromise::reject(e, ctx),
    })
}

fn download_(url: String, destination: Option<String>) -> JsResult<JsString> {
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

    let path = if let Some(dir) = destination {
        PathBuf::from(dir).join(file_name)
    } else {
        let temp_dir = tempdir::TempDir::new("golem")
            .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;
        temp_dir.path().join(file_name)
    };

    std::fs::create_dir_all(path.parent().unwrap())
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;
    let mut file = std::fs::File::create(path.clone())
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;

    std::io::copy(&mut response, &mut file)
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;

    Ok(JsString::from(path.display().to_string()))
}

fn is_online_(ctx: &mut Context) -> JsPromise {
    let is_online = ping::ping(
        [1, 1, 1, 1].into(),
        Some(Duration::from_secs(1)),
        None,
        None,
        None,
        None,
    )
    .is_ok();
    JsPromise::resolve(is_online, ctx)
}

pub fn create_module(context: &mut Context) -> JsResult<(JsString, Module)> {
    Ok((
        js_string!("net"),
        [
            (
                js_string!("isOnline"),
                is_online_.into_js_function_copied(context),
            ),
            (
                js_string!("fetchJson"),
                fetch_json_.into_js_function_copied(context),
            ),
            (
                js_string!("download"),
                download_.into_js_function_copied(context),
            ),
        ]
        .into_js_module(context),
    ))
}

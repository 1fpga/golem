use boa_engine::object::builtins::{JsArrayBuffer, JsPromise};
use boa_engine::{js_error, js_string, Context, JsResult, JsString, JsValue, Module};
use boa_interop::{IntoJsFunctionCopied, IntoJsModule};
use boa_macros::TryFromJs;
use std::ops::Deref;

#[derive(TryFromJs)]
struct IpsPatchOption {}

fn ips_patch_(
    rom: JsArrayBuffer,
    patch: JsArrayBuffer,
    options: Option<IpsPatchOption>,
    context: &mut Context,
) -> JsResult<JsPromise> {
    let rom_bytes = rom
        .data_mut()
        .ok_or_else(|| js_error!("Invalid rom ArrayBuffer"))?;
    let ips_bytes = patch
        .data()
        .ok_or_else(|| js_error!("Invalid patch ArrayBuffer"))?;

    let patch = ips::Patch::parse(ips_bytes.deref()).map_err(|e| js_error!(e.to_string()))?;

    for hunk in patch.hunks() {
        let offset = hunk.offset();
        let payload = hunk.payload();
        rom_bytes[offset..offset + payload.len()].copy_from_slice(payload);
    }

    if let Some(truncation) = patch.truncation() {
        rom_bytes.truncate(truncation);
    }

    Ok(JsPromise::resolve(JsValue::undefined(), context))
}

pub fn create_module(context: &mut Context) -> JsResult<(JsString, Module)> {
    Ok((
        js_string!("utils"),
        [(
            js_string!("ipsPatch"),
            ips_patch_.into_js_function_copied(context),
        )]
        .into_js_module(context),
    ))
}

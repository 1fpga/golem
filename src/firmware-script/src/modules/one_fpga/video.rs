use boa_engine::{js_error, js_string, Context, JsError, JsResult, JsString, Module};
use boa_interop::{ContextData, IntoJsFunctionCopied, IntoJsModule};
use std::str::FromStr;

use crate::HostData;
use mister_fpga::config::edid::DefaultVideoMode;
use mister_fpga::core::AsMisterCore;

fn set_mode_(mode: String, ContextData(data): ContextData<HostData>) -> JsResult<()> {
    let app = data.app_mut();
    let mut core = app
        .platform_mut()
        .core_manager_mut()
        .get_current_core()
        .ok_or_else(|| js_error!("No core loaded"))?;

    let core = core
        .as_mister_core_mut()
        .ok_or_else(|| js_error!("Core is not a MisterFpgaCore"))?;

    let video_mode = DefaultVideoMode::from_str(&mode).map_err(JsError::from_rust)?;

    eprintln!("Setting video mode: {:?}", video_mode);
    mister_fpga::core::video::select_mode(
        video_mode.into(),
        false,
        None,
        None,
        core.spi_mut(),
        true,
    )
    .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;
    Ok(())
}

pub fn create_module(context: &mut Context) -> JsResult<(JsString, Module)> {
    Ok((
        js_string!("video"),
        [(
            js_string!("setMode"),
            set_mode_.into_js_function_copied(context),
        )]
        .into_js_module(context),
    ))
}

use boa_engine::{js_string, Context, JsError, JsResult, JsString, Module};
use boa_interop::{ContextData, IntoJsFunctionCopied, IntoJsModule};
use one_fpga::Core;
use std::str::FromStr;

use mister_fpga::config::edid::DefaultVideoMode;
use mister_fpga::core::MisterFpgaCore;

use crate::HostData;

fn set_mode_(mode: String, ContextData(data): ContextData<HostData>) -> JsResult<()> {
    let app = data.app_mut();
    let core_manager = app.platform_mut().core_manager_mut();
    let mut golem_core = core_manager.get_current_core().unwrap();
    let core = golem_core
        .as_any_mut()
        .downcast_mut::<MisterFpgaCore>()
        .unwrap();

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

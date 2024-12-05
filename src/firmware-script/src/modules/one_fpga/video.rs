use crate::HostData;
use boa_engine::value::TryIntoJs;
use boa_engine::{js_error, js_string, Context, JsError, JsResult, JsString, JsValue, Module};
use boa_interop::{ContextData, IntoJsFunctionCopied, IntoJsModule};
use mister_fpga::config::edid::DefaultVideoMode;
use mister_fpga::config::resolution;
use mister_fpga::core::AsMisterCore;
use std::str::FromStr;

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

#[derive(Debug, TryIntoJs)]
pub struct Resolution {
    width: u64,
    height: u64,
}

impl From<resolution::Resolution> for Resolution {
    fn from(value: resolution::Resolution) -> Self {
        Self {
            width: value.width as u64,
            height: value.height as u64,
        }
    }
}

fn get_resolution_(
    ContextData(data): ContextData<HostData>,
    context: &mut Context,
) -> JsResult<Option<JsValue>> {
    let app = data.app_mut();
    let mut core = app
        .platform_mut()
        .core_manager_mut()
        .get_current_core()
        .ok_or_else(|| js_error!("No core loaded"))?;

    let Some(core) = core.as_menu_core_mut() else {
        return Ok(None);
    };

    let video_info = core
        .video_info()
        .map_err(|e| js_error!("Failed to get video info: {}", e))?;

    let resolution = video_info.fb_resolution();
    Ok(Some(Resolution::from(resolution).try_into_js(context)?))
}

pub fn create_module(context: &mut Context) -> JsResult<(JsString, Module)> {
    Ok((
        js_string!("video"),
        [
            (
                js_string!("setMode"),
                set_mode_.into_js_function_copied(context),
            ),
            (
                js_string!("getResolution"),
                get_resolution_.into_js_function_copied(context),
            ),
        ]
        .into_js_module(context),
    ))
}

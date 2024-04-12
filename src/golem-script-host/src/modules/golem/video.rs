use boa_engine::{Context, js_string, JsError, JsResult, JsString, Module};
use boa_interop::{ContextData, IntoJsFunctionCopied, IntoJsModule};

use golem_ui::platform::GoLEmPlatform;
use mister_fpga::config::edid::DefaultVideoMode;

use crate::HostData;

fn set_mode_(mode: String, ContextData(data): ContextData<HostData>) -> JsResult<()> {
    let app = data.app_mut();
    let core_manager = app.platform_mut().core_manager_mut();
    let mut golem_core = core_manager.get_current_core().unwrap();
    let core = golem_core.as_mister_mut().unwrap();

    let video_mode = match mode.as_str() {
        "V1920x1080r60" => DefaultVideoMode::V1920x1080r60,
        "V1920x1080r50" => DefaultVideoMode::V1920x1080r50,
        "V1280x720r60" => DefaultVideoMode::V1280x720r60,
        "V640x480r60" => DefaultVideoMode::V640x480r60,
        _ => {
            return Ok(());
        }
    };

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

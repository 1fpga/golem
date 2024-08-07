use std::path::PathBuf;

use boa_engine::class::Class;
use boa_engine::value::TryFromJs;
use boa_engine::{js_string, Context, JsError, JsResult, JsString, JsValue, Module};
use boa_interop::{ContextData, IntoJsFunctionCopied, IntoJsModule};
use boa_macros::{Finalize, JsData, Trace};
use one_fpga::core::Rom;
use one_fpga::runner::CoreLaunchInfo;
use serde::Deserialize;

use golem_ui::application::panels::core_loop::run_core_loop;

use crate::modules::golem::core::js_core::JsCore;
use crate::HostData;

pub mod js_core;

/// The core type from JavaScript.
#[derive(Debug, Trace, Finalize, JsData, Deserialize)]
#[serde(tag = "type")]
pub enum CoreType {
    Path { path: String },
}

/// The game type for JavaScript.
#[derive(Debug, Trace, Finalize, JsData, Deserialize)]
#[serde(tag = "type")]
pub enum GameType {
    RomPath { path: String },
}

#[derive(Debug, Trace, Finalize, JsData, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunOptions {
    core: CoreType,
    game: Option<GameType>,
    files: Option<Vec<Option<String>>>,
    savestate: Option<String>,
    show_menu: Option<bool>,
    auto_loop: Option<bool>,
}

impl TryFromJs for RunOptions {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        serde_json::from_value(value.to_json(context)?)
            .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))
    }
}

fn run_(
    options: RunOptions,
    ContextData(app): ContextData<HostData>,
    context: &mut Context,
) -> JsResult<JsValue> {
    let app = app.app_mut();
    let mut core_options = match &options.core {
        CoreType::Path { path } => CoreLaunchInfo::rbf(PathBuf::from(path)),
    };

    match &options.game {
        Some(GameType::RomPath { path }) => {
            core_options = core_options.with_rom(Rom::File(PathBuf::from(path)));
        }
        None => {}
    };

    if let Some(files) = &options.files {
        for (i, file) in files
            .iter()
            .enumerate()
            .filter_map(|(i, x)| x.as_ref().map(|x| (i, x)))
        {
            core_options
                .files
                .insert(i, one_fpga::runner::Slot::File(PathBuf::from(file)));
        }
    }

    eprintln!("Launching core: {:?}", core_options);
    let mut core = app
        .platform_mut()
        .core_manager_mut()
        .launch(core_options)
        .unwrap();

    if options.auto_loop.unwrap_or(true) {
        run_core_loop(&mut *app, &mut core, options.show_menu.unwrap_or(true));
        Ok(JsValue::undefined())
    } else {
        Ok(JsValue::Object(JsCore::from_data(
            JsCore::new(core),
            context,
        )?))
    }
}

pub fn create_module(context: &mut Context) -> JsResult<(JsString, Module)> {
    context.register_global_class::<JsCore>()?;

    Ok((
        js_string!("core"),
        [(js_string!("run"), run_.into_js_function_copied(context))].into_js_module(context),
    ))
}

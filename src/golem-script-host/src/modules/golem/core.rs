use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use boa_engine::{Context, js_string, JsResult, JsString, Module};
use boa_interop::{IntoJsFunctionUnsafe, IntoJsModule};
use boa_macros::{Finalize, JsData, Trace, TryFromJs};

use golem_core::runner::CoreLauncher;
use golem_ui::application::panels::core_loop::run_core_loop;
use golem_ui::platform::GoLEmPlatform;

#[derive()]
pub struct GolemCore {}

#[derive(Debug, Trace, Finalize, JsData, TryFromJs)]
struct RunOptions {
    core: String,
    game: Option<String>,
    sav: Option<String>,
    savestate: Option<String>,
    autoloop: Option<bool>,
}

fn run_(
    app: &mut golem_ui::application::GoLEmApp,
    options: RunOptions,
    _ctx: &mut Context,
) {
    let mut core_options =
        CoreLauncher::rbf(PathBuf::from(&options.core));
    if let Some(ref game) = options.game {
        core_options = core_options.with_file(PathBuf::from(game));
    }

    eprintln!("Launching core: {:?}", core_options);
    let mut core = core_options
        .launch(app.platform_mut().core_manager_mut())
        .unwrap();
    if options.autoloop.unwrap_or(true) {
        run_core_loop(app, &mut core, true);
    }
}

pub fn create_module(
    context: &mut Context,
    app: Rc<RefCell<golem_ui::application::GoLEmApp>>,
) -> JsResult<(JsString, Module)> {
    unsafe {
        let run = {
            let app = app.clone();
            move |options: RunOptions, context: &mut Context| {
                run_(&mut app.borrow_mut(), options, context)
            }
        }.into_js_function_unsafe(context);

        Ok((js_string!("core"),
            [
                (js_string!("run"), run),
            ].into_js_module(context)
        ))
    }
}

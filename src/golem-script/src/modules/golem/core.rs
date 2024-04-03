use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use boa_engine::{Context, js_string, JsResult, JsString, JsValue, Module};
use boa_interop::{IntoJsFunctionUnsafe, IntoJsModule};
use boa_macros::{Finalize, JsData, Trace, TryFromJs};

use golem_core::runner::CoreLauncher;
use golem_ui::application::panels::core_loop::run_core_loop;
use golem_ui::platform::GoLEmPlatform;

#[derive(Debug, Trace, Finalize, JsData, TryFromJs)]
struct RunOptions {
    core: String,
    game: Option<String>,
}

pub fn run(
    app: &mut golem_ui::application::GoLEmApp,
    options: RunOptions,
    ctx: &mut Context,
) -> JsResult<JsValue> {
    eprintln!("Options: {:?}", options);
    let mut core_options =
        CoreLauncher::rbf(PathBuf::from(&options.core));
    if let Some(game) = options.game {
        core_options = core_options.with_file(PathBuf::from(&game));
    }

    eprintln!("Launching core: {:?}", core_options);
    let core = core_options
        .launch(app.platform_mut().core_manager_mut())
        .unwrap();
    run_core_loop(app, core, true);
    Ok(JsValue::undefined())
}

pub fn create_module(
    context: &mut Context,
    app: Rc<RefCell<golem_ui::application::GoLEmApp>>,
) -> JsResult<(JsString, Module)> {
    unsafe {
        let run = {
            let app = app.clone();
            move |options: RunOptions| {
                run(&mut app.borrow_mut(), options, context)
            }
        }.into_js_function_unsafe(context);

        Ok((js_string!("core"),
            [
                (js_string!("run"), run),
            ].into_js_module(context)
        ))
    }
}

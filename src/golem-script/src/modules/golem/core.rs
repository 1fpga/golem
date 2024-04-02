use boa_engine::{js_string, Context, JsError, JsResult, JsString, JsValue, Module};
use boa_macros::{Finalize, JsData, Trace, TryFromJs};
use golem_core::runner::CoreLauncher;
use golem_ui::application::panels::core_loop::run_core_loop;
use golem_ui::platform::GoLEmPlatform;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Debug, Trace, Finalize, JsData, TryFromJs)]
struct RunOptions {
    core: String,
    game: String,
}

pub fn run(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
    app: &mut golem_ui::application::GoLEmApp,
) -> JsResult<JsValue> {
    eprintln!("Running core");
    let Some(options) = args.get(0) else {
        return Err(JsError::from_opaque(
            js_string!("No options provided").into(),
        ));
    };

    eprintln!("Options: {}", options.display());
    let options = options.try_js_into::<RunOptions>(ctx)?;
    let core_options =
        CoreLauncher::rbf(PathBuf::from(&options.core)).with_file(PathBuf::from(&options.game));

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
    let run = boa_engine::object::FunctionObjectBuilder::new(
        context.realm(),
        boa_engine::NativeFunction::from_copy_closure({
            let app = Rc::downgrade(&app).as_ptr();
            move |_this, args, ctx| run(_this, args, ctx, unsafe { &mut (*app).borrow_mut() })
        }),
    )
    .name(js_string!("run"))
    .build();

    Ok((
        js_string!("core"),
        Module::synthetic(
            // Make sure to list all exports beforehand.
            &[js_string!("run")],
            // The initializer is evaluated every time a module imports this synthetic module,
            // so we avoid creating duplicate objects by capturing and cloning them instead.
            boa_engine::module::SyntheticModuleInitializer::from_copy_closure_with_captures(
                |module, fns, _| {
                    module.set_export(&js_string!("run"), fns.0.clone().into())?;

                    Ok(())
                },
                (run,),
            ),
            None,
            context,
        ),
    ))
}

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use boa_engine::{Context, js_string, JsError, Module, Source};
use boa_engine::builtins::promise::PromiseState;
use boa_engine::property::Attribute;
use tracing::{error, info};

use crate::module_loader::GolemModuleLoader;

mod module_loader;

mod console;
mod modules;

pub fn run(
    script: Option<&impl AsRef<Path>>,
    mut app: golem_ui::application::GoLEmApp,
) -> Result<(), Box<dyn std::error::Error>> {
    app.init_platform();
    let app = Rc::new(RefCell::new(app));

    let script_path = script.expect("No script provided").as_ref();
    let dir = script_path.parent().unwrap();

    let loader = Rc::new(GolemModuleLoader::new(dir).unwrap());

    // Instantiate the execution context
    let mut context = Context::builder().module_loader(loader.clone()).build()?;

    // Initialize the Console object.
    let console = console::Console::init(&mut context);

    // Register the console as a global property to the context.
    context
        .register_global_property(
            js_string!(console::Console::NAME),
            console,
            Attribute::all(),
        )
        .expect("The console object shouldn't exist yet");

    modules::register_modules(loader.clone(), &mut context, app.clone())?;

    let source = Source::from_reader(std::fs::File::open(script_path)?, Some(script_path));

    // Can also pass a `Some(realm)` if you need to execute the module in another realm.
    let module = Module::parse(source, None, &mut context)?;

    let promise_result = module.load_link_evaluate(&mut context);

    let result = loop {
        // Very important to push forward the job queue after queueing promises.
        context.run_jobs();

        // Checking if the final promise didn't return an error.
        match promise_result.state() {
            PromiseState::Pending => {}
            PromiseState::Fulfilled(v) => {
                break v;
            }
            PromiseState::Rejected(err) => {
                error!("Javascript Error: {}", err.display());
                return Err(JsError::from_opaque(err).try_native(&mut context)?.into());
            }
        }
    };

    info!(?result, "Script executed successfully.");

    Ok(())
}

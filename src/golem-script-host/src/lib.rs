use std::path::Path;
use std::rc::Rc;

use boa_engine::{Context, js_string, JsError, Module, Source};
use boa_engine::builtins::promise::PromiseState;
use boa_engine::property::Attribute;
use boa_macros::{Finalize, JsData, Trace};
use tracing::{error, info};

use golem_ui::application::GoLEmApp;

use crate::module_loader::GolemModuleLoader;

mod module_loader;

mod console;
mod modules;

/// The application type for HostDefined information.
#[derive(Clone, Trace, Finalize, JsData)]
pub(crate) struct HostData {
    // TODO: remove the pointer. This is safe because the JS code
    //       stops execution before the App is dropped, but it would
    //       be better to have a safe way to handle this.
    //       A RefCell isn't good enough because it's recursive.
    /// The GoLEm application.
    #[unsafe_ignore_trace]
    app: Rc<*mut GoLEmApp>,
}

impl std::fmt::Debug for HostData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HostData").finish()
    }
}

impl HostData {
    pub fn app_mut(&self) -> &mut golem_ui::application::GoLEmApp {
        unsafe { self.app.as_mut().unwrap() }
    }
}

pub fn run(
    script: Option<&impl AsRef<Path>>,
    mut app: golem_ui::application::GoLEmApp,
) -> Result<(), Box<dyn std::error::Error>> {
    app.init_platform();
    let app = Rc::new((&mut app) as *mut GoLEmApp);
    let host_defined = HostData { app };

    let script_path = script.expect("No script provided").as_ref();
    let dir = script_path.parent().unwrap();

    let loader = Rc::new(GolemModuleLoader::new(dir).unwrap());

    // Instantiate the execution context
    let mut context = Context::builder().module_loader(loader.clone()).build()?;
    context.insert_data(host_defined);

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

    modules::register_modules(loader.clone(), &mut context)?;

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

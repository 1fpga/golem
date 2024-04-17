use std::path::Path;
use std::rc::Rc;

use boa_engine::{Context, js_string, JsError, JsValue, Module, Source};
use boa_engine::builtins::promise::PromiseState;
use boa_engine::module::ModuleLoader;
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
    pub fn app_mut(&self) -> &mut GoLEmApp {
        unsafe { self.app.as_mut().unwrap() }
    }
}

fn create_context(
    script: Option<&impl AsRef<Path>>,
    host_defined: HostData,
) -> Result<(Context, Rc<GolemModuleLoader>), JsError> {
    let loader = match script {
        Some(p) => {
            let dir = p.as_ref().parent().expect("Cannot use root.");

            Rc::new(GolemModuleLoader::new(dir).expect("Could not find the script folder."))
        }
        None => Rc::new(GolemModuleLoader::default())
    };

    let mut context = Context::builder().module_loader(loader.clone()).build()?;

    context.insert_data(host_defined);

    Ok((context, loader))
}

pub fn run(
    script: Option<&impl AsRef<Path>>,
    mut app: golem_ui::application::GoLEmApp,
) -> Result<(), Box<dyn std::error::Error>> {
    app.init_platform();
    let app = Rc::new((&mut app) as *mut GoLEmApp);
    let host_defined = HostData { app };

    let (mut context, loader) = create_context(script, host_defined)?;

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

    let module = match script {
        Some(script_path) => {
            let source = Source::from_reader(
                std::fs::File::open(script_path)?,
                Some(script_path.as_ref()),
            );

            // Can also pass a `Some(realm)` if you need to execute the module in another realm.
            Module::parse(source, None, &mut context)?
        }
        None => {
            let source = Source::from_bytes(b"export { main } from '/main.js';");
            Module::parse(source, None, &mut context)?
        }
    };

    let promise_result = module.load_link_evaluate(&mut context);

    loop {
        // Very important to push forward the job queue after queueing promises.
        context.run_jobs();

        // Checking if the final promise didn't return an error.
        match promise_result.state() {
            PromiseState::Pending => {}
            PromiseState::Fulfilled(_) => {
                break;
            }
            PromiseState::Rejected(err) => {
                error!("Javascript Error: {}", err.display());
                return Err(JsError::from_opaque(err).try_native(&mut context)?.into());
            }
        }
    };

    let main_fn = module.namespace(&mut context).get(js_string!("main"), &mut context)?;
    let result = main_fn.as_callable().expect("Main was not callable").call(&JsValue::undefined(), &[], &mut context)?;

    context.run_jobs();

    info!(?result, "Script executed successfully.");

    Ok(())
}

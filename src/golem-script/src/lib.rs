use crate::module_loader::GolemModuleLoader;
use crate::modules::ui;
use boa_engine::builtins::promise::PromiseState;
use boa_engine::property::Attribute;
use boa_engine::{js_string, Context, JsError, JsString, JsValue, Module, Source};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

mod module_loader;

mod console;
mod modules;

mod utils;

pub fn run(
    script: Option<&impl AsRef<Path>>,
    app: Rc<RefCell<golem_ui::application::GoLEmApp>>,
) -> Result<(), Box<dyn std::error::Error>> {
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

    let (name, ui_module) = ui::create_module(&mut context, app.clone())?;
    let module_name = JsString::concat(&js_string!("golem/"), &name);
    loader.insert_named(module_name, ui_module);

    let source = Source::from_reader(std::fs::File::open(script_path)?, Some(script_path));

    // Can also pass a `Some(realm)` if you need to execute the module in another realm.
    let module = Module::parse(source, None, &mut context)?;

    let promise_result = module.load_link_evaluate(&mut context);

    // Very important to push forward the job queue after queueing promises.
    context.run_jobs();

    // Checking if the final promise didn't return an error.
    match promise_result.state() {
        PromiseState::Pending => return Err("module didn't execute!".into()),
        PromiseState::Fulfilled(v) => {
            assert_eq!(v, JsValue::undefined());
        }
        PromiseState::Rejected(err) => {
            return Err(JsError::from_opaque(err).try_native(&mut context)?.into())
        }
    }

    Ok(())
}

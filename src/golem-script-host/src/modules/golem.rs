use std::cell::RefCell;
use std::rc::Rc;

use boa_engine::{Context, js_string, JsResult, JsString};

use golem_ui::application::GoLEmApp;

use crate::module_loader::GolemModuleLoader;

pub mod core;
pub mod db;
pub mod ui;

pub(super) fn register_modules(
    loader: Rc<GolemModuleLoader>,
    context: &mut Context,
    app: Rc<RefCell<GoLEmApp>>,
) -> JsResult<()> {
    for create_fn in [core::create_module, db::create_module, ui::create_module] {
        let (name, module) = create_fn(context, app.clone())?;
        let module_name = JsString::concat(&js_string!("golem/"), &name);
        loader.insert_named(module_name, module);
    }

    Ok(())
}

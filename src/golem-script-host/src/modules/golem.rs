use std::rc::Rc;

use boa_engine::{Context, js_string, JsResult, JsString};

use crate::module_loader::GolemModuleLoader;

pub mod core;
pub mod db;
pub mod ui;

pub(super) fn register_modules(
    loader: Rc<GolemModuleLoader>,
    context: &mut Context,
) -> JsResult<()> {
    for create_fn in [core::create_module, db::create_module, ui::create_module] {
        let (name, module) = create_fn(context)?;
        let module_name = JsString::concat(&js_string!("golem/"), &name);
        loader.insert_named(module_name, module);
    }


    Ok(())
}

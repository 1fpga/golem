use std::rc::Rc;

use boa_engine::{js_string, Context, JsResult, JsString, Module};

use crate::module_loader::GolemModuleLoader;

mod core;
mod db;
mod storage;
mod ui;
mod video;

pub(super) fn register_modules(
    loader: Rc<GolemModuleLoader>,
    context: &mut Context,
) -> JsResult<()> {
    let modules = [
        core::create_module,
        db::create_module,
        storage::create_module,
        video::create_module,
        ui::create_module,
    ];

    for create_fn in modules.iter() {
        let (name, module) = create_fn(context)?;
        let module_name = JsString::concat(&js_string!("golem/"), &name);
        loader.insert_named(module_name, module);
    }

    // The patrons module.
    loader.insert_named(
        js_string!("golem/patrons"),
        Module::parse_json(
            js_string!(include_str!("../../../../scripts/patreon/patrons.json")),
            context,
        )?,
    );

    Ok(())
}

use std::rc::Rc;

use boa_engine::{js_string, Context, JsResult, JsString, Module};
use boa_macros::js_str;

use crate::module_loader::GolemModuleLoader;

mod commands;
mod core;
mod db;
mod fs;
mod net;
mod schema;
mod settings;
mod ui;
mod upgrade;
mod video;

mod globals;

pub use commands::CommandMap;
pub use globals::classes::*;

pub(super) fn register_modules(
    loader: Rc<GolemModuleLoader>,
    context: &mut Context,
) -> JsResult<()> {
    let modules = [
        commands::create_module,
        core::create_module,
        db::create_module,
        fs::create_module,
        net::create_module,
        schema::create_module,
        settings::create_module,
        ui::create_module,
        upgrade::create_module,
        video::create_module,
    ];

    for create_fn in modules.iter() {
        let (name, module) = create_fn(context)?;
        let module_name = JsString::concat(js_str!("@:golem/"), name.as_str());
        loader.insert_named(module_name, module);
    }

    // The patrons module.
    loader.insert_named(
        js_string!("@:golem/patrons"),
        Module::parse_json(
            js_string!(include_str!("../../../../scripts/patreon/patrons.json")),
            context,
        )?,
    );

    globals::register_globals(context)?;

    Ok(())
}

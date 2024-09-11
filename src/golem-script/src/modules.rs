use std::rc::Rc;

use boa_engine::JsResult;

use crate::module_loader::GolemModuleLoader;

mod fs;
mod golem;

pub use golem::CommandMap;

pub(super) fn register_modules(
    loader: Rc<GolemModuleLoader>,
    context: &mut boa_engine::Context,
) -> JsResult<()> {
    golem::register_modules(loader.clone(), context)?;
    fs::register_modules(loader, context)?;
    Ok(())
}

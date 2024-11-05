use std::rc::Rc;

use boa_engine::JsResult;

use crate::module_loader::GolemModuleLoader;

mod golem;

pub use golem::*;

pub(super) fn register_modules(
    loader: Rc<GolemModuleLoader>,
    context: &mut boa_engine::Context,
) -> JsResult<()> {
    golem::register_modules(loader.clone(), context)?;
    Ok(())
}

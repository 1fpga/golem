use std::cell::RefCell;
use std::rc::Rc;

use boa_engine::JsResult;

use crate::module_loader::GolemModuleLoader;

mod golem;

pub(super) fn register_modules(
    loader: Rc<GolemModuleLoader>,
    context: &mut boa_engine::Context,
    app: Rc<RefCell<golem_ui::application::GoLEmApp>>,
) -> JsResult<()> {
    golem::register_modules(loader, context, app)?;
    Ok(())
}

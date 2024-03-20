use crate::module_loader::GolemModuleLoader;
use boa_engine::JsResult;
use std::cell::RefCell;
use std::rc::Rc;

mod golem;

pub(super) fn register_modules(
    loader: Rc<GolemModuleLoader>,
    context: &mut boa_engine::Context,
    app: Rc<RefCell<golem_ui::application::GoLEmApp>>,
) -> JsResult<()> {
    golem::register_modules(loader, context, app)?;
    Ok(())
}

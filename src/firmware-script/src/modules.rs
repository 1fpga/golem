use std::rc::Rc;

use boa_engine::JsResult;

use crate::module_loader::OneFpgaModuleLoader;

mod one_fpga;

pub use one_fpga::*;

pub(super) fn register_modules(
    loader: Rc<OneFpgaModuleLoader>,
    context: &mut boa_engine::Context,
) -> JsResult<()> {
    one_fpga::register_modules(loader.clone(), context)?;
    Ok(())
}

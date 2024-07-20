mod commands;
mod js_core;

mod classes {
    pub use super::commands::JsCommand;
    pub use super::js_core::JsCore;
}

pub fn register_globals(context: &mut boa_engine::Context) -> boa_engine::JsResult<()> {
    context.register_global_class::<classes::JsCommand>()?;
    context.register_global_class::<classes::JsCore>()?;
    Ok(())
}

//! Console module based on BOA's own implementation (in `boa_runtime`) but uses
//! `tracing` for logging instead of `println!`.

use boa_engine::{Context, JsResult, JsString};
use boa_gc::Trace;
use boa_macros::Finalize;
use boa_runtime::{ConsoleState, Logger};
use tracing::{error, info, warn};

fn stack(context: &mut Context) -> Vec<String> {
    context
        .stack_trace()
        .map(|frame| frame.code_block().name())
        .map(JsString::to_std_string_escaped)
        .collect::<Vec<_>>()
}

#[derive(Debug, Trace, Finalize)]
pub struct TracingLogger;

impl Logger for TracingLogger {
    fn log(&self, msg: String, state: &ConsoleState, context: &mut Context) -> JsResult<()> {
        let indent = state.indent();
        if tracing::enabled!(tracing::Level::TRACE) {
            let stack = stack(context);
            info!(?stack, "{msg:>indent$}");
        } else {
            info!("{msg:>indent$}");
        }
        Ok(())
    }

    fn info(&self, msg: String, state: &ConsoleState, context: &mut Context) -> JsResult<()> {
        self.log(msg, state, context)
    }

    fn warn(&self, msg: String, state: &ConsoleState, context: &mut Context) -> JsResult<()> {
        let indent = state.indent();
        if tracing::enabled!(tracing::Level::TRACE) {
            let stack = stack(context);
            warn!(?stack, "{msg:>indent$}");
        } else {
            warn!("{msg:>indent$}");
        }
        Ok(())
    }

    fn error(&self, msg: String, state: &ConsoleState, context: &mut Context) -> JsResult<()> {
        let indent = state.indent();
        if tracing::enabled!(tracing::Level::TRACE) {
            let stack = stack(context);
            error!(?stack, "{msg:>indent$}");
        } else {
            error!("{msg:>indent$}");
        }
        Ok(())
    }
}

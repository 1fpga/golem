use crate::modules::CommandMap;
use crate::modules::JsCore;
use boa_engine::class::Class;
use boa_engine::object::builtins::JsFunction;
use boa_engine::{Context, JsError, JsValue};
use firmware_ui::application::OneFpgaApp;
use firmware_ui::input::commands::CommandId;
use mister_fpga::core::AsMisterCore;
use std::time::Instant;
use tracing::{debug, trace};

fn call_command_inner(
    app: &mut OneFpgaApp,
    command: &JsFunction,
    context: &mut Context,
) -> Result<(), JsError> {
    // If core is the menu, do not pass it to the command.
    let core = app
        .platform_mut()
        .core_manager_mut()
        .get_current_core()
        .and_then(|core| {
            // The only "non" core is the main menu core.
            if core.as_menu_core().is_some() {
                None
            } else {
                Some(core)
            }
        });

    let core = core.map_or(JsValue::undefined(), |c| {
        JsValue::Object(JsCore::from_data(JsCore::new(c.clone()), context).unwrap())
    });
    let result = command.call(&JsValue::undefined(), &[core], context)?;

    // If the command returns a promise, wait for it to resolve (or reject).
    if let Some(p) = result.as_promise() {
        debug!("Waiting for promise to resolve...");
        p.await_blocking(context).map_err(JsError::from_opaque)?;
    }
    Ok(())
}

pub fn maybe_call_command(
    app: &mut OneFpgaApp,
    id: CommandId,
    command_map: &mut CommandMap,
    context: &mut Context,
) -> Result<(), JsError> {
    let start = Instant::now();
    if let Some(command) = command_map.get(id) {
        trace!("Calling command: {:?}", id);
        match call_command_inner(app, command, context) {
            Ok(_) => {
                debug!(elapsed = ?start.elapsed(), "Command executed successfully.");
                Ok(())
            }
            Err(error) => {
                debug!(elapsed = ?start.elapsed(), ?error, "Command failed.");
                Err(error)
            }
        }
    } else {
        trace!("Command not found: {:?}", id);

        // Command not found, ignore.
        Ok(())
    }
}

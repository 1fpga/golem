use crate::HostData;
use boa_engine::object::builtins::JsFunction;
use boa_engine::{js_error, js_string, Context, JsResult, JsString, Module};
use boa_interop::{ContextData, IntoJsFunctionCopied, IntoJsModule};
use boa_macros::{Finalize, JsData, Trace};
use golem_ui::input::commands::CommandId;
use golem_ui::input::shortcut::Shortcut;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::AtomicUsize;
use tracing::debug;

#[derive(Default, Trace, Finalize, JsData)]
pub struct CommandMap {
    #[unsafe_ignore_trace]
    inner: HashMap<CommandId, JsFunction>,

    #[unsafe_ignore_trace]
    next_id: AtomicUsize,
}

impl CommandMap {
    pub fn next_id(&self) -> CommandId {
        CommandId::from_id(
            self.next_id
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        )
    }

    pub fn get(&self, id: CommandId) -> Option<&JsFunction> {
        self.inner.get(&id)
    }

    pub fn insert(&mut self, id: CommandId, shortcut: JsFunction) {
        self.inner.insert(id, shortcut);
    }

    pub fn remove(&mut self, id: &CommandId) -> Option<JsFunction> {
        self.inner.remove(id)
    }
}

fn create_shortcut_(
    ContextData(data): ContextData<HostData>,
    shortcut: String,
    action: JsFunction,
) -> JsResult<()> {
    debug!(?shortcut, "Creating shortcut");
    let shortcut =
        Shortcut::from_str(&shortcut).map_err(|e| js_error!("Invalid shortcut: {:?}", e))?;

    let app = data.app_mut();
    let command_map = data.command_map_mut();
    let id = command_map.next_id();
    command_map.insert(id, action);
    app.add_shortcut(shortcut, id);

    Ok(())
}

fn remove_shortcut_(ContextData(data): ContextData<HostData>, shortcut: String) -> JsResult<()> {
    debug!(?shortcut, "Removing shortcut");
    let shortcut =
        Shortcut::from_str(&shortcut).map_err(|e| js_error!("Invalid shortcut: {:?}", e))?;

    let app = data.app_mut();
    let command_map = data.command_map_mut();

    if let Some(command_id) = app.remove_shortcut(&shortcut) {
        command_map.remove(&command_id);
    }

    Ok(())
}

pub fn create_module(context: &mut Context) -> JsResult<(JsString, Module)> {
    Ok((
        js_string!("commands"),
        [
            (
                js_string!("createShortcut"),
                create_shortcut_.into_js_function_copied(context),
            ),
            (
                js_string!("removeShortcut"),
                remove_shortcut_.into_js_function_copied(context),
            ),
        ]
        .into_js_module(context),
    ))
}

use crate::modules::golem::globals::classes::{JsCommand, JsCore};
use crate::HostData;
use boa_engine::class::Class;
use boa_engine::object::builtins::JsFunction;
use boa_engine::{js_string, Context, JsResult, JsString, JsValue, Module};
use boa_interop::{ContextData, IntoJsFunctionCopied, IntoJsModule};
use boa_macros::{Finalize, JsData, Trace};
use golem_ui::application::GoLEmApp;
use golem_ui::input::commands::CommandId;
use golem_ui::input::shortcut::Shortcut;
use one_fpga::{Core, GolemCore};
use std::collections::HashMap;
use std::time::Instant;
use tracing::debug;

#[derive(Clone, Default, Trace, Finalize, JsData)]
pub struct CommandMap {
    #[unsafe_ignore_trace]
    inner: HashMap<CommandId, Command>,
}

impl CommandMap {
    pub fn get(&self, id: CommandId) -> Option<&Command> {
        self.inner.get(&id)
    }

    pub fn get_mut(&mut self, id: CommandId) -> Option<&mut Command> {
        self.inner.get_mut(&id)
    }

    pub fn insert(&mut self, id: CommandId, command: Command) {
        self.inner.insert(id, command);
    }
}

/// A command type.
#[derive(Debug, Clone)]
pub enum CommandType {
    General,
    Core,
    CoreSpecific(String),
}

impl CommandType {
    pub fn matches(&self, running_core: Option<&GolemCore>) -> bool {
        match self {
            CommandType::General => true,
            CommandType::Core => running_core.is_some(),
            CommandType::CoreSpecific(core) => {
                running_core.map_or(false, |c| c.name() == core.as_str())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Command {
    pub ty: CommandType,
    pub short_name: String,
    pub name: String,
    pub description: String,
    pub action: JsFunction,
    pub shortcuts: Vec<Shortcut>,
}

impl Command {
    pub fn execute(
        &mut self,
        _app: &mut GoLEmApp,
        running_core: Option<&GolemCore>,
        context: &mut Context,
    ) -> JsResult<()> {
        let start = Instant::now();

        if self.ty.matches(running_core) {
            let core = running_core.map_or(JsValue::undefined(), |c| {
                JsValue::Object(JsCore::from_data(JsCore::new(c.clone()), context).unwrap())
            });
            self.action.call(&JsValue::undefined(), &[core], context)?;
            context.run_jobs();

            debug!(elapsed = ?start.elapsed(), "Command {:?} executed successfully", self.short_name);
            Ok(())
        } else {
            debug!(
                "Command {:?} does not match the current core",
                self.short_name
            );
            Ok(())
        }
    }
}

fn create_general_command_(
    ContextData(data): ContextData<HostData>,
    short_name: JsString,
    name: JsString,
    description: JsString,
    action: JsFunction,
    context: &mut Context,
) -> JsResult<JsValue> {
    let command = Command {
        ty: CommandType::General,
        short_name: short_name.to_std_string_escaped(),
        name: name.to_std_string_escaped(),
        description: description.to_std_string_escaped(),
        action,
        shortcuts: Default::default(),
    };
    data.command_map_mut()
        .insert(CommandId::new(command.short_name.as_str()), command.clone());

    JsCommand::new(command).into_object(context)
}

fn create_core_command_(
    ContextData(data): ContextData<HostData>,
    short_name: JsString,
    name: JsString,
    description: JsString,
    action: JsFunction,
    context: &mut Context,
) -> JsResult<JsValue> {
    let command = Command {
        ty: CommandType::Core,
        short_name: short_name.to_std_string_escaped(),
        name: name.to_std_string_escaped(),
        description: description.to_std_string_escaped(),
        action,
        shortcuts: Default::default(),
    };
    data.command_map_mut()
        .insert(CommandId::new(command.short_name.as_str()), command.clone());

    JsCommand::new(command).into_object(context)
}

fn create_core_specific_command_(
    ContextData(data): ContextData<HostData>,
    short_name: JsString,
    name: JsString,
    description: JsString,
    core: JsString,
    action: JsFunction,
    context: &mut Context,
) -> JsResult<JsValue> {
    let command = Command {
        ty: CommandType::CoreSpecific(core.to_std_string_escaped()),
        short_name: short_name.to_std_string_escaped(),
        name: name.to_std_string_escaped(),
        description: description.to_std_string_escaped(),
        action,
        shortcuts: Default::default(),
    };
    data.command_map_mut()
        .insert(CommandId::new(command.short_name.as_str()), command.clone());

    JsCommand::new(command).into_object(context)
}

pub fn create_module(context: &mut Context) -> JsResult<(JsString, Module)> {
    Ok((
        js_string!("commands"),
        [
            (
                js_string!("createGeneralCommand"),
                create_general_command_.into_js_function_copied(context),
            ),
            (
                js_string!("createCoreCommand"),
                create_core_command_.into_js_function_copied(context),
            ),
            (
                js_string!("createCoreSpecificCommand"),
                create_core_specific_command_.into_js_function_copied(context),
            ),
        ]
        .into_js_module(context),
    ))
}

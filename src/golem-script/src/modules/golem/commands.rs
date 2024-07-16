use crate::modules::golem::core::js_core::JsCore;
use crate::HostData;
use boa_engine::class::Class;
use boa_engine::object::builtins::{JsArray, JsFunction};
use boa_engine::{js_error, js_string, Context, JsResult, JsString, JsValue, Module};
use boa_interop::{js_class, ContextData, IntoJsFunctionCopied, IntoJsModule, JsClass};
use boa_macros::{Finalize, JsData, Trace};
use golem_ui::application::GoLEmApp;
use golem_ui::data::settings::commands::CommandId;
use golem_ui::input::shortcut::Shortcut;
use one_fpga::{Core, GolemCore};
use std::collections::HashMap;
use std::str::FromStr;

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

/// A command that can be run, in JavaScript. This corresponds to the
/// `Command` class in JS.
#[derive(Clone, Trace, Finalize, JsData)]
pub struct JsCommand {
    #[unsafe_ignore_trace]
    command: Command,
}

impl JsCommand {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn into_object(self, context: &mut Context) -> JsResult<JsValue> {
        JsCommand::from_data(self, context).map(JsValue::Object)
    }

    pub fn execute(&self, app: &mut GoLEmApp, context: &mut Context) -> JsResult<()> {
        let running_core = app.platform_mut().core_manager_mut().get_current_core();
        if self.command.ty.matches(running_core.as_ref()) {
            let core = running_core.map_or(JsValue::undefined(), |c| {
                JsValue::Object(JsCore::from_data(JsCore::new(c), context).unwrap())
            });
            self.command
                .action
                .call(&JsValue::undefined(), &[core], context)?;
            context.run_jobs();
            Ok(())
        } else {
            Err(js_error!("Command does not match the current core"))
        }
    }

    pub fn set_shortcuts(&mut self, shortcuts: Vec<String>) -> JsResult<()> {
        self.command.shortcuts = shortcuts
            .into_iter()
            .map(|s| Shortcut::from_str(s.as_str()).map_err(|e| js_error!(js_string!(e))))
            .collect::<JsResult<Vec<Shortcut>>>()?;
        Ok(())
    }
}

js_class! {
    class JsCommand as "Command" {
        property short_name as "shortName" {
            get(this: JsClass<JsCommand>) -> JsResult<JsString> {
                Ok(JsString::from(this.borrow().command.short_name.as_str()))
            }
        }

        property name {
            get(this: JsClass<JsCommand>) -> JsResult<JsString> {
                Ok(JsString::from(this.borrow().command.name.as_str()))
            }
        }

        property description {
            get(this: JsClass<JsCommand>) -> JsResult<JsString> {
                Ok(JsString::from(this.borrow().command.description.as_str()))
            }
        }

        property shortcuts {
            fn get(this: JsClass<JsCommand>, context: &mut Context) -> JsResult<JsValue> {
                let shortcuts = JsArray::from_iter(
                    this.borrow().command.shortcuts.iter().map(|s| {
                        JsValue::from(js_string!(s.to_string()))
                    }),
                    context,
                );

                Ok(shortcuts.into())
            }

            fn set(this: JsClass<JsCommand>, new_shortcuts: Vec<String>) -> JsResult<()> {
                this.borrow_mut().set_shortcuts(new_shortcuts)
            }
        }

        constructor(short_name: JsString, name: JsString, description: JsString) {
            Err(js_error!("Cannot create a command from JS"))
        }

        fn execute(this: JsClass<JsCommand>, host: ContextData<HostData>, context: &mut Context) -> JsResult<()> {
            this.borrow().execute(host.0.app_mut(), context)
        }
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

fn create_general_command_(
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

    Ok(JsCommand::new(command).into_object(context)?)
}

fn create_core_command_(
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

    Ok(JsCommand::new(command).into_object(context)?)
}

fn create_core_specific_command_(
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

    Ok(JsCommand::new(command).into_object(context)?)
}

pub fn create_module(context: &mut Context) -> JsResult<(JsString, Module)> {
    context.register_global_class::<JsCommand>()?;

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

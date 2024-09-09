use crate::modules::golem::commands::Command;
use crate::modules::golem::globals::classes::JsCore;
use crate::HostData;
use boa_engine::class::Class;
use boa_engine::object::builtins::JsArray;
use boa_engine::{js_error, js_string, Context, JsResult, JsString, JsValue};
use boa_interop::{js_class, ContextData, JsClass};
use boa_macros::{Finalize, JsData, Trace};
use golem_ui::application::GoLEmApp;
use golem_ui::input::commands::CommandId;
use golem_ui::input::shortcut::Shortcut;
use std::str::FromStr;

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

    pub fn set_shortcuts(&mut self, shortcuts: Vec<String>, app: &mut GoLEmApp) -> JsResult<()> {
        // Remove previous shortcuts.
        let this_id = CommandId::new(&self.command.short_name);
        for shortcut in self.command.shortcuts.iter() {
            // Make sure the shortcut isn't registered to another command.
            if app.commands().get(shortcut) == Some(&this_id) {
                app.commands_mut().remove(shortcut);
            }
        }

        self.command.shortcuts = shortcuts
            .into_iter()
            .map(|s| Shortcut::from_str(s.as_str()).map_err(|e| js_error!(js_string!(e))))
            .collect::<JsResult<Vec<Shortcut>>>()?;

        for shortcut in self.command.shortcuts.iter() {
            app.commands_mut().insert(shortcut.clone(), this_id);
        }

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

            fn set(
                this: JsClass<JsCommand>,
                new_shortcuts: Vec<String>,
                host: ContextData<HostData>
            ) -> JsResult<()> {
                this.borrow_mut().set_shortcuts(new_shortcuts, host.0.app_mut())
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

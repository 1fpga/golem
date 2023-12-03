use crate::application::panels::core_loop::menu::core_menu;
use crate::application::GoLEmApp;
use crate::platform::Core;
use std::fmt::{Display, Formatter};
use tracing::{debug, error, warn};

/// Input commands that can be associated with a shortcut.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ShortcutCommand {
    /// Show the menu in the core.
    ShowCoreMenu,

    /// Reset the core.
    ResetCore,

    /// Quit the core and return to the main menu.
    QuitCore,

    /// This is a core-specific command, which is identified by a `u32` hash of its
    /// label. This allows cores to change the order of their bits, as long as
    /// the label stays the same. The hash is considered safe enough for this purpose
    /// (low risk of collision).
    CoreSpecificCommand(u32),
}

pub enum CommandResult {
    /// The command was executed successfully.
    Ok,

    /// The command was not executed successfully (with an error message).
    Err(String),

    /// Quit the core and return to the main menu.
    QuitCore,
}

impl Display for ShortcutCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ShortcutCommand::ShowCoreMenu => write!(f, "Show Menu"),
            ShortcutCommand::ResetCore => write!(f, "Reset Core"),
            ShortcutCommand::QuitCore => write!(f, "Quit Core"),
            // Command::SaveState => "Save State",
            // Command::LoadSaveState => "Load Save State",
            ShortcutCommand::CoreSpecificCommand(id) => write!(f, "Core Specific Command {id}"),
        }
    }
}

impl ShortcutCommand {
    pub fn globals() -> Vec<Self> {
        vec![
            ShortcutCommand::ShowCoreMenu,
            ShortcutCommand::ResetCore,
            ShortcutCommand::QuitCore,
        ]
    }

    pub fn execute(&self, app: &mut GoLEmApp, core: &mut impl Core) -> CommandResult {
        match self {
            ShortcutCommand::ShowCoreMenu => {
                debug!("Opening core menu");
                if core_menu(app, core) {
                    CommandResult::QuitCore
                } else {
                    CommandResult::Ok
                }
            }
            ShortcutCommand::ResetCore => {
                debug!("Resetting core");
                core.status_pulse(0);
                CommandResult::Ok
            }
            ShortcutCommand::QuitCore => {
                debug!("Quitting core");
                CommandResult::QuitCore
            }
            ShortcutCommand::CoreSpecificCommand(id) => {
                let menu = core.menu_options().iter().find(|m| {
                    eprintln!("{:?} == {} ({:?})", m.id(), id, m.label());
                    m.id() == Some(*id)
                });
                let menu = menu.cloned();
                debug!(
                    ?menu,
                    "Sending core-specific command {} ({})",
                    id,
                    menu.as_ref().and_then(|m| m.label()).unwrap_or("<unknown>")
                );

                if let Some(menu) = menu {
                    match core.trigger_menu(&menu) {
                        Ok(true) => {
                            debug!("Core-specific command {} triggered", id);
                            CommandResult::Ok
                        }
                        Ok(false) => {
                            debug!("Core-specific command {} not triggered", id);
                            CommandResult::Ok
                        }
                        Err(e) => {
                            error!("Error triggering menu: {}", e);
                            CommandResult::Err(e)
                        }
                    }
                } else {
                    warn!("Core-specific command {} not found", id);
                    CommandResult::Err("Core-specific command not found".to_string())
                }
            }
        }
    }
}

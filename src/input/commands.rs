use crate::application::panels::core_loop::menu::core_menu;
use crate::application::GoLEmApp;
use crate::data::paths;
use crate::input::shortcut::Shortcut;
use crate::platform::Core;
use image::GenericImageView;
use sdl3::keyboard::Scancode;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::time::Instant;
use tracing::{debug, error, info, warn};

/// Input commands that can be associated with a shortcut.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ShortcutCommand {
    /// Show the menu in the core.
    ShowCoreMenu,

    /// Reset the core.
    ResetCore,

    /// Quit the core and return to the main menu.
    QuitCore,

    /// Take a screenshot and saves it to the screenshots folder.
    TakeScreenshot,

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
            ShortcutCommand::TakeScreenshot => write!(f, "Take Screenshot"),
            ShortcutCommand::CoreSpecificCommand(id) => write!(f, "Core Specific Command {id}"),
        }
    }
}

impl FromStr for ShortcutCommand {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "show_menu" => Ok(ShortcutCommand::ShowCoreMenu),
            "reset_core" => Ok(ShortcutCommand::ResetCore),
            "quit_core" => Ok(ShortcutCommand::QuitCore),
            "take_screenshot" => Ok(ShortcutCommand::TakeScreenshot),
            _ => Err("Invalid shortcut"),
        }
    }
}

impl ShortcutCommand {
    pub fn globals() -> Vec<Self> {
        vec![
            ShortcutCommand::ShowCoreMenu,
            ShortcutCommand::ResetCore,
            ShortcutCommand::QuitCore,
            ShortcutCommand::TakeScreenshot,
        ]
    }

    pub fn setting_name(&self) -> Option<&'static str> {
        match self {
            ShortcutCommand::ShowCoreMenu => Some("show_menu"),
            ShortcutCommand::ResetCore => Some("reset_core"),
            ShortcutCommand::QuitCore => Some("quit_core"),
            ShortcutCommand::TakeScreenshot => Some("take_screenshot"),
            ShortcutCommand::CoreSpecificCommand(_) => None,
        }
    }

    pub fn default_shortcut(&self) -> Option<Shortcut> {
        match self {
            ShortcutCommand::ShowCoreMenu => Some(Shortcut::default().with_key(Scancode::F12)),
            ShortcutCommand::ResetCore => Some(Shortcut::default().with_key(Scancode::F11)),
            ShortcutCommand::QuitCore => Some(Shortcut::default().with_key(Scancode::F10)),
            ShortcutCommand::TakeScreenshot => Some(Shortcut::default().with_key(Scancode::SysReq)),
            ShortcutCommand::CoreSpecificCommand(_) => None,
        }
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
            ShortcutCommand::TakeScreenshot => {
                debug!("Taking screenshot");
                let start = Instant::now();
                let img = match core.take_screenshot() {
                    Ok(img) => img,
                    Err(e) => {
                        return CommandResult::Err(e);
                    }
                };

                let core_name = core.name();
                let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
                let path = paths::screenshots_root().join(format!(
                    "{}_{}.png",
                    core_name.replace(" ", "_"),
                    timestamp
                ));

                if let Err(e) = img.save(path.clone()) {
                    return CommandResult::Err(format!("Error saving screenshot: {}", e));
                }

                let elapsed = start.elapsed().as_millis();
                info!(
                    ?path,
                    dimensions = ?img.dimensions(),
                    elapsed,
                    "Screenshot taken."
                );

                CommandResult::Ok
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

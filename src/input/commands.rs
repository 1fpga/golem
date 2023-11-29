use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CoreCommands {
    ShowCoreMenu,
    ResetCore,
    QuitCore,

    CoreSpecificCommand(u32),
}

impl Display for CoreCommands {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreCommands::ShowCoreMenu => write!(f, "Show Menu"),
            CoreCommands::ResetCore => write!(f, "Reset Core"),
            CoreCommands::QuitCore => write!(f, "Quit Core"),
            // Command::SaveState => "Save State",
            // Command::LoadSaveState => "Load Save State",
            CoreCommands::CoreSpecificCommand(id) => write!(f, "Core Specific Command {id}"),
        }
    }
}

impl CoreCommands {
    pub fn globals() -> Vec<Self> {
        vec![
            CoreCommands::ShowCoreMenu,
            CoreCommands::ResetCore,
            CoreCommands::QuitCore,
        ]
    }
}

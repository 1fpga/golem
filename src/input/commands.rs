use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CoreCommands {
    ShowCoreMenu,
    QuitCore,
    // SaveState,
    // LoadSaveState,
}

impl Display for CoreCommands {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CoreCommands::ShowCoreMenu => "Show Menu",
                CoreCommands::QuitCore => "Quit Core",
                // Command::SaveState => "Save State",
                // Command::LoadSaveState => "Load Save State",
            }
        )
    }
}

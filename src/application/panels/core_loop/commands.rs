use crate::macguiver::application::Application;
use crate::platform::Core;
use embedded_graphics::pixelcolor::BinaryColor;

pub enum CommandReturn {
    BackToCore,
    QuitCore,
}

pub trait Command {
    fn description(&self) -> &str;

    fn execute(
        &mut self,
        app: impl Application<Color = BinaryColor>,
        core: impl Core,
    ) -> CommandReturn;
}

pub struct OpenMenuCommand;

impl Command for OpenMenuCommand {
    fn description(&self) -> &str {
        "Open Menu"
    }

    fn execute(
        &mut self,
        app: &mut impl Application<Color = BinaryColor>,
        core: &mut impl Core,
    ) -> CommandReturn {
        if super::menu::core_menu(app, core) {
            CommandReturn::QuitCore
        } else {
            CommandReturn::BackToCore
        }
    }
}

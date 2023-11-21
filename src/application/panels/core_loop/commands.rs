use crate::macguiver::application::Application;
use mister_fpga::core::buttons::MisterFpgaButtons;

pub enum Command {
    ShowCoreMenu,
    QuitCore,
    MiSTerFpgaButton(MisterFpgaButtons),
}

#![cfg(feature = "platform_desktop")]

use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::platform::sdl::settings::OutputSettingsBuilder;
use crate::macguiver::platform::sdl::theme::BinaryColorTheme;
use crate::macguiver::platform::sdl::{SdlInitState, SdlPlatform, Window};
use crate::macguiver::platform::{Platform, PlatformWindow};
use crate::main_inner::Flags;
use crate::platform::{sizes, Core, CoreManager, GoLEmPlatform};
use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::BinaryColor;
use mister_fpga::config_string::{ConfigMenu, LoadFileInfo};
use mister_fpga::types::StatusBitMap;
use sdl3::event::Event;
use sdl3::gamepad::{Axis, Button};
use sdl3::keyboard::Scancode;
use std::io::Read;
use std::path::Path;
use tracing::info;

impl crate::platform::SaveState for () {
    fn is_dirty(&self) -> bool {
        false
    }

    fn write_to(&mut self, _write: impl std::io::Write) -> Result<(), String> {
        Ok(())
    }

    fn read_from(&mut self, reader: impl Read) -> Result<(), String> {
        Ok(())
    }
}

pub struct DummyCore;

impl Core for DummyCore {
    type SaveState = ();

    fn name(&self) -> &str {
        "DummyCore"
    }

    fn load_file(&mut self, path: &Path, _file_info: Option<LoadFileInfo>) -> Result<(), String> {
        info!("DummyCore::load_file({:?})", path);
        Ok(())
    }

    fn version(&self) -> Option<&str> {
        None
    }

    fn menu_options(&self) -> &[ConfigMenu] {
        todo!()
    }

    fn trigger_menu(&mut self, _menu: &ConfigMenu) -> Result<bool, String> {
        unreachable!()
    }

    fn status_mask(&self) -> StatusBitMap {
        unreachable!()
    }

    fn status_bits(&self) -> StatusBitMap {
        unreachable!()
    }

    fn set_status_bits(&mut self, _bits: StatusBitMap) {
        unreachable!()
    }

    fn take_screenshot(&mut self) -> Result<image::DynamicImage, String> {
        unreachable!()
    }

    fn key_down(&mut self, key: Scancode) {
        info!("DummyCore::key_down({})", key);
    }

    fn key_up(&mut self, key: Scancode) {
        info!("DummyCore::key_up({})", key);
    }

    fn sdl_button_down(&mut self, joystick_idx: u8, button: Button) {
        info!(
            "DummyCore::sdl_joy_button_down({}, {})",
            joystick_idx,
            button.string()
        );
    }

    fn sdl_button_up(&mut self, joystick_idx: u8, button: Button) {
        info!(
            "DummyCore::sdl_joy_button_up({}, {})",
            joystick_idx,
            button.string()
        );
    }

    fn sdl_axis_motion(&mut self, controller: u8, axis: Axis, value: i16) {
        info!(
            "DummyCore::sdl_axis_motion({}, {:?}, {})",
            controller, axis, value
        );
    }

    fn save_states(&mut self) -> Option<&mut [()]> {
        None
    }
}

pub struct DummyCoreManager;

impl CoreManager for DummyCoreManager {
    type Core = DummyCore;

    fn load_core(&mut self, path: impl AsRef<Path>) -> Result<Self::Core, String> {
        info!("DummyCoreManager::load_core({:?})", path.as_ref());
        Ok(DummyCore)
    }

    fn load_menu(&mut self) -> Result<Self::Core, String> {
        info!("DummyCoreManager::load_menu()");
        Ok(DummyCore)
    }

    fn show_menu(&mut self) {}

    fn hide_menu(&mut self) {}
}

pub struct DesktopPlatform {
    platform: SdlPlatform<BinaryColor>,
    window_title: Window<BinaryColor>,
    window_main: Window<BinaryColor>,
    core_manager: DummyCoreManager,
}

impl Default for DesktopPlatform {
    fn default() -> Self {
        let mut platform = SdlPlatform::init(SdlInitState::new(
            OutputSettingsBuilder::new()
                .scale(3)
                .theme(BinaryColorTheme::LcdBlue)
                .build(),
        ));
        let mut window_title = platform.window("Title", sizes::TITLE);
        let mut window_main = platform.window("Title", sizes::MAIN);

        // Move the title above the window and make sure both are visible.
        let mut pos = window_main.position();
        pos.y -= sizes::TITLE.height as i32 + 128;
        window_title.set_position(pos);
        window_title.focus();
        window_main.focus();

        Self {
            platform,
            window_title,
            window_main,
            core_manager: DummyCoreManager,
        }
    }
}

impl GoLEmPlatform for DesktopPlatform {
    type Color = BinaryColor;
    type CoreManager = DummyCoreManager;

    fn init(&mut self, _: &Flags) {}

    fn update_toolbar(&mut self, buffer: &DrawBuffer<Self::Color>) {
        self.window_title.update(buffer);
    }
    fn update_main(&mut self, buffer: &DrawBuffer<Self::Color>) {
        self.window_main.update(buffer);
    }

    fn toolbar_dimensions(&self) -> Size {
        sizes::TITLE
    }
    fn main_dimensions(&self) -> Size {
        sizes::MAIN
    }

    fn events(&mut self) -> Vec<Event> {
        self.platform.events()
    }

    fn sdl(&mut self) -> &mut SdlPlatform<Self::Color> {
        &mut self.platform
    }

    fn start_loop(&mut self) {}

    fn end_loop(&mut self) {}

    fn core_manager_mut(&mut self) -> &mut Self::CoreManager {
        &mut self.core_manager
    }
}

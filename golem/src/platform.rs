//! The platform module provides a platform-agnostic interface for interacting with the host
//! platform. This includes things like the window manager, keyboard, mouse, etc.
//!
//! The platform module is designed to be used by the `Application` struct, and should not be used
//! directly by the user. An Application will define the common logic for the MiSTer application
//! itself, and the Platform defined here will be responsible to send it to the host platform,
//! which is either Desktop, DE10Nano, null (for testing) or others.
//!
//! Platforms are responsible for mocking the FPGA logic, graphics and initializing SDL.
use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::platform::sdl::SdlPlatform;
use crate::main_inner::Flags;
use cfg_if::cfg_if;
use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::{BinaryColor, PixelColor};
use image::DynamicImage;
use mister_fpga::config_string::{ConfigMenu, LoadFileInfo};
use mister_fpga::types::StatusBitMap;
use sdl3::event::Event;
use sdl3::gamepad::{Axis, Button};
use sdl3::keyboard::Scancode;
use std::path::Path;

cfg_if! {
    if #[cfg(test)] {
        mod null;
        pub use null::NullPlatform as PlatformWindowManager;
        pub use null::NullCore as CoreType;
    } else if #[cfg(any(
        all(feature = "platform_de10", feature = "platform_desktop"),
        all(feature = "platform_de10", test),
        all(feature = "platform_desktop", test),
    ))] {
        compile_error!("Only one platform can be enabled at a time.");
    } else if #[cfg(feature = "platform_desktop")] {
        mod desktop;
        pub use desktop::DesktopPlatform as PlatformWindowManager;
        pub use desktop::DummyCore as CoreType;
    } else if #[cfg(feature = "platform_de10")] {
        pub mod de10;
        pub use de10::De10Platform as PlatformWindowManager;
        pub use de10::core_manager::core::MisterFpgaCore as CoreType;
    } else {
        compile_error!("At least one platform must be enabled.");
    }
}

// In tests, this is unused as there are no OSD.
#[allow(unused)]
mod sizes {
    use embedded_graphics::geometry::Size;

    /// The size of the title bar.
    pub const TITLE: Size = Size::new(256, 15);

    /// The size of the main OSD display. We never resize it to be smaller,
    /// instead reusing the size for other information.
    pub const MAIN: Size = Size::new(256, 16 * 8);
}

pub trait SaveState {
    fn is_dirty(&self) -> bool;
    fn write_to(&mut self, writer: impl std::io::Write) -> Result<(), String>;
    fn read_from(&mut self, reader: impl std::io::Read) -> Result<(), String>;
}

pub trait Core {
    type SaveState: SaveState;

    fn name(&self) -> &str;

    fn current_game(&self) -> Option<&Path> {
        None
    }

    /// Send a file to the core. The file_info is implementation specific.
    fn load_file(&mut self, path: &Path, file_info: Option<LoadFileInfo>) -> Result<(), String>;

    fn version(&self) -> Option<&str>;

    fn menu_options(&self) -> &[ConfigMenu];

    /// Trigger a menu by its option. This will return `true` if the core can and
    /// did execute the command successfully. `false` will be returned if the core
    /// cannot execute the command (this is not an error).
    fn trigger_menu(&mut self, menu: &ConfigMenu) -> Result<bool, String>;

    fn status_mask(&self) -> StatusBitMap;
    fn status_bits(&self) -> StatusBitMap;
    fn status_pulse(&mut self, bit: usize) {
        let mut bits = self.status_bits();
        bits.set(bit, true);
        self.set_status_bits(bits);

        bits.set(bit, false);
        self.set_status_bits(bits);
    }
    fn set_status_bits(&mut self, bits: StatusBitMap);

    fn take_screenshot(&mut self) -> Result<DynamicImage, String>;

    fn key_down(&mut self, key: Scancode);
    fn key_up(&mut self, key: Scancode);

    fn sdl_button_down(&mut self, controller: u8, button: Button);

    fn sdl_button_up(&mut self, controller: u8, button: Button);

    fn sdl_axis_motion(&mut self, controller: u8, axis: Axis, value: i16);

    fn save_states(&mut self) -> Option<&mut [Self::SaveState]>;
}

pub trait CoreManager {
    type Core: Core;

    /// Load a core into the FPGA.
    // TODO: Change the error type to something more usable than string.
    fn load_core(&mut self, path: impl AsRef<Path>) -> Result<Self::Core, String>;

    fn get_current_core(&mut self) -> Result<Self::Core, String>;

    fn load_game(
        &mut self,
        core_path: impl AsRef<Path>,
        game_path: impl AsRef<Path>,
    ) -> Result<Self::Core, String> {
        let mut core = self.load_core(core_path)?;
        core.load_file(game_path.as_ref(), None)?;
        Ok(core)
    }

    /// Load the main menu core.
    fn load_menu(&mut self) -> Result<Self::Core, String>;

    /// Show the menu (OSD).
    fn show_menu(&mut self);
    /// Hide the menu (OSD).
    fn hide_menu(&mut self);
}

pub trait GoLEmPlatform {
    type Color: PixelColor;
    type CoreManager: CoreManager;

    fn init(&mut self, flags: &Flags);

    fn update_toolbar(&mut self, buffer: &DrawBuffer<Self::Color>);
    fn update_main(&mut self, buffer: &DrawBuffer<Self::Color>);

    fn toolbar_dimensions(&self) -> Size;
    fn main_dimensions(&self) -> Size;

    fn events(&mut self) -> Vec<Event>;

    fn sdl(&mut self) -> &mut SdlPlatform<Self::Color>;

    fn start_loop(&mut self);
    fn end_loop(&mut self);

    fn core_manager_mut(&mut self) -> &mut Self::CoreManager;
}

/// The [WindowManager] structure is responsible for managing and holding the state
/// of the application itself. It takes the main loop, and with every iteration will
/// poll the user input, update the display, and send it to the screen (either in the
/// simulator or on the device's display).
///
/// Because of the differences between SDL/Desktop and the MiSTer itself, we need some
/// abstraction over where and how things are displayed and how inputs are taken.
///
/// Everything that's not related to taking inputs and displaying buffers is handled
/// by the MisterApplication itself.
#[derive(Default)]
pub struct WindowManager {
    inner: PlatformWindowManager,
}

impl WindowManager {}

impl GoLEmPlatform for WindowManager {
    type Color = <PlatformWindowManager as GoLEmPlatform>::Color;
    type CoreManager = <PlatformWindowManager as GoLEmPlatform>::CoreManager;

    fn init(&mut self, flags: &Flags) {
        self.inner.init(flags);
    }

    fn update_toolbar(&mut self, buffer: &DrawBuffer<BinaryColor>) {
        self.inner.update_toolbar(buffer);
    }
    fn update_main(&mut self, buffer: &DrawBuffer<BinaryColor>) {
        self.inner.update_main(buffer);
    }

    fn toolbar_dimensions(&self) -> Size {
        self.inner.toolbar_dimensions()
    }
    fn main_dimensions(&self) -> Size {
        self.inner.main_dimensions()
    }

    fn events(&mut self) -> Vec<Event> {
        self.inner.events()
    }

    fn sdl(&mut self) -> &mut SdlPlatform<Self::Color> {
        self.inner.sdl()
    }

    fn start_loop(&mut self) {
        self.inner.start_loop();
    }

    fn end_loop(&mut self) {
        self.inner.end_loop();
    }

    fn core_manager_mut(&mut self) -> &mut Self::CoreManager {
        self.inner.core_manager_mut()
    }
}

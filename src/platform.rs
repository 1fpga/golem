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
use embedded_graphics::geometry::{OriginDimensions, Size};
use embedded_graphics::pixelcolor::{BinaryColor, PixelColor};
use mister_fpga::config_string::{ConfigMenu, LoadFileInfo};
use mister_fpga::types::StatusBitMap;
use sdl3::event::Event;
use std::path::Path;
use tracing::trace;

cfg_if! {
    if #[cfg(test)] {
        mod null;
        pub use null::NullPlatform as PlatformWindowManager;
    } else if #[cfg(any(
        all(feature = "platform_de10", feature = "platform_desktop"),
        all(feature = "platform_de10", test),
        all(feature = "platform_desktop", test),
    ))] {
        compile_error!("Only one platform can be enabled at a time.");
    } else if #[cfg(feature = "platform_desktop")] {
        mod desktop;
        pub use desktop::DesktopPlatform as PlatformWindowManager;
    } else if #[cfg(feature = "platform_de10")] {
        pub mod de10;
        pub use de10::De10Platform as PlatformWindowManager;
    } else {
        compile_error!("At least one platform must be enabled.");
    }
}

mod sizes {
    use embedded_graphics::geometry::Size;

    /// The size of the title bar.
    pub const TITLE: Size = Size::new(256, 15);

    /// The size of the main OSD display. We never resize it to be smaller,
    /// instead reusing the size for other information.
    pub const MAIN: Size = Size::new(256, 16 * 8);
}

#[derive(Default)]
pub struct PlatformState {
    events: Vec<Event>,
    should_quit: bool,
    target_size: Size,
}

impl PlatformState {
    pub fn new(target_size: Size, events: Vec<Event>) -> Self {
        Self {
            target_size,
            events,
            ..Default::default()
        }
    }

    pub fn new_frame(&mut self) {
        self.should_quit = false;
    }

    pub fn should_quit(&mut self) -> bool {
        self.should_quit
    }

    pub fn events(&self) -> impl Iterator<Item = &'_ Event> + '_ {
        self.events.iter().map(|ev| {
            trace!("Event: {:?}", ev);
            ev
        })
    }
}

impl OriginDimensions for PlatformState {
    fn size(&self) -> Size {
        self.target_size
    }
}

pub trait Core {
    fn name(&self) -> &str;

    /// Send a file to the core. The file_info is implementation specific.
    fn load_file(&mut self, path: &Path, file_info: Option<LoadFileInfo>) -> Result<(), String>;

    fn version(&self) -> Option<&str>;

    fn menu_options(&self) -> &[ConfigMenu];

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

    fn send_key(&mut self, key: u8);

    fn sdl_joy_button_down(&mut self, joystick_idx: u8, button: u8);

    fn sdl_joy_button_up(&mut self, joystick_idx: u8, button: u8);
}

pub trait CoreManager {
    type Core: Core;

    /// Load a core into the FPGA.
    // TODO: Change the error type to something more usable than string.
    fn load_program(&mut self, path: impl AsRef<Path>) -> Result<Self::Core, String>;

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

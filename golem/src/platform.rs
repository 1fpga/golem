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
use crate::Flags;
use cfg_if::cfg_if;
use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::{BinaryColor, PixelColor};
use golem_core::CoreManager;
use sdl3::event::Event;

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

pub trait GoLEmPlatform {
    type Color: PixelColor;

    fn init(&mut self, flags: &Flags);

    fn update_toolbar(&mut self, buffer: &DrawBuffer<Self::Color>);
    fn update_main(&mut self, buffer: &DrawBuffer<Self::Color>);

    fn toolbar_dimensions(&self) -> Size;
    fn main_dimensions(&self) -> Size;

    fn events(&mut self) -> Vec<Event>;

    fn sdl(&mut self) -> &mut SdlPlatform<Self::Color>;

    fn start_loop(&mut self);
    fn end_loop(&mut self);

    fn core_manager_mut(&mut self) -> &mut CoreManager;
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

    fn core_manager_mut(&mut self) -> &mut CoreManager {
        self.inner.core_manager_mut()
    }
}

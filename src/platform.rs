//! The platform module provides a platform-agnostic interface for interacting with the host
//! platform. This includes things like the window manager, keyboard, mouse, etc.
//!
//! The platform module is designed to be used by the `Application` struct, and should not be used
//! directly by the user. An Application will define the common logic for the MiSTer application
//! itself, and the Platform defined here will be responsible to send it to the host platform,
//! which is either Desktop, DE10Nano, null (for testing) or others.
//!
//! Platforms are responsible for mocking the FPGA logic, graphics and initializing SDL.
use crate::macguiver::application::Application;
use crate::main_inner::Flags;
use cfg_if::cfg_if;
use embedded_graphics::geometry::{OriginDimensions, Size};
use embedded_graphics::pixelcolor::{BinaryColor, PixelColor};
use sdl3::event::Event;
use tracing::trace;

cfg_if! {
    if #[cfg(any(
        all(feature = "platform_de10", feature = "platform_desktop"),
        all(feature = "platform_de10", test),
        all(feature = "platform_desktop", test),
    ))] {
        compile_error!("Only one platform can be enabled at a time.");
    } else if #[cfg(feature = "platform_desktop")] {
        mod desktop;
        pub use desktop::PlatformWindowManager;
    } else if #[cfg(feature = "platform_de10")] {
        mod de10;
        pub use de10::PlatformWindowManager;
    } else if #[cfg(test)] {
        mod null;
        pub use null::PlatformWindowManager;
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

trait PlatformInner {
    type Color: PixelColor;

    fn run(&mut self, application: &mut impl Application<Color = BinaryColor>, flags: Flags);
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

impl WindowManager {
    pub fn run(
        &mut self,
        application: &mut impl Application<Color = BinaryColor>,
        flags: Flags,
    ) -> Result<(), String> {
        self.inner.run(application, flags);
        Ok(())
    }
}

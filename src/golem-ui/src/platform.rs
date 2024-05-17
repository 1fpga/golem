//! The platform module provides a platform-agnostic interface for interacting with the host
//! platform. This includes things like the window manager, keyboard, mouse, etc.
//!
//! The platform module is designed to be used by the `Application` struct, and should not be used
//! directly by the user. An Application will define the common logic for the MiSTer application
//! itself, and the Platform defined here will be responsible to send it to the host platform,
//! which is either Desktop, DE10Nano, null (for testing) or others.
//!
//! Platforms are responsible for mocking the FPGA logic, graphics and initializing SDL.

pub use crate::core_manager::CoreManager;

pub mod de10;

// In tests, this is unused as there are no OSD.
#[cfg_attr(test, allow(unused))]
mod sizes {
    use embedded_graphics::geometry::Size;

    /// The size of the title bar.
    pub const TITLE: Size = Size::new(256, 15);

    /// The size of the main OSD display. We never resize it to be smaller,
    /// instead reusing the size for other information.
    pub const MAIN: Size = Size::new(256, 16 * 8);
}

pub use de10::De10Platform as WindowManager;

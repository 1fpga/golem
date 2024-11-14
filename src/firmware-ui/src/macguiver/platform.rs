//! Platform abstraction for MacGUIver applications.

use crate::macguiver::buffer::DrawBuffer;
use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::PixelColor;

pub mod sdl;

pub trait PlatformWindow {
    type Color: PixelColor;

    /// Blit the buffer to the window.
    fn update(&mut self, framebuffer: &DrawBuffer<Self::Color>);
}

pub trait Platform {
    type InitState: Send;
    type Window: PlatformWindow;
    type State;
    type Event;

    /// Initialize the platform.
    fn init(state: Self::InitState) -> Self;

    /// Create a new window.
    fn window(&mut self, title: &str, size: Size) -> Self::Window;

    /// Start an event loop. These can be nested. Every loop is an update event.
    /// It includes a list of current events.
    /// This is different from an application loop which also manages the
    /// application state, including rendering the display etc.
    fn event_loop(&mut self, loop_fn: impl FnMut(&mut Self, &Self::State) -> bool);
}

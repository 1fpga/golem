//! Platform abstraction for MacGUIver applications.

use crate::macguiver::buffer::DrawBuffer;
use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::PixelColor;

pub mod sdl;

pub trait Application {
    type Color: PixelColor;
    type State;

    fn init(state: Self::State, platform: &impl Platform) -> Self;
    fn redraw(&self);
    fn update(&mut self);
}

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
    fn event_loop(&mut self, update: impl FnMut(&Self::State) -> bool);
}

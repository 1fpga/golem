use crate::macguiver::application::Application;
use crate::macguiver::buffer::DrawBuffer;
use crate::platform::{PlatformInner, PlatformState};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::DrawTarget;
use embedded_graphics::Drawable;
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use std::convert::TryInto;
pub use DesktopWindowManager as PlatformWindowManager;

pub mod keyboard;

mod sizes {
    use embedded_graphics::geometry::Size;

    /// The size of the title bar.
    pub const TITLE: Size = Size::new(256, 15);

    /// The size of the main OSD display. We never resize it to be smaller,
    /// instead reusing the size for other information.
    pub const MAIN: Size = Size::new(256, 16 * 8);
}

pub struct DesktopWindowManager {
    pub osd: DrawBuffer<BinaryColor>,
    pub title: DrawBuffer<BinaryColor>,

    window_osd: Window,
    window_title: Window,
}

impl Default for DesktopWindowManager {
    fn default() -> Self {
        let title = DrawBuffer::new(sizes::TITLE);
        let osd = DrawBuffer::new(sizes::MAIN);

        let window_title = Window::new(
            "Title",
            &OutputSettingsBuilder::new()
                .max_fps(10000)
                .theme(BinaryColorTheme::OledBlue)
                .build(),
        );

        let window_osd = Window::new(
            "OSD",
            &OutputSettingsBuilder::new()
                .max_fps(10000)
                .theme(BinaryColorTheme::OledBlue)
                .build(),
        );

        Self {
            osd,
            title,
            window_osd,
            window_title,
        }
    }
}

impl PlatformInner for DesktopWindowManager {
    type Color = BinaryColor;

    fn run(&mut self, app: &mut impl Application<Color = BinaryColor>) {
        let mut state: PlatformState = Default::default();

        'main: loop {
            app.update(&state);

            // Clear the buffers.
            self.osd.clear(BinaryColor::Off).unwrap();
            self.title.clear(BinaryColor::Off).unwrap();

            app.draw(&mut self.osd);
            app.draw_title(&mut self.title);

            let mut display = SimulatorDisplay::new(self.osd.size());
            self.osd.draw(&mut display).unwrap();
            self.window_osd.update(&display);

            let mut display = SimulatorDisplay::new(self.title.size());
            self.title.draw(&mut display).unwrap();
            self.window_title.update(&display);

            for ev in self.window_osd.events().chain(self.window_title.events()) {
                match ev {
                    SimulatorEvent::KeyUp { keycode, .. } => {
                        if let Ok(kc) = keycode.try_into() {
                            state.keys.up(kc);
                        }
                    }
                    SimulatorEvent::KeyDown { keycode, .. } => {
                        if let Ok(kc) = keycode.try_into() {
                            state.keys.down(kc);
                        }
                    }

                    SimulatorEvent::MouseButtonUp { .. } => {}
                    SimulatorEvent::MouseButtonDown { .. } => {}
                    SimulatorEvent::MouseWheel { .. } => {}
                    SimulatorEvent::MouseMove { .. } => {}
                    SimulatorEvent::Quit => break 'main,
                }
            }
        }
    }
}

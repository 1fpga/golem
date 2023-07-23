use crate::macgyver::buffer::DrawBuffer;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::Drawable;
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

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
        let mut title = DrawBuffer::new(sizes::TITLE);
        let mut osd = DrawBuffer::new(sizes::MAIN);

        let window_title = Window::new(
            "Title",
            &OutputSettingsBuilder::new()
                .max_fps(1000)
                .theme(BinaryColorTheme::OledBlue)
                .build(),
        );

        let window_osd = Window::new(
            "OSD",
            &OutputSettingsBuilder::new()
                .max_fps(1000)
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

impl DesktopWindowManager {
    pub fn run(&mut self, app: &mut impl Application) {
        'main: loop {
            let _ = app.update();
            app.draw(&mut self.osd);

            let mut display = SimulatorDisplay::new(self.osd.size());
            self.osd.draw(&mut display).unwrap();
            self.window_osd.update(&display);

            let mut display = SimulatorDisplay::new(self.title.size());
            self.title.draw(&mut display).unwrap();
            self.window_title.update(&display);

            if self.window_osd.events().any(|e| e == SimulatorEvent::Quit) {
                break 'main;
            }
            if self
                .window_title
                .events()
                .any(|e| e == SimulatorEvent::Quit)
            {
                break 'main;
            }
        }
    }
}

use crate::application::{Application, Command};
pub use DesktopWindowManager as PlatformWindowManager;

use crate::macguiver::application::Application;
use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::platform::sdl::{
    BinaryColorTheme, OutputSettingsBuilder, SdlInitState, SdlPlatform, Window,
};
use crate::macguiver::platform::{Platform, PlatformWindow};
use crate::main_inner::Flags;
use crate::platform::{PlatformInner, PlatformState};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::DrawTarget;
use embedded_graphics::Drawable;
pub use DesktopWindowManager as PlatformWindowManager;

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

    platform: SdlPlatform<BinaryColor>,

    window_osd: Window<BinaryColor>,
    window_title: Window<BinaryColor>,
}

impl Default for DesktopWindowManager {
    fn default() -> Self {
        let title = DrawBuffer::new(sizes::TITLE);
        let osd = DrawBuffer::new(sizes::MAIN);

        let mut platform = SdlPlatform::init(SdlInitState::new(
            OutputSettingsBuilder::new()
                .scale(3)
                .theme(BinaryColorTheme::LcdBlue)
                .build(),
        ));

        let window_title = platform.window("Title", sizes::TITLE);
        let window_osd = platform.window("OSD", sizes::MAIN);

        Self {
            osd,
            title,
            platform,
            window_osd,
            window_title,
        }
    }
}

impl PlatformInner for DesktopWindowManager {
    type Color = BinaryColor;

    fn run(&mut self, app: &mut impl Application<Color = BinaryColor>, _flags: Flags) {
        let mut platform_state: PlatformState = PlatformState::default();
        let osd = &mut self.osd;
        let title = &mut self.title;
        let window_osd = &mut self.window_osd;
        let window_title = &mut self.window_title;

        self.platform.event_loop(|state| {
            app.update(&platform_state);

            // Clear the buffers.
            osd.clear(BinaryColor::Off).unwrap();
            title.clear(BinaryColor::Off).unwrap();

            app.draw(osd);
            app.draw_title(title);
            title.invert();

            let mut display = DrawBuffer::new(osd.size());
            osd.draw(&mut display).unwrap();
            window_osd.update(&display);

            let mut display = DrawBuffer::new(title.size());
            title.draw(&mut display).unwrap();
            window_title.update(&display);

            let mut should_return = false;
            state.events(|ev| match ev {
                sdl3::event::Event::Quit { .. } => should_return = true,
                sdl3::event::Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    platform_state.keys.down(keycode.into());
                }
                sdl3::event::Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    platform_state.keys.up(keycode.into());
                }
                _ => {}
            });

            should_return
        });
    }
}

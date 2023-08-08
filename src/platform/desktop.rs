use crate::macguiver::application::{Application, UpdateResult};
use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::platform::sdl::{
    BinaryColorTheme, OutputSettingsBuilder, SdlInitState, SdlPlatform,
};
use crate::macguiver::platform::{Platform, PlatformWindow};
use crate::main_inner::Flags;
use crate::platform::{sizes, PlatformInner, PlatformState};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::DrawTarget;
pub use DesktopWindowManager as PlatformWindowManager;

pub struct DesktopWindowManager {
    platform: SdlPlatform<BinaryColor>,
}

impl Default for DesktopWindowManager {
    fn default() -> Self {
        let platform = SdlPlatform::init(SdlInitState::new(
            OutputSettingsBuilder::new()
                .scale(3)
                .theme(BinaryColorTheme::LcdBlue)
                .build(),
        ));

        Self { platform }
    }
}

impl PlatformInner for DesktopWindowManager {
    type Color = BinaryColor;

    fn run(&mut self, app: &mut impl Application<Color = BinaryColor>, _flags: Flags) {
        let mut window_title = self.platform.window("Title", sizes::TITLE);
        let mut window_osd = self.platform.window("Title", sizes::MAIN);
        let mut title_buffer = DrawBuffer::new(sizes::TITLE);
        let mut osd = DrawBuffer::new(sizes::MAIN);

        self.platform.event_loop(|state| {
            let mut platform_state: PlatformState =
                PlatformState::new(sizes::MAIN, state.events().collect());

            match app.update(&platform_state) {
                UpdateResult::Redraw(title, main) => {
                    if title {
                        title_buffer.clear(BinaryColor::Off).unwrap();
                        app.draw_title(&mut title_buffer);
                        title_buffer.invert();
                        window_title.update(&title_buffer);
                    }
                    if main {
                        osd.clear(BinaryColor::Off).unwrap();
                        app.draw_main(&mut osd);
                        window_osd.update(&osd);
                    }
                }
                UpdateResult::NoRedraw => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                UpdateResult::Quit => return true,
            }

            platform_state.should_quit()
        });
    }
}

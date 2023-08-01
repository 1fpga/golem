use crate::ffi::fpga;
use crate::macguiver::application::Application;
use crate::macguiver::platform::sdl::{
    BinaryColorTheme, OutputSettingsBuilder, SdlInitState, SdlPlatform, Window,
};
use crate::macguiver::platform::{Platform, PlatformWindow};
use crate::platform::{PlatformInner, PlatformState};
use crate::{menu, osd, spi};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;

mod buffer;

use libc::{O_RDONLY, O_RDWR, O_WRONLY};
use std::fs::{File, OpenOptions};
use std::os::unix::{fs::OpenOptionsExt, io::OwnedFd};
use std::path::Path;

const SDL_VIDEO_DRIVER_VARNAME: &str = "SDL_VIDEO_DRIVER";
const SDL_VIDEO_DRIVER_DEFAULT: &str = "evdev";

struct Interface;

pub struct De10Platform {
    pub osd: buffer::OsdDisplayView,
    pub title: buffer::OsdDisplayView,

    pub platform: SdlPlatform<BinaryColor>,
}

impl Default for De10Platform {
    fn default() -> Self {
        if std::env::var_os(SDL_VIDEO_DRIVER_VARNAME).is_none() {
            std::env::set_var(SDL_VIDEO_DRIVER_VARNAME, SDL_VIDEO_DRIVER_DEFAULT);
        }

        let osd = buffer::OsdDisplayView::main();
        let title = buffer::OsdDisplayView::title();

        let platform = SdlPlatform::init(SdlInitState::default());

        Self {
            osd,
            title,
            platform,
        }
    }
}

impl PlatformInner for De10Platform {
    type Color = BinaryColor;

    fn run(
        &mut self,
        app: &mut impl Application<Color = Self::Color>,
        flags: crate::main_inner::Flags,
    ) {
        let mut platform_state: PlatformState = PlatformState::default();
        let osd = &mut self.osd;
        let title = &mut self.title;

        osd::OsdSetSize(19);
        unsafe {
            while fpga::is_fpga_ready(1) == 0 {
                fpga::fpga_wait_to_reset();
            }
        }

        self.platform.event_loop(|state| unsafe {
            crate::user_io::user_io_poll();
            crate::input::input_poll(0);
            menu::HandleUI();

            app.update(&platform_state);

            // Clear the buffers.
            osd.clear(BinaryColor::Off).unwrap();
            title.clear(BinaryColor::Off).unwrap();

            app.draw(&mut osd.inner);
            app.draw_title(&mut title.inner);
            title.inner.invert();

            // Send everything to the scaler.
            for line in osd.line_iter().chain(title.line_iter()) {
                let line_buffer = osd.get_binary_line_array(line);
                spi::spi_osd_cmd_cont(osd::OSD_CMD_WRITE | (line as u8));
                spi::spi_write(line_buffer.as_ptr(), 256, 0);
                spi::DisableOsd();
            }

            let mut should_return = false;
            state.events(|ev| {
                eprintln!("sdl: {ev:?}");
                match ev {
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
                }
            });

            should_return
        });
    }
}

pub use De10Platform as PlatformWindowManager;

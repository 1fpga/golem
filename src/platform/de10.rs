use crate::ffi::fpga;
use crate::macguiver::application::Application;
use crate::macguiver::platform::sdl::{SdlInitState, SdlPlatform};
use crate::macguiver::platform::Platform;
use crate::platform::{PlatformInner, PlatformState};
use crate::{menu, osd, spi};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;
use tracing::{debug, error, info};

mod buffer;

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

        let mut fpga = unsafe {
            fpga::Fpga::init().unwrap_or_else(|| {
                debug!("GPI[31] == 1");
                error!("FPGA is uninitialized or incompatible core loaded.");
                error!("Quitting. Bye bye...\n");
                std::process::exit(1);
            })
        };

        unsafe {
            fpga.wait_for_ready();

            info!("Core type: {:?}", fpga.core_type());
            info!("Core interface: {:?}", fpga.core_interface_type());
            info!("Core version: {:?}", fpga.core_io_version());

            osd::OsdEnable(0);
            crate::file_io::FindStorage();
            let (core, xml) = (
                std::ffi::CString::new(flags.core).unwrap(),
                flags.xml.map(|str| std::ffi::CString::new(str).unwrap()),
            );

            crate::user_io::user_io_init(
                core.as_bytes_with_nul().as_ptr(),
                xml.map(|str| str.as_bytes_with_nul().as_ptr())
                    .unwrap_or(std::ptr::null()),
            );

            osd::OsdSetSize(19);
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
            for (line, buffer) in title
                .line_iter()
                .map(|line| (line, &title))
                .chain(osd.line_iter().map(|line| (line, &osd)))
            {
                let line_buffer = buffer.get_binary_line_array(line);
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

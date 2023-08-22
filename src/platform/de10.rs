use crate::ffi::fpga;
use crate::macguiver::platform::sdl::{SdlInitState, SdlPlatform};
use crate::macguiver::platform::Platform;
use crate::main_inner::Flags;
use crate::osd;
use crate::platform::de10::buffer::OsdDisplayView;
use crate::platform::{sizes, MiSTerPlatform};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::Drawable;
use sdl3::event::Event;
use tracing::{debug, error, info};

mod buffer;

const SDL_VIDEO_DRIVER_VARNAME: &str = "SDL_VIDEO_DRIVER";
const SDL_VIDEO_DRIVER_DEFAULT: &str = "evdev";

pub struct De10Platform {
    pub platform: SdlPlatform<BinaryColor>,
    title_display: OsdDisplayView,
    main_display: OsdDisplayView,
}

impl Default for De10Platform {
    fn default() -> Self {
        if std::env::var_os(SDL_VIDEO_DRIVER_VARNAME).is_none() {
            std::env::set_var(SDL_VIDEO_DRIVER_VARNAME, SDL_VIDEO_DRIVER_DEFAULT);
        }

        let platform = SdlPlatform::init(SdlInitState::default());

        let title_display = OsdDisplayView::title();
        let main_display = OsdDisplayView::main();

        Self {
            platform,
            title_display,
            main_display,
        }
    }
}

// impl De10Platform {
//     pub fn event_loop<DrawFn>(&mut self, update: impl Fn(&mut PlatformState) -> bool, draw: DrawFn)
//     where
//         Target: DrawTarget<BinaryColor>,
//         DrawFn: FnMut(&mut Target),
//     {
//         let mut platform_state: PlatformState = PlatformState::default();
//         let osd = &mut self.osd;
//         let title = &mut self.title;
//
//         self.platform.event_loop(|state| unsafe {
//             crate::user_io::user_io_poll();
//             crate::input::input_poll(0);
//             menu::HandleUI();
//
//             // Clear the buffers.
//             osd.clear(BinaryColor::Off).unwrap();
//             title.clear(BinaryColor::Off).unwrap();
//
//             let mut should_return = handler(&platform_state, osd);
//
//             // Send everything to the scaler.
//             for (line, buffer) in title
//                 .line_iter()
//                 .map(|line| (line, &title))
//                 .chain(osd.line_iter().map(|line| (line, &osd)))
//             {
//                 let line_buffer = buffer.get_binary_line_array(line);
//                 spi::spi_osd_cmd_cont(osd::OSD_CMD_WRITE | (line as u8));
//                 spi::spi_write(line_buffer.as_ptr(), 256, 0);
//                 spi::DisableOsd();
//             }
//
//             state.events(|ev| {
//                 eprintln!("sdl: {ev:?}");
//                 match ev {
//                     sdl3::event::Event::Quit { .. } => should_return = true,
//                     sdl3::event::Event::KeyDown {
//                         keycode: Some(keycode),
//                         ..
//                     } => {
//                         platform_state.keys.down(keycode.into());
//                     }
//                     sdl3::event::Event::KeyUp {
//                         keycode: Some(keycode),
//                         ..
//                     } => {
//                         platform_state.keys.up(keycode.into());
//                     }
//                     _ => {}
//                 }
//             });
//
//             // Sleep a little.
//             std::thread::sleep(std::time::Duration::from_millis(10));
//
//             should_return
//         });
//     }
// }

impl MiSTerPlatform for De10Platform {
    type Color = BinaryColor;

    fn init(&mut self, flags: &Flags) {
        let mut fpga = fpga::Fpga::init().unwrap_or_else(|| {
            debug!("GPI[31] == 1");
            error!("FPGA is uninitialized or incompatible core loaded.");
            error!("Quitting. Bye bye...\n");
            std::process::exit(1);
        });

        unsafe {
            fpga.wait_for_ready();

            info!("Core type: {:?}", fpga.core_type());
            info!("Core interface: {:?}", fpga.core_interface_type());
            info!("Core version: {:?}", fpga.core_io_version());

            osd::OsdEnable(0);
            crate::file_io::FindStorage();
            let (core, xml) = (
                std::ffi::CString::new(flags.core.clone()).unwrap(),
                flags
                    .xml
                    .clone()
                    .map(|str| std::ffi::CString::new(str).unwrap()),
            );

            crate::user_io::user_io_init(
                core.as_bytes_with_nul().as_ptr(),
                xml.map(|str| str.as_bytes_with_nul().as_ptr())
                    .unwrap_or(std::ptr::null()),
            );

            osd::OsdSetSize(19);
        }
    }

    fn update_toolbar(&mut self, buffer: &DrawBuffer<Self::Color>) {
        buffer.draw(&mut self.title_display.inner).unwrap();
        self.title_display.inner.invert();
        self.title_display.send();
    }
    fn update_main(&mut self, buffer: &DrawBuffer<Self::Color>) {
        self.main_display.inner.clear(BinaryColor::Off).unwrap();
        buffer.draw(&mut self.main_display.inner).unwrap();
        self.main_display.send();
    }

    fn toolbar_dimensions(&self) -> Size {
        sizes::TITLE
    }
    fn main_dimensions(&self) -> Size {
        sizes::MAIN
    }

    fn events(&mut self) -> Vec<Event> {
        self.platform.events()
    }
    //
    // fn run(&mut self, app: &mut impl Application<Color = BinaryColor>, flags: Flags) {
    //     self.platform.event_loop(|state| {
    //         let mut title_buffer = &mut title_display.inner;
    //         let mut osd_buffer = &mut osd_display.inner;
    //
    //         match app.update(&platform_state) {
    //             UpdateResult::Redraw(title, main) => {
    //                 if title {
    //                     title_buffer.clear(BinaryColor::Off).unwrap();
    //                     app.draw_title(&mut title_buffer);
    //                     title_buffer.invert();
    //                     title_display.send();
    //                 }
    //                 if main {
    //                     osd_buffer.clear(BinaryColor::Off).unwrap();
    //                     app.draw_main(&mut osd_buffer);
    //                     osd_display.send();
    //                 }
    //             }
    //             UpdateResult::NoRedraw => {
    //                 std::thread::sleep(std::time::Duration::from_millis(10));
    //             }
    //             UpdateResult::Quit => return true,
    //         }
    //
    //         platform_state.should_quit()
    //     });
    // }
}

use crate::macguiver::buffer::DrawBuffer;
pub use De10Platform as PlatformWindowManager;

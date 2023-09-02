#![cfg(feature = "platform_de10")]
use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::platform::sdl::{SdlInitState, SdlPlatform, Window};
use crate::macguiver::platform::Platform;
use crate::main_inner::Flags;
use crate::platform::de10::buffer::OsdDisplay;
use crate::platform::{sizes, CoreManager, MiSTerPlatform};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{OriginDimensions, Size};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::Drawable;
use sdl3::event::Event;
use tracing::{debug, error, info};

mod battery;
mod buffer;
mod core_manager;
mod fpga;
mod input;
mod menu;
mod osd;
mod shmem;
mod smbus;
pub mod spi;
mod support;
pub mod user_io;

const SDL_VIDEO_DRIVER_VARNAME: &str = "SDL_VIDEO_DRIVER";
const SDL_VIDEO_DRIVER_DEFAULT: &str = "evdev";

pub struct De10Platform {
    pub platform: SdlPlatform<BinaryColor>,
    title_display: OsdDisplay,
    main_display: OsdDisplay,
    _window: Window<BinaryColor>,

    toolbar_buffer: DrawBuffer<BinaryColor>,

    core_manager: core_manager::CoreManager,
}

impl Default for De10Platform {
    fn default() -> Self {
        if std::env::var_os(SDL_VIDEO_DRIVER_VARNAME).is_none() {
            std::env::set_var(SDL_VIDEO_DRIVER_VARNAME, SDL_VIDEO_DRIVER_DEFAULT);
        }

        let mut platform = SdlPlatform::init(SdlInitState::default());

        let title_display = OsdDisplay::title();
        let main_display = OsdDisplay::main();

        let toolbar_buffer = DrawBuffer::new(title_display.size());

        // Need at least 1 window to get events.
        let window = platform.window("Title", Size::new(1, 1));

        let fpga = fpga::Fpga::init().unwrap();
        if !fpga.is_ready() {
            debug!("GPI[31] == 1");
            error!("FPGA is uninitialized or incompatible core loaded.");
            error!("Quitting. Bye bye...\n");
            std::process::exit(1);
        }

        let core_manager = core_manager::CoreManager::new(fpga);

        Self {
            platform,
            title_display,
            main_display,
            _window: window,
            toolbar_buffer,
            core_manager,
        }
    }
}

impl MiSTerPlatform for De10Platform {
    type Color = BinaryColor;
    type CoreManager = core_manager::CoreManager;

    fn init(&mut self, flags: &Flags) {
        unsafe {
            self.core_manager.fpga_mut().wait_for_ready();

            if flags.core.is_empty() {
                self.core_manager
                    .load_program("/media/fat/menu.rbf")
                    .unwrap();
            }

            let fpga = self.core_manager.fpga_mut();
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

            user_io::user_io_init(
                core.as_bytes_with_nul().as_ptr() as *const _,
                xml.map(|str| str.as_bytes_with_nul().as_ptr() as *const _)
                    .unwrap_or(std::ptr::null()),
            );

            osd::OsdSetSize(19);
        }
    }

    fn update_toolbar(&mut self, buffer: &DrawBuffer<Self::Color>) {
        self.toolbar_buffer.clear(BinaryColor::Off).unwrap();
        buffer.draw(&mut self.toolbar_buffer).unwrap();
        self.title_display.send(&self.toolbar_buffer);
    }
    fn update_main(&mut self, buffer: &DrawBuffer<Self::Color>) {
        self.main_display.send(buffer);
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

    fn end_loop(&mut self) {
        unsafe {
            user_io::user_io_poll();
            input::input_poll(0);
            menu::HandleUI();
        }
    }

    fn core_manager_mut(&mut self) -> &mut Self::CoreManager {
        &mut self.core_manager
    }
}

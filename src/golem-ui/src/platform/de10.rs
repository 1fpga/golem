use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::{BinaryColor, Rgb888};
use image::RgbImage;
use mister_fpga::core::AsMisterCore;
use sdl3::event::Event;
use tracing::{debug, error};

use mister_fpga::fpga;
use mister_fpga::osd::OsdDisplay;

use crate::core_manager::CoreManager;
use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::platform::sdl::{SdlInitState, SdlPlatform, Window};
use crate::macguiver::platform::Platform;
use crate::platform::sizes;

const SDL_VIDEO_DRIVER_VARNAME: &str = "SDL_VIDEO_DRIVER";
const SDL_VIDEO_DRIVER_DEFAULT: &str = "evdev";

pub struct De10Platform {
    pub platform: SdlPlatform<BinaryColor>,
    _window: Window<BinaryColor>,
    title_display: OsdDisplay,
    osd_display: OsdDisplay,
    core_manager: CoreManager,
    core_framebuffer: DrawBuffer<Rgb888>,
}

impl Default for De10Platform {
    fn default() -> Self {
        if std::env::var_os(SDL_VIDEO_DRIVER_VARNAME).is_none() {
            std::env::set_var(SDL_VIDEO_DRIVER_VARNAME, SDL_VIDEO_DRIVER_DEFAULT);
        }

        let mut platform = SdlPlatform::init(SdlInitState::default());

        // Need at least 1 window to get events.
        let window = platform.window("Title", Size::new(1, 1));

        let fpga = fpga::MisterFpga::init().unwrap();
        if !fpga.is_ready() {
            debug!("GPI[31] == 1");
            error!("FPGA is uninitialized or incompatible core loaded.");
            error!("Quitting. Bye bye...\n");
            std::process::exit(1);
        }

        let core_manager = CoreManager::new(fpga);
        let core_framebuffer = DrawBuffer::new(sizes::MAIN);

        Self {
            platform,
            _window: window,
            title_display: OsdDisplay::title(),
            osd_display: OsdDisplay::main(),
            core_manager,
            core_framebuffer,
        }
    }
}

impl De10Platform {
    pub fn init(&mut self) {
        self.core_manager.load_menu().unwrap();
    }

    pub fn update_toolbar(&mut self, buffer: &DrawBuffer<BinaryColor>) {
        self.title_display
            .send(self.core_manager.fpga_mut(), buffer);
    }

    pub fn update_osd(&mut self, buffer: &DrawBuffer<BinaryColor>) {
        self.osd_display.send(self.core_manager.fpga_mut(), buffer);
    }

    pub fn update_menu_framebuffer(&mut self) {
        if let Some(mut c) = self.core_manager_mut().get_current_core() {
            if let Some(menu) = c.as_menu_core_mut() {
                let size = self.core_framebuffer.size();
                let img = RgbImage::from_raw(
                    size.width,
                    size.height,
                    self.core_framebuffer.to_be_bytes(),
                );

                if let Some(i) = img {
                    menu.send_to_framebuffer(&i).unwrap();
                }
            }
        }
    }

    pub fn toolbar_dimensions(&self) -> Size {
        sizes::TITLE
    }

    pub fn osd_dimensions(&self) -> Size {
        sizes::MAIN
    }

    pub fn main_buffer(&mut self) -> &mut DrawBuffer<Rgb888> {
        &mut self.core_framebuffer
    }

    pub fn events(&mut self) -> Vec<Event> {
        self.platform.events()
    }

    pub fn sdl(&mut self) -> &mut SdlPlatform<BinaryColor> {
        &mut self.platform
    }

    pub fn start_loop(&mut self) {}

    pub fn end_loop(&mut self) {}

    pub fn core_manager_mut(&mut self) -> &mut CoreManager {
        &mut self.core_manager
    }
}

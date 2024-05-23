use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{OriginDimensions, Point, Size};
use embedded_graphics::pixelcolor::{BinaryColor, Rgb888};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::Drawable;
use sdl3::event::Event;
use std::convert::TryInto;
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
    title_display: OsdDisplay,
    main_display: OsdDisplay,
    _window: Window<BinaryColor>,

    toolbar_buffer: DrawBuffer<BinaryColor>,

    core_manager: CoreManager,

    buffer: DrawBuffer<Rgb888>,
    mapper: cyclone_v::memory::DevMemMemoryMapper,
    core_framebuffer: DrawBuffer<Rgb888>,
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

        let fpga = fpga::MisterFpga::init().unwrap();
        if !fpga.is_ready() {
            debug!("GPI[31] == 1");
            error!("FPGA is uninitialized or incompatible core loaded.");
            error!("Quitting. Bye bye...\n");
            std::process::exit(1);
        }

        let core_manager = CoreManager::new(fpga);

        const FB_BASE: usize = 0x20000000 + (32 * 1024 * 1024);
        let fb_addr = FB_BASE + (1920 * 1080) * 4;
        use cyclone_v::memory::MemoryMapper;
        let mut mapper =
            cyclone_v::memory::DevMemMemoryMapper::create(fb_addr, 1920 * 1080 * 4).unwrap();

        let slice: &'static mut [Rgb888] =
            unsafe { std::slice::from_raw_parts_mut(mapper.as_mut_ptr(), 1920 * 1080) };
        let slice: &'static mut [Rgb888; 1920 * 1080] = slice.try_into().unwrap();

        let core_framebuffer =
            unsafe { DrawBuffer::from_memory_slice(slice, Size::new(1920, 1080)) };
        let buffer = DrawBuffer::new(sizes::MAIN);

        Self {
            platform,
            title_display,
            main_display,
            _window: window,
            toolbar_buffer,
            core_manager,
            mapper,
            buffer,
            core_framebuffer, // linux_framebuffer,
        }
    }
}

impl De10Platform {
    pub fn init(&mut self) {
        self.core_manager.load_menu().unwrap();
    }

    pub fn update_toolbar(&mut self, buffer: &DrawBuffer<BinaryColor>) {
        self.toolbar_buffer.clear(BinaryColor::Off).unwrap();
        buffer.draw(&mut self.toolbar_buffer).unwrap();
        self.title_display
            .send(self.core_manager.fpga_mut(), &self.toolbar_buffer);
    }

    pub fn update_osd(&mut self, buffer: &DrawBuffer<BinaryColor>) {
        self.main_display.send(self.core_manager.fpga_mut(), buffer);
    }

    pub fn update_menu_framebuffer(&mut self) {
        self.buffer
            .draw(
                &mut self
                    .core_framebuffer
                    .sub_buffer(Rectangle::new(Point::new(300, 300), self.buffer.size())),
            )
            .unwrap();
    }

    pub fn toolbar_dimensions(&self) -> Size {
        sizes::TITLE
    }

    pub fn main_dimensions(&self) -> Size {
        sizes::MAIN
    }

    pub fn main_buffer(&mut self) -> &mut DrawBuffer<Rgb888> {
        &mut self.buffer
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

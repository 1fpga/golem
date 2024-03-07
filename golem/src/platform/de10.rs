#![cfg(feature = "platform_de10")]
use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::platform::sdl::{SdlInitState, SdlPlatform, Window};
use crate::macguiver::platform::Platform;
use crate::platform::{sizes, CoreManager, GoLEmPlatform};
use crate::Flags;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{OriginDimensions, Size};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::Drawable;
use mister_fpga::fpga;
use mister_fpga::osd::OsdDisplay;
use sdl3::event::Event;
use tracing::{debug, error};

pub mod core_manager;

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

        let fpga = fpga::MisterFpga::init().unwrap();
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

impl GoLEmPlatform for De10Platform {
    type Color = BinaryColor;
    type CoreManager = core_manager::CoreManager;

    fn init(&mut self, _flags: &Flags) {
        self.core_manager.load_menu().unwrap();
    }

    fn update_toolbar(&mut self, buffer: &DrawBuffer<Self::Color>) {
        self.toolbar_buffer.clear(BinaryColor::Off).unwrap();
        buffer.draw(&mut self.toolbar_buffer).unwrap();
        self.title_display
            .send(self.core_manager.fpga_mut(), &self.toolbar_buffer);
    }

    fn update_main(&mut self, buffer: &DrawBuffer<Self::Color>) {
        self.main_display.send(self.core_manager.fpga_mut(), buffer);
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

    fn sdl(&mut self) -> &mut SdlPlatform<Self::Color> {
        &mut self.platform
    }

    fn start_loop(&mut self) {}

    fn end_loop(&mut self) {}

    fn core_manager_mut(&mut self) -> &mut Self::CoreManager {
        &mut self.core_manager
    }
}

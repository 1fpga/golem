use crate::macguiver::application::Application;
use crate::platform::{PlatformInner, PlatformState};
use crate::{fpga, osd, spi};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;

mod buffer;

pub struct De10Platform {
    pub osd: buffer::OsdDisplayView,
    pub title: buffer::OsdDisplayView,
}

impl Default for De10Platform {
    fn default() -> Self {
        let osd = buffer::OsdDisplayView::main();
        let title = buffer::OsdDisplayView::title();

        Self { osd, title }
    }
}

impl PlatformInner for De10Platform {
    type Color = BinaryColor;

    fn run(&mut self, app: &mut impl Application<Color = Self::Color>) {
        let state = PlatformState::default();

        osd::OsdSetSize(19);
        unsafe {
            while fpga::is_fpga_ready(1) == 0 {
                fpga::fpga_wait_to_reset();
            }
        }

        unsafe {
            loop {
                crate::user_io::user_io_poll();
                crate::input::input_poll(0);

                app.update(&state);

                // Clear the buffers.
                self.osd.clear(BinaryColor::Off).unwrap();
                self.title.clear(BinaryColor::Off).unwrap();

                app.draw(&mut self.osd.inner);
                app.draw_title(&mut self.title.inner);

                for line in self.osd.line_iter() {
                    let line_buffer = self.osd.get_binary_line_array(line);
                    spi::spi_osd_cmd_cont(osd::OSD_CMD_WRITE | (line as u8));
                    spi::spi_write(line_buffer.as_ptr(), 256, 0);
                    spi::DisableOsd();
                }
                for line in self.title.line_iter() {
                    let line_buffer = self.title.get_binary_line_array(line);
                    spi::spi_osd_cmd_cont(osd::OSD_CMD_WRITE | (line as u8));
                    spi::spi_write(line_buffer.as_ptr(), 256, 0);
                    spi::DisableOsd();
                }
            }
        }
    }
}

pub use De10Platform as PlatformWindowManager;

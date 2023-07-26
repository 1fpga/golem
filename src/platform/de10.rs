use crate::macguiver::application::Application;
use crate::platform::{PlatformInner, PlatformState};
use crate::{fpga, menu, osd, spi};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;

mod buffer;
mod keyboard;

use input::event::keyboard::KeyboardEventTrait;
use input::event::EventTrait;
use input::{Libinput, LibinputInterface};
use libc::{O_RDONLY, O_RDWR, O_WRONLY};
use std::fs::{File, OpenOptions};
use std::os::unix::{fs::OpenOptionsExt, io::OwnedFd};
use std::path::Path;

struct Interface;

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_RDONLY != 0) | (flags & O_RDWR != 0))
            .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
            .open(path)
            .map(|file| file.into())
            .map_err(|err| err.raw_os_error().unwrap())
    }
    fn close_restricted(&mut self, fd: OwnedFd) {
        drop(File::from(fd));
    }
}

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

        let context = sdl2::init().unwrap();
        let mut event_pump = context.event_pump().unwrap();
        let kb = context.keyboard();

        osd::OsdSetSize(19);
        unsafe {
            while fpga::is_fpga_ready(1) == 0 {
                fpga::fpga_wait_to_reset();
            }
        }

        let mut input = Libinput::new_with_udev(Interface);
        input.udev_assign_seat("seat0").unwrap();

        unsafe {
            loop {
                crate::user_io::user_io_poll();
                crate::input::input_poll(0);
                menu::HandleUI();

                app.update(&state);

                // Clear the buffers.
                self.osd.clear(BinaryColor::Off).unwrap();
                self.title.clear(BinaryColor::Off).unwrap();

                app.draw(&mut self.osd.inner);
                app.draw_title(&mut self.title.inner);
                self.title.inner.invert();

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

                input.dispatch().unwrap();
                for ev in &mut input {
                    match ev {
                        input::Event::Device(device) => {
                            eprintln!("added device: {:?}", device.device());
                        }
                        event => println!("Got event: {:?}", event),
                    }
                }

                for ev in event_pump.poll_iter() {
                    eprintln!("sdl2 event: {:?}", ev);
                }

                eprintln!("loop...");
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
}

pub use De10Platform as PlatformWindowManager;

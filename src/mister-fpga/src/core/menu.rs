use crate::config::Config;
use crate::core::MisterFpgaCore;
use crate::fpga::MisterFpga;
use crate::types::units::UnitConversion;
use cyclone_v::memory::{DevMemMemoryMapper, MemoryMapper};
use image::DynamicImage;
use one_fpga::core::{Bios, ConfigMenuId, CoreMenuItem, Error, MountedFile, Rom, SaveState};
use one_fpga::inputs::Button;
use one_fpga::inputs::Scancode;
use one_fpga::Core;
use std::time::SystemTime;
use tracing::warn;

pub struct MenuCore {
    inner: MisterFpgaCore,
    menu_fb_mapper: DevMemMemoryMapper,
}

impl MenuCore {
    pub fn new(inner: MisterFpga) -> Result<Self, String> {
        let mut inner = MisterFpgaCore::new(inner)?;
        inner.is_menu = true;

        let fb_base: usize = cyclone_v::ranges::HOST_MEMORY.start + 32.mebibytes();
        let fb_addr = fb_base + (1920 * 1080) * 4;
        let menu_fb_mapper = DevMemMemoryMapper::create(fb_addr, 1920 * 1080 * 4).unwrap();

        Ok(Self {
            inner,
            menu_fb_mapper,
        })
    }

    pub fn send_to_framebuffer(&mut self, image: &image::RgbImage) -> Result<(), String> {
        let menu_fb_size = self.inner.video_info()?.resolution();

        let mut dest = image::ImageBuffer::from_raw(
            menu_fb_size.width as u32,
            menu_fb_size.height as u32,
            self.menu_fb_mapper.as_mut_range(..),
        )
        .unwrap();

        if image.width() > menu_fb_size.width as u32 || image.height() > menu_fb_size.height as u32
        {
            warn!("Image size is larger than framebuffer size");
        }

        image::imageops::overlay(&mut dest, image, 0, 0);

        Ok(())
    }
}

impl Core for MenuCore {
    fn init(&mut self) -> Result<(), Error> {
        self.inner.init()?;
        self.inner.init_video(&Config::base().into_inner(), true)?;
        Ok(())
    }

    fn name(&self) -> &str {
        "MENU"
    }

    fn reset(&mut self) -> Result<(), Error> {
        self.inner.soft_reset();
        Ok(())
    }

    fn set_volume(&mut self, volume: u8) -> Result<(), Error> {
        self.inner.set_volume(volume)
    }

    fn set_rtc(&mut self, time: SystemTime) -> Result<(), Error> {
        self.inner.set_rtc(time)
    }

    fn screenshot(&self) -> Result<DynamicImage, Error> {
        self.inner.screenshot()
    }

    fn save_state_mut(&mut self, _slot: usize) -> Result<Option<&mut dyn SaveState>, Error> {
        unreachable!("Menu core does not support save states")
    }

    fn save_state(&self, _slot: usize) -> Result<Option<&dyn SaveState>, Error> {
        unreachable!("Menu core does not support save states")
    }

    fn mounted_file_mut(&mut self, _slot: usize) -> Result<Option<&mut dyn MountedFile>, Error> {
        unreachable!("Menu core does not support mounted files")
    }

    fn send_rom(&mut self, _rom: Rom) -> Result<(), Error> {
        unreachable!("Menu core does not support ROMs")
    }

    fn send_bios(&mut self, _bios: Bios) -> Result<(), Error> {
        unreachable!("Menu core does not support BIOS")
    }

    fn key_up(&mut self, _key: Scancode) -> Result<(), Error> {
        unreachable!("Menu core does not support inputs")
    }

    fn key_down(&mut self, _key: Scancode) -> Result<(), Error> {
        unreachable!("Menu core does not support inputs")
    }

    fn keys_set(&mut self, _keys: &[Scancode]) -> Result<(), Error> {
        unreachable!("Menu core does not support inputs")
    }

    fn keys_update(&mut self, _up: &[Scancode], _down: &[Scancode]) -> Result<(), Error> {
        unreachable!("Menu core does not support inputs")
    }

    fn keys(&self) -> Result<&[Scancode], Error> {
        unreachable!("Menu core does not support inputs")
    }

    fn gamepad_button_up(&mut self, _index: usize, _button: Button) -> Result<(), Error> {
        unreachable!("Menu core does not support inputs")
    }

    fn gamepad_button_down(&mut self, _index: usize, _button: Button) -> Result<(), Error> {
        unreachable!("Menu core does not support inputs")
    }

    fn gamepad_buttons_set(&mut self, _index: usize, _buttons: &[Button]) -> Result<(), Error> {
        unreachable!("Menu core does not support inputs")
    }

    fn gamepad_buttons_update(
        &mut self,
        _index: usize,
        _up: &[Button],
        _down: &[Button],
    ) -> Result<(), Error> {
        unreachable!("Menu core does not support inputs")
    }

    fn gamepad_buttons(&self, _index: usize) -> Result<Option<&[Button]>, Error> {
        unreachable!("Menu core does not support inputs")
    }

    fn menu(&self) -> Result<Vec<CoreMenuItem>, Error> {
        Ok(vec![])
    }

    fn trigger(&mut self, _id: ConfigMenuId) -> Result<(), Error> {
        todo!()
    }

    fn int_option(&mut self, _id: ConfigMenuId, _value: u32) -> Result<(), Error> {
        todo!()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

use byteorder::{LittleEndian, ReadBytesExt};
use mister_fpga::config::{Config, HdmiLimitedConfig, VgaMode};
use mister_fpga::core::MisterFpgaCore;
use mister_fpga::fpga::user_io::{ButtonSwitches, UserIoButtonSwitch};
use mister_fpga::fpga::MisterFpga;
use std::path::Path;

pub struct CoreManager {
    fpga: MisterFpga,
}

impl CoreManager {
    pub fn new(fpga: MisterFpga) -> Self {
        Self { fpga }
    }

    pub fn fpga(&self) -> &MisterFpga {
        &self.fpga
    }

    pub fn fpga_mut(&mut self) -> &mut MisterFpga {
        &mut self.fpga
    }

    /// Create a core for the current FPGA configuration.
    fn create_core(&mut self, is_menu: bool) -> Result<crate::GolemCore, String> {
        let mut core = MisterFpgaCore::new(self.fpga.clone())
            .map_err(|e| format!("Could not instantiate Core: {e}"))?;

        let options = Config::base().into_inner();

        core.init()?;
        core.init_video(&options, is_menu)?;
        core.send_volume(0)?;

        let mut switches = UserIoButtonSwitch::new();
        if options.vga_scaler == Some(true) {
            switches |= ButtonSwitches::VgaScaler;
        }
        if options.vga_sog == Some(true) {
            switches |= ButtonSwitches::VgaSog;
        }
        if options.composite_sync == Some(true) {
            switches |= ButtonSwitches::CompositeSync;
        }
        if options.vga_mode() == VgaMode::Ypbpr {
            switches |= ButtonSwitches::Ypbpr;
        }
        if options.forced_scandoubler() {
            switches |= ButtonSwitches::ForcedScandoubler;
        }
        if options.hdmi_audio_96k() {
            switches |= ButtonSwitches::Audio96K;
        }
        if options.dvi_mode() {
            switches |= ButtonSwitches::Dvi;
        }
        match options.hdmi_limited() {
            HdmiLimitedConfig::Limited => switches |= ButtonSwitches::HdmiLimited1,
            HdmiLimitedConfig::LimitedForVgaConverters => switches |= ButtonSwitches::HdmiLimited2,
            _ => {}
        }
        if options.direct_video() {
            switches |= ButtonSwitches::DirectVideo;
        }

        core.spi_mut().execute(switches).unwrap();
        core.send_rtc()?;

        Ok(crate::GolemCore::new(core))
    }

    pub fn load(&mut self, program: &[u8], is_menu: bool) -> Result<crate::GolemCore, String> {
        let program = if &program[..6] != b"MiSTer" {
            program
        } else {
            let start = 16;
            let size = (&program[12..]).read_u32::<LittleEndian>().unwrap() as usize;
            &program[start..start + size]
        };

        self.fpga.wait_for_ready();
        self.fpga
            .load(program)
            .map_err(|e| format!("Could not load program: {e:?}"))?;
        self.fpga
            .core_reset()
            .map_err(|_| "Could not reset the Core".to_string())?;

        let core = self.create_core(is_menu)?;

        Ok(core)
    }

    pub fn load_menu(&mut self) -> Result<crate::GolemCore, String> {
        let bytes = include_bytes!("../assets/menu.rbf");

        let mut core = self.load(bytes, true)?;

        // Send the logo to the framebuffer.
        let logo = include_bytes!("../../../logo.png");
        let image = image::load_from_memory_with_format(logo, image::ImageFormat::Png)
            .map_err(|e| format!("Could not load logo: {e}"))?;

        let mut fullframe = image::DynamicImage::new_rgba8(1920, 1080);
        let rgba8 = fullframe.as_mut_rgba8().unwrap();
        rgba8
            .pixels_mut()
            .for_each(|p| *p = image::Rgba([64, 64, 64, 0]));
        image::imageops::overlay(&mut fullframe, &image, 32, 32);
        core.send_to_framebuffer(fullframe.as_bytes())?;

        self.fpga_mut().osd_enable();
        Ok(core)
    }

    pub fn load_core(&mut self, path: impl AsRef<Path>) -> Result<crate::GolemCore, String> {
        let bytes = std::fs::read(path.as_ref()).map_err(|e| e.to_string())?;
        let core = self.load(&bytes, false)?;
        Ok(core)
    }

    pub fn get_current_core(&mut self) -> Result<crate::GolemCore, String> {
        let core = MisterFpgaCore::new(self.fpga.clone())
            .map_err(|e| format!("Could not instantiate Core: {e}"))?;
        Ok(crate::GolemCore::new(core))
    }

    pub fn show_menu(&mut self) {
        self.fpga_mut().osd_enable();
    }

    pub fn hide_menu(&mut self) {
        self.fpga_mut().osd_disable();
    }
}

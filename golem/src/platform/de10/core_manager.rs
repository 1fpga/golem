use byteorder::{LittleEndian, ReadBytesExt};
use mister_fpga::config::{Config, HdmiLimitedConfig, VgaMode};
use mister_fpga::core::MisterFpgaCore;
use mister_fpga::fpga::user_io::{ButtonSwitches, UserIoButtonSwitch};
use mister_fpga::fpga::MisterFpga;
use std::path::Path;

pub mod core;

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
    fn create_core(&mut self, is_menu: bool) -> Result<core::MisterFpgaCore, String> {
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

        unsafe {
            crate::platform::de10::user_io::user_io_init(
                "\0".as_ptr() as *const _,
                std::ptr::null(),
            );
        }

        // extern "C" {
        //     pub fn user_io_send_buttons(force: u8);
        // }
        //
        // unsafe {
        //     user_io_send_buttons(1);
        // }
        //
        // let mut bits = core.read_status_bits().clone();
        // bits.set(0, true);
        // bits.set(4, false);
        // bits.set_range(1..4, 0);
        // core.send_status_bits(bits);
        // core.read_status_bits();
        // core.send_rtc()?;

        Ok(core::MisterFpgaCore::new(core))
    }

    fn load(&mut self, program: &[u8], is_menu: bool) -> Result<core::MisterFpgaCore, String> {
        let program = if &program[..6] != b"MiSTer" {
            program
        } else {
            let start = 16;
            let size = (&program[12..]).read_u32::<LittleEndian>().unwrap() as usize;
            &program[start..start + size]
        };

        self.fpga.wait_for_ready();
        self.fpga
            .load_rbf(program)
            .map_err(|e| format!("Could not load program: {e:?}"))?;
        self.fpga
            .core_reset()
            .map_err(|_| "Could not reset the Core".to_string())?;

        let core = self.create_core(is_menu)?;

        Ok(core)
    }
}

impl crate::platform::CoreManager for CoreManager {
    type Core = core::MisterFpgaCore;

    fn load_core(&mut self, path: impl AsRef<Path>) -> Result<Self::Core, String> {
        let bytes = std::fs::read(path.as_ref()).map_err(|e| e.to_string())?;
        let core = self.load(&bytes, false)?;
        Ok(core)
    }

    fn get_current_core(&mut self) -> Result<Self::Core, String> {
        let core = MisterFpgaCore::new(self.fpga.clone())
            .map_err(|e| format!("Could not instantiate Core: {e}"))?;
        Ok(core::MisterFpgaCore::new(core))
    }

    fn load_menu(&mut self) -> Result<Self::Core, String> {
        #[repr(align(4))]
        struct Aligned<T: ?Sized>(T);

        let bytes = Aligned(include_bytes!("../../../assets/menu.rbf"));

        let core = self.load(bytes.0, true)?;
        self.fpga_mut().osd_enable();
        Ok(core)
    }

    fn show_menu(&mut self) {
        self.fpga_mut().osd_enable();
    }

    fn hide_menu(&mut self) {
        self.fpga_mut().osd_disable();
    }
}

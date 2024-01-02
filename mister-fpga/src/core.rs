use crate::config_string;
use crate::config_string::{FpgaRamMemoryAddress, LoadFileInfo};
use crate::core::buttons::ButtonMap;
use crate::fpga::file_io::{
    FileIoFileExtension, FileIoFileIndex, FileIoFileTxData16Bits, FileIoFileTxData8Bits,
    FileIoFileTxDisabled, FileIoFileTxEnabled,
};
use crate::fpga::user_io::{
    GetStatusBits, SetStatusBits, UserIoJoystick, UserIoKeyboardKeyDown, UserIoKeyboardKeyUp,
    UserIoRtc,
};
use crate::fpga::{CoreInterfaceType, CoreType, MisterFpga};
use crate::keyboard::Ps2Scancode;
use crate::savestate::Savestate;
use crate::types::StatusBitMap;
use cyclone_v::memory::{DevMemMemoryMapper, MemoryMapper};
use image::DynamicImage;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;
use tracing::{debug, info, trace};

pub mod buttons;

pub enum MisterFpgaSendFileInfo {
    Memory {
        index: u8,
        address: FpgaRamMemoryAddress,
    },
    Buffered {
        index: u8,
    },
}

impl MisterFpgaSendFileInfo {
    pub fn from_file_info(file_info: LoadFileInfo) -> Result<Self, String> {
        let index = file_info.index;
        match file_info.address {
            None => Ok(Self::Buffered { index }),
            Some(address) => Ok(Self::Memory { index, address }),
        }
    }

    pub fn from_path(path: impl AsRef<Path>, core: &MisterFpgaCore) -> Result<Self, String> {
        let info = core
            .config
            .load_info(path)?
            .ok_or("Could not find info for extension")?;
        Self::from_file_info(info)
    }
}

pub struct MisterFpgaCore {
    fpga: MisterFpga,
    pub core_type: CoreType,
    pub spi_type: CoreInterfaceType,
    pub io_version: u8,
    config: config_string::Config,

    save_states: Option<SaveStateManager<DevMemMemoryMapper>>,
    map: ButtonMap,

    status: StatusBitMap,
}

impl MisterFpgaCore {
    pub fn new(mut fpga: MisterFpga) -> Result<Self, String> {
        let core_type = fpga.core_type().ok_or("Could not get core type.")?;
        let spi_type = fpga
            .core_interface_type()
            .ok_or("Could not get SPI type.")?;
        let io_version = fpga.core_io_version().ok_or("Could not get IO version.")?;
        let config = config_string::Config::from_fpga(&mut fpga)?;

        let mut map = ButtonMap::default();
        if let Some(list) = config.snes_default_button_list() {
            info!("Loading mapping from config file");
            map = ButtonMap::map_from_snes_list(
                &list.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
            );
        }
        info!(?map);

        info!(?core_type, ?spi_type, io_version, "Core loaded");
        info!(
            "Status bit map (mask):\n{}",
            config.status_bit_map_mask().debug_string(true)
        );
        info!("Core config: {:#?}", config);
        fpga.wait_for_ready();

        let save_states = SaveStateManager::from_config_string(&config);

        Ok(MisterFpgaCore {
            fpga,
            core_type,
            spi_type,
            io_version,
            config,
            save_states,
            map,
            status: Default::default(),
        })
    }

    pub fn send_rtc(&mut self) -> Result<(), String> {
        self.fpga.spi_mut().execute(UserIoRtc::now())?;
        Ok(())
    }

    pub fn send_file(
        &mut self,
        path: &Path,
        file_info: Option<LoadFileInfo>,
    ) -> Result<(), String> {
        let info = file_info.map_or_else(
            || MisterFpgaSendFileInfo::from_path(path, self),
            MisterFpgaSendFileInfo::from_file_info,
        )?;

        let ext = path
            .extension()
            .unwrap_or(OsStr::new(""))
            .to_str()
            .unwrap_or("")
            .to_uppercase();

        let now = std::time::Instant::now();
        debug!("Sending file {:?} to core", path);
        match info {
            MisterFpgaSendFileInfo::Memory { index, address } => {
                let f = File::open(path).map_err(|e| e.to_string())?;
                let size = f.metadata().map_err(|e| e.to_string())?.len() as u32;
                trace!(?index, ?address, ?ext, ?size, "File info (memory)");
                self.send_file_to_memory_(index, &ext, size, address, f)?;
            }
            MisterFpgaSendFileInfo::Buffered { index } => {
                let f = File::open(path).map_err(|e| e.to_string())?;
                let size = f.metadata().map_err(|e| e.to_string())?.len() as u32;
                trace!(?index, ?ext, ?size, "File info (buffered)");
                self.send_file_to_buffer_(index, &ext, size, f)?;
            }
        }
        debug!("Done in {}ms", now.elapsed().as_millis());

        Ok(())
    }

    fn start_send_file(&mut self, index: u8, ext: &str, size: u32) -> Result<(), String> {
        self.fpga.spi_mut().execute(FileIoFileIndex::from(index))?;
        self.fpga.spi_mut().execute(FileIoFileExtension(ext))?;
        self.fpga.spi_mut().execute(FileIoFileTxEnabled(Some(size)))
    }

    fn end_send_file(&mut self) -> Result<(), String> {
        self.read_status_bits();

        // Disable download.
        self.fpga.spi_mut().execute(FileIoFileTxDisabled)
    }

    pub fn config(&self) -> &config_string::Config {
        &self.config
    }

    pub fn status_bits(&self) -> &StatusBitMap {
        &self.status
    }

    pub fn read_status_bits(&mut self) -> &StatusBitMap {
        self.fpga
            .spi_mut()
            .execute(GetStatusBits(&mut self.status))
            .unwrap();
        &self.status
    }

    pub fn send_status_bits(&mut self, bits: StatusBitMap) {
        debug!(?bits, "Setting status bits");
        self.fpga.spi_mut().execute(SetStatusBits(&bits)).unwrap();
        self.status = bits;
    }

    pub fn key_down(&mut self, keycode: sdl3::keyboard::Scancode) {
        let scancode = Ps2Scancode::from(keycode);
        debug!(?keycode, ?scancode, "Keydown");
        if scancode != Ps2Scancode::None {
            self.fpga
                .spi_mut()
                .execute(UserIoKeyboardKeyDown::from(scancode))
                .unwrap();
        }
    }

    pub fn key_up(&mut self, keycode: sdl3::keyboard::Scancode) {
        let scancode = Ps2Scancode::from(keycode);
        debug!(?keycode, ?scancode, "Keyup");
        self.fpga
            .spi_mut()
            .execute(UserIoKeyboardKeyUp::from(scancode))
            .unwrap();
    }

    pub fn gamepad_button_down(&mut self, joystick_idx: u8, button: u8) {
        self.map.down(button);

        self.fpga
            .spi_mut()
            .execute(UserIoJoystick::from_joystick_index(joystick_idx, &self.map))
            .unwrap();
    }

    pub fn gamepad_button_up(&mut self, joystick_idx: u8, button: u8) {
        self.map.up(button);

        self.fpga
            .spi_mut()
            .execute(UserIoJoystick::from_joystick_index(joystick_idx, &self.map))
            .unwrap();
    }

    pub fn take_screenshot(&mut self) -> Result<DynamicImage, String> {
        crate::framebuffer::FpgaFramebuffer::default().take_screenshot()
    }

    fn send_file_to_memory_(
        &mut self,
        index: u8,
        ext: &str,
        size: u32,
        address: FpgaRamMemoryAddress,
        mut reader: impl std::io::Read,
    ) -> Result<(), String> {
        // Verify invariants.
        if size >= 0x2000_0000 {
            return Err("File too large.".to_string());
        }
        let mut crc = crc32fast::Hasher::new();
        let mut mem = DevMemMemoryMapper::create(address.as_usize(), size as usize)?;

        self.start_send_file(index, ext, size)?;

        let mut bytes2send = size;
        while bytes2send > 0 {
            // let start = (size - bytes2send) as usize;
            // let len = (bytes2send.min(256u32.kibibytes())) as usize;
            let sz = reader
                // .read(mem.as_mut_range(start..start + len))
                .read(mem.as_mut_range(..))
                .map_err(|e| e.to_string())?;

            // crc.update(mem.as_range(start..start + len));
            bytes2send -= sz as u32;
        }
        crc.update(mem.as_range(..));
        self.end_send_file()?;

        // TODO: implement check_status_change into Rust.
        extern "C" {
            pub fn check_status_change();
        }
        unsafe {
            check_status_change();
        }

        let crc = crc.finalize();
        debug!("CRC: {:08X}", crc);
        Ok(())
    }

    fn send_file_to_buffer_(
        &mut self,
        index: u8,
        ext: &str,
        size: u32,
        mut reader: impl std::io::Read,
    ) -> Result<(), String> {
        // Verify invariants.
        if size >= 0x2000_0000 {
            return Err("File too large.".to_string());
        }
        self.start_send_file(index, ext, size)?;

        let mut crc = crc32fast::Hasher::new();
        let now = std::time::Instant::now();

        let mut buffer = [0u8; 4096];
        loop {
            match reader.read(&mut buffer).map_err(|e| e.to_string()) {
                Ok(0) => break,
                Ok(size) => {
                    crc.update(&buffer[..size]);

                    match self.spi_type {
                        CoreInterfaceType::SpiBus8Bit => {
                            self.fpga
                                .spi_mut()
                                .execute(FileIoFileTxData8Bits(&buffer[..size]))?;
                        }
                        CoreInterfaceType::SpiBus16Bit => {
                            // This is safe since size cannot be larger than 4096.
                            let buf16 = unsafe {
                                std::slice::from_raw_parts(
                                    buffer.as_ptr() as *const u16,
                                    size + size % 2,
                                )
                            };
                            self.fpga
                                .spi_mut()
                                .execute(FileIoFileTxData16Bits(&buf16[..size / 2]))?;
                        }
                    }
                }
                Err(e) => {
                    self.end_send_file()?;
                    return Err(e);
                }
            }
        }

        debug!("Read {} bytes", size);
        trace!("Took {}ms", now.elapsed().as_millis());
        let crc = crc.finalize();
        debug!("CRC: {:08X}", crc);

        self.end_send_file()?;
        Ok(())
    }
}

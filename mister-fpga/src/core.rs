use crate::config_string;
use crate::config_string::{FpgaRamMemoryAddress, LoadFileInfo};
use crate::core::buttons::ButtonMap;
use crate::fpga::{CoreInterfaceType, CoreType, MisterFpga, SpiCommands};
use crate::types::StatusBitMap;
use cyclone_v::memory::{DevMemMemoryMapper, MemoryMapper};
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

    map: ButtonMap,

    status: StatusBitMap,
}

impl MisterFpgaCore {
    pub fn new(mut fpga: MisterFpga) -> Result<Self, &'static str> {
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

        Ok(MisterFpgaCore {
            fpga,
            core_type,
            spi_type,
            io_version,
            config,
            // mapping,
            map,
            status: Default::default(),
        })
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
        // Send index.
        self.fpga
            .spi_mut()
            .command(SpiCommands::FileIoFileIndex)
            .write_b(index);

        // Send extension.
        let ext_bytes = ext.as_bytes();
        // Extend to 4 characters with the dot.
        let ext: [u8; 4] = [
            b'.',
            ext_bytes.get(0).copied().unwrap_or(0),
            ext_bytes.get(1).copied().unwrap_or(0),
            ext_bytes.get(2).copied().unwrap_or(0),
        ];

        self.fpga
            .spi_mut()
            .command(SpiCommands::FileIoFileInfo)
            .write((ext[0] as u16) << 8 | ext[1] as u16)
            .write((ext[2] as u16) << 8 | ext[3] as u16);

        self.fpga
            .spi_mut()
            .command(SpiCommands::FileIoFileTx)
            .write_b(0xff)
            .write_cond(size != 0, size as u16)
            .write_cond(size != 0, (size >> 16) as u16);

        Ok(())
    }

    fn end_send_file(&mut self) -> Result<(), String> {
        self.read_status_bits();

        // Disable download.
        self.fpga
            .spi_mut()
            .command(SpiCommands::FileIoFileTx)
            .write_b(0);
        Ok(())
    }

    pub fn config(&self) -> &config_string::Config {
        &self.config
    }

    pub fn status_bits(&self) -> &StatusBitMap {
        &self.status
    }

    pub fn read_status_bits(&mut self) -> &StatusBitMap {
        let mut bits = StatusBitMap::default();
        {
            let mut stchg = 0;
            let mut command = self
                .fpga
                .spi_mut()
                .command_read(SpiCommands::GetStatusBits, &mut stchg);

            if ((stchg & 0xF0) == 0xA0) && (stchg & 0x0F) != 0 {
                for word in bits.as_mut_raw_slice() {
                    command = command.write_read(0u16, word);
                }
            }
        }

        bits.set(0, false);
        self.status = bits;
        &self.status
    }

    pub fn send_status_bits(&mut self, bits: StatusBitMap) {
        debug!(?bits, "Setting status bits");
        let bits16 = bits.as_raw_slice();

        let command = self
            .fpga
            .spi_mut()
            .command(SpiCommands::SetStatus32Bits)
            .write_buffer(&bits16[0..4]);
        if bits.has_extra() {
            command.write_buffer(&bits16[4..]);
        }
        self.status = bits;
    }

    pub fn send_key_code(&mut self, keycode: sdl3::keyboard::Keycode) {
        let key = keycode as u8;
        debug!(?key, "Sending key code");
        self.fpga
            .spi_mut()
            .command(SpiCommands::UserIoKeyboard)
            .write_b(key);
    }

    pub fn gamepad_button_down(&mut self, joystick_idx: u8, button: u8) {
        let button_mask = self.map.down(button);

        let spi = self.fpga.spi_mut();
        let command = spi
            .command(SpiCommands::from_joystick_index(joystick_idx))
            .write(button_mask as u16);
        if button_mask >> 16 == 0 {
            command.write((button_mask >> 16) as u16);
        }
    }

    pub fn gamepad_button_up(&mut self, joystick_idx: u8, button: u8) {
        let button_mask = self.map.up(button);

        let spi = self.fpga.spi_mut();
        let command = spi
            .command(SpiCommands::from_joystick_index(joystick_idx))
            .write(button_mask as u16);
        if button_mask >> 16 == 0 {
            command.write((button_mask >> 16) as u16);
        }
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
        self.start_send_file(index, ext, size)?;

        let mut crc = crc32fast::Hasher::new();
        let mut mem = DevMemMemoryMapper::create(address.as_usize(), size as usize)?;

        let now = std::time::Instant::now();
        let sz = reader
            .read(mem.as_mut_range(..))
            .map_err(|e| e.to_string())?;

        debug!("Read {} bytes", sz);
        trace!("Took {}ms", now.elapsed().as_millis());
        crc.update(mem.as_range(..sz));
        let crc = crc.finalize();
        debug!("CRC: {:08X}", crc);

        self.end_send_file()?;
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
                                .command(SpiCommands::FileIoFileTxDat)
                                .write_buffer_b(&buffer[..size]);
                        }
                        CoreInterfaceType::SpiBus16Bit => {
                            let buf16 = unsafe {
                                std::slice::from_raw_parts(
                                    buffer.as_ptr() as *const u16,
                                    size + size % 2,
                                )
                            };
                            self.fpga
                                .spi_mut()
                                .command(SpiCommands::FileIoFileTxDat)
                                .write_buffer(buf16);
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

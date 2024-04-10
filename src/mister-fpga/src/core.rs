use std::ffi::OsStr;
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use image::DynamicImage;
use tracing::{debug, info, trace};

use cyclone_v::memory::{DevMemMemoryMapper, MemoryMapper};

use crate::config::MisterConfig;
use crate::config_string;
use crate::config_string::{FpgaRamMemoryAddress, LoadFileInfo};
use crate::core::buttons::ButtonMap;
use crate::core::file::SdCard;
use crate::core::video::VideoInfo;
use crate::core::volume::IntoVolume;
use crate::fpga::{CoreInterfaceType, CoreType, MisterFpga, user_io};
use crate::fpga::file_io::{
    FileExtension, FileIndex, FileTxData16Bits, FileTxData8Bits, FileTxDisabled, FileTxEnabled,
};
use crate::fpga::user_io::{
    GetSdStat, GetStatusBits, SdRead, SdStatOutput, SdWrite, SetSdConf, SetSdInfo, SetSdStat,
    SetStatusBits, UserIoJoystick, UserIoKeyboardKeyDown, UserIoKeyboardKeyUp, UserIoRtc,
};
use crate::keyboard::Ps2Scancode;
use crate::savestate::SaveStateManager;
use crate::types::StatusBitMap;

pub mod buttons;
pub mod file;
pub mod volume;

mod video;

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

    pub fn index(&self) -> u8 {
        match self {
            Self::Memory { index, .. } => *index,
            Self::Buffered { index } => *index,
        }
    }
}

pub struct MisterFpgaCore {
    fpga: MisterFpga,
    pub core_type: CoreType,
    pub spi_type: CoreInterfaceType,
    pub io_version: u8,
    config: config_string::Config,

    // All the images that are mounted. Can only have 16 images at once.
    cards: Box<[Option<SdCard>; 16]>,

    save_states: Option<SaveStateManager<DevMemMemoryMapper>>,
    gamepads: [ButtonMap; 6],

    status: StatusBitMap,
    status_counter: u8,

    framebuffer: crate::framebuffer::FpgaFramebuffer<DevMemMemoryMapper>,
}

impl MisterFpgaCore {
    pub fn new(mut fpga: MisterFpga) -> Result<Self, String> {
        fpga.wait_for_ready();

        let config = config_string::Config::from_fpga(&mut fpga)?;

        let mut map = ButtonMap::default();
        if let Some(list) = config.snes_default_button_list() {
            info!("Loading mapping from config file");
            map = ButtonMap::map_from_snes_list(
                &list.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
            );
        }
        info!(?map);

        info!(
            "Status bit map (mask):\n{}",
            config.status_bit_map_mask().debug_string(true)
        );
        info!("Core config: {:#?}", config);

        let core_type = fpga.core_type().ok_or("Could not get core type.")?;
        let spi_type = fpga
            .core_interface_type()
            .ok_or("Could not get SPI type.")?;
        let io_version = fpga.core_io_version().ok_or("Could not get IO version.")?;
        info!(?core_type, ?spi_type, io_version, "Core loaded");

        let save_states = SaveStateManager::from_config_string(&config);
        const NONE: Option<SdCard> = None;

        Ok(MisterFpgaCore {
            fpga,
            core_type,
            spi_type,
            io_version,
            config,
            cards: Box::new([NONE; 16]),
            save_states,
            gamepads: [map; 6],
            status: Default::default(),
            status_counter: 0,
            framebuffer: crate::framebuffer::FpgaFramebuffer::default(),
        })
    }

    pub fn init(&mut self) -> Result<(), String> {
        self.soft_reset();
        self.fpga
            .spi_mut()
            .execute(user_io::SetMemorySize::from_fpga().unwrap())?;

        // Initialize the framebuffer.
        self.fpga.spi_mut().execute(user_io::SetFramebufferToCore)?;

        Ok(())
    }

    // TODO: move this to a de10 platform and to the GoLEm code.
    pub fn init_video(&mut self, config: &MisterConfig, is_menu: bool) -> Result<(), String> {
        video::init(config);
        video::init_mode(config, self, is_menu);
        Ok(())
    }

    pub fn spi_mut(&mut self) -> &mut crate::fpga::Spi<DevMemMemoryMapper> {
        self.fpga.spi_mut()
    }

    /// Perform a soft reset.
    pub fn soft_reset(&mut self) {
        self.read_status_bits();
        self.status.set(0, true);
        self.send_status_bits(self.status);
        self.status.set(0, false);
        self.send_status_bits(self.status);
    }

    /// Send the Real Time Clock to the core.
    pub fn send_rtc(&mut self) -> Result<(), String> {
        self.fpga.spi_mut().execute(UserIoRtc::now())?;
        Ok(())
    }

    pub fn send_volume(&mut self, volume: impl IntoVolume) -> Result<(), String> {
        let volume = volume.into_volume();
        debug!(?volume, "Setting volume");
        self.fpga.spi_mut().execute(volume.into_user_io())?;
        Ok(())
    }

    /// Send a file (ROM or BIOS) to the core on an index.
    pub fn load_file(
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

        let f = File::open(path).map_err(|e| e.to_string())?;
        let size = f.metadata().map_err(|e| e.to_string())?.len() as u32;

        self.start_send_file(info.index(), &ext, size)?;
        match info {
            MisterFpgaSendFileInfo::Memory { index, address } => {
                trace!(?index, ?address, ?ext, ?size, "File info (memory)");
                self.send_file_to_sdram_(size, address, f)?;
            }
            MisterFpgaSendFileInfo::Buffered { index } => {
                trace!(?index, ?ext, ?size, "File info (buffered)");
                self.send_file_to_buffer_(size, f)?;
            }
        }
        self.read_status_bits();

        self.status.set(0, false);
        self.send_status_bits(self.status);

        // self.end_send_file()?;
        debug!("Done in {}ms", now.elapsed().as_millis());

        Ok(())
    }

    fn start_send_file(&mut self, index: u8, ext: &str, size: u32) -> Result<(), String> {
        self.fpga.spi_mut().execute(FileIndex::from(index))?;
        self.fpga.spi_mut().execute(FileExtension(ext))?;
        self.fpga.spi_mut().execute(FileTxEnabled(Some(size)))
    }

    pub fn end_send_file(&mut self) -> Result<(), String> {
        // Disable download.
        self.fpga.spi_mut().execute(FileTxDisabled)
    }

    /// Return the core parsed config structure.
    pub fn config(&self) -> &config_string::Config {
        &self.config
    }

    /// Return the video info of the core.
    pub fn video_info(&mut self) -> Result<VideoInfo, String> {
        VideoInfo::create(self.spi_mut())
    }

    // TODO: rethink how framebuffers are handled.
    pub fn send_to_menu_framebuffer(&mut self, bytes: &[u8]) -> Result<(), String> {
        const FB_BASE: usize = 0x20000000 + (32 * 1024 * 1024);

        let fb_addr = FB_BASE + (1920 * 1080) * 4;
        let mut mapper = DevMemMemoryMapper::create(fb_addr, 1920 * 1080 * 4).unwrap();
        mapper.as_mut_range(..bytes.len()).copy_from_slice(bytes);
        Ok(())
    }

    /// Return the core status bits. This is an internal cache of the
    /// status bits. Use `[Core::read_status_bits()]` to read the status
    /// from the core.
    pub fn status_bits(&self) -> &StatusBitMap {
        &self.status
    }

    /// Update the internal cache and return it.
    pub fn read_status_bits(&mut self) -> &StatusBitMap {
        self.fpga
            .spi_mut()
            .execute(GetStatusBits(&mut self.status, &mut self.status_counter))
            .unwrap();
        &self.status
    }

    /// Send status bits to the core.
    pub fn send_status_bits(&mut self, bits: StatusBitMap) {
        debug!(?bits, "Setting status bits");
        self.fpga.spi_mut().execute(SetStatusBits(&bits)).unwrap();
        self.status = bits;
    }

    /// Notify the core of a keyboard key down event.
    pub fn key_down(&mut self, keycode: impl Into<Ps2Scancode> + Debug + Copy) {
        let scancode = keycode.into();
        debug!(?keycode, ?scancode, "Keydown");
        if scancode != Ps2Scancode::None {
            self.fpga
                .spi_mut()
                .execute(UserIoKeyboardKeyDown::from(scancode))
                .unwrap();
        }
    }

    /// Notify the core of a keyboard key up event.
    pub fn key_up(&mut self, keycode: impl Into<Ps2Scancode> + Debug + Copy) {
        let scancode = keycode.into();
        debug!(?keycode, ?scancode, "Keyup");
        if scancode != Ps2Scancode::None {
            self.fpga
                .spi_mut()
                .execute(UserIoKeyboardKeyUp::from(scancode))
                .unwrap();
        }
    }

    pub fn gamepad(&self, idx: u8) -> Option<&ButtonMap> {
        self.gamepads.get(idx as usize)
    }

    pub fn gamepad_mut(&mut self, idx: u8) -> Option<&mut ButtonMap> {
        self.gamepads.get_mut(idx as usize)
    }

    pub fn send_gamepad(&mut self, idx: u8, map: ButtonMap) {
        if idx > 5 {
            return;
        }

        self.fpga
            .spi_mut()
            .execute(UserIoJoystick::from_joystick_index(idx, &map))
            .unwrap();
        self.gamepads[idx as usize] = map;
    }

    /// Notify the core of a gamepad button down event.
    pub fn gamepad_button_down(&mut self, joystick_idx: u8, button: u8) {
        let g = &mut self.gamepads[joystick_idx as usize];
        g.down(button);

        self.fpga
            .spi_mut()
            .execute(UserIoJoystick::from_joystick_index(joystick_idx, g))
            .unwrap();
    }

    /// Notify the core of a gamepad button up event.
    pub fn gamepad_button_up(&mut self, joystick_idx: u8, button: u8) {
        let g = &mut self.gamepads[joystick_idx as usize];
        g.up(button);

        self.fpga
            .spi_mut()
            .execute(UserIoJoystick::from_joystick_index(joystick_idx, g))
            .unwrap();
    }

    /// Access the internal save state manager, in readonly.
    pub fn save_states(&self) -> Option<&SaveStateManager<DevMemMemoryMapper>> {
        self.save_states.as_ref()
    }

    /// Access the internal save state manager.
    pub fn save_states_mut(&mut self) -> Option<&mut SaveStateManager<DevMemMemoryMapper>> {
        self.save_states.as_mut()
    }

    /// Take a screenshot and return the image in memory.
    pub fn take_screenshot(&mut self) -> Result<DynamicImage, String> {
        self.framebuffer.take_screenshot()
    }

    /// Mount an SD card to the core.
    pub fn mount(&mut self, file: SdCard, index: u8) -> Result<(), String> {
        self.fpga.spi_mut().execute(
            SetSdConf::default()
                .with_wide(self.spi_type.is_wide())
                .with_size(file.size()),
        )?;

        self.fpga
            .spi_mut()
            .execute(SetSdInfo::from(&file).with_io_version(self.io_version))?;

        // Notify the core of the SD card update.
        self.fpga.spi_mut().execute(
            SetSdStat::default()
                .with_writable(file.writeable())
                .with_index(index),
        )?;

        info!(?file, index, "Mounted SD Card");
        self.cards[index as usize] = Some(file);

        Ok(())
    }

    /// Check for updates (read/write) to SD cards. Returns true if any write/read
    /// operations were requested by the core (which means there might be more).
    pub fn poll_mounts(&mut self) -> Result<bool, String> {
        let mut result = false;

        for (_i, card) in self
            .cards
            .iter_mut()
            .enumerate()
            .filter_map(|(i, c)| c.as_mut().map(|c| (i, c)))
        {
            let mut stat: SdStatOutput = Default::default();
            self.fpga.spi_mut().execute(GetSdStat(&mut stat))?;
            trace!(?stat, "SD stat");

            if stat.op.is_write() {
                result = true;
                let mut buffer = vec![0; stat.size];

                self.fpga.spi_mut().execute(SdWrite::new(
                    &mut buffer,
                    self.spi_type.is_wide(),
                    stat.ack,
                ))?;

                let addr = stat.lba * stat.block_size as u64;
                let io = card.as_io();
                io.seek(SeekFrom::Start(addr)).map_err(|e| e.to_string())?;
                io.write_all(&buffer).map_err(|e| e.to_string())?;
            } else if stat.op.is_read() {
                result = true;
                let mut buffer = vec![0; stat.size];
                let addr = stat.lba * stat.block_size as u64;
                let io = card.as_io();
                io.seek(SeekFrom::Start(addr)).map_err(|e| e.to_string())?;
                io.read_exact(&mut buffer).map_err(|e| e.to_string())?;

                // Blocks are now in memory, send them to the core.
                self.fpga.spi_mut().execute(SdRead::new(
                    &buffer,
                    self.spi_type.is_wide(),
                    stat.ack,
                ))?;
            }
        }
        Ok(result)
    }

    fn send_file_to_sdram_(
        &mut self,
        size: u32,
        address: FpgaRamMemoryAddress,
        mut reader: impl Read,
    ) -> Result<(), String> {
        // Verify invariants.
        if size >= 0x2000_0000 {
            return Err("File too large.".to_string());
        }
        let mut crc = crc32fast::Hasher::new();
        let mut mem = DevMemMemoryMapper::create(address.as_usize(), size as usize)?;

        let mut bytes2send = size;
        while bytes2send > 0 {
            let sz = reader
                .read(mem.as_mut_range(..))
                .map_err(|e| e.to_string())?;

            // crc.update(mem.as_range(start..start + len));
            bytes2send -= sz as u32;
        }
        crc.update(mem.as_range(..));

        let crc = crc.finalize();
        debug!("CRC: {:08X}", crc);
        Ok(())
    }

    fn send_file_to_buffer_(&mut self, size: u32, mut reader: impl Read) -> Result<(), String> {
        // Verify invariants.
        if size >= 0x2000_0000 {
            return Err("File too large.".to_string());
        }

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
                                .execute(FileTxData8Bits(&buffer[..size]))?;
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
                                .execute(FileTxData16Bits(&buf16[..size / 2]))?;
                        }
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        debug!("Read {} bytes", size);
        trace!("Took {}ms", now.elapsed().as_millis());
        let crc = crc.finalize();
        debug!("CRC: {:08X}", crc);

        Ok(())
    }
}

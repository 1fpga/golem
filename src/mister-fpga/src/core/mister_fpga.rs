use std::any::Any;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::SystemTime;

use image::DynamicImage;
use tracing::{debug, info, trace};

use cyclone_v::memory::{DevMemMemoryMapper, MemoryMapper};
use one_fpga::core::{Bios, CoreSettings, Error, MountedFile, Rom, SaveState, SettingId};
use one_fpga::inputs::gamepad::ButtonSet;
use one_fpga::inputs::keyboard::ScancodeSet;
use one_fpga::inputs::{Button, Scancode};
use one_fpga::Core;

use crate::config::{Config, HdmiLimitedConfig, VgaMode};
use crate::config_string;
use crate::config_string::{ConfigMenu, FpgaRamMemoryAddress, LoadFileInfo};
use crate::core::buttons::ButtonMap;
use crate::core::file::SdCard;
use crate::core::video;
use crate::core::video::VideoInfo;
use crate::core::volume::{IntoVolume, Volume};
use crate::fpga::file_io::{
    FileExtension, FileIndex, FileTxData16Bits, FileTxData8Bits, FileTxDisabled, FileTxEnabled,
};
use crate::fpga::user_io::{
    ButtonSwitches, GetSdStat, GetStatusBits, SdRead, SdStatOutput, SdWrite, SetSdConf, SetSdInfo,
    SetSdStat, SetStatusBits, UserIoButtonSwitch, UserIoJoystick, UserIoKeyboardKeyDown,
    UserIoKeyboardKeyUp, UserIoRtc,
};
use crate::fpga::{user_io, CoreInterfaceType, CoreType, MisterFpga};
use crate::keyboard::Ps2Scancode;
use crate::savestate::SaveStateManager;
use crate::types::StatusBitMap;

#[derive(Debug)]
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
    pub is_menu: bool,
    pub core_type: CoreType,
    pub spi_type: CoreInterfaceType,
    pub io_version: u8,
    config: config_string::Config,

    volume: u8,

    // All the images that are mounted. Can only have 16 images at once.
    cards: Box<[Option<SdCard>; 16]>,

    save_states: Option<SaveStateManager<DevMemMemoryMapper>>,
    gamepads: [ButtonMap; 6],

    status: StatusBitMap,
    status_counter: u8,

    framebuffer: crate::framebuffer::FpgaFramebuffer<DevMemMemoryMapper>,

    // A cache for the video_info.
    video_info: Option<VideoInfo>,

    // Whether we should quit.
    should_quit: bool,
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
        info!("Core config {:#?}", config);

        let core_type = fpga.core_type().ok_or("Could not get core type.")?;
        let spi_type = fpga
            .core_interface_type()
            .ok_or("Could not get SPI type.")?;
        let io_version = fpga.core_io_version().ok_or("Could not get IO version.")?;
        info!(?core_type, ?spi_type, io_version, "Core loaded");

        let save_states = SaveStateManager::from_config_string(&config);
        const NONE: Option<SdCard> = None;

        Ok(MisterFpgaCore {
            is_menu: false,
            fpga,
            core_type,
            spi_type,
            io_version,
            volume: 0,
            config,
            cards: Box::new([NONE; 16]),
            save_states,
            gamepads: [map; 6],
            status: Default::default(),
            status_counter: 0,
            framebuffer: crate::framebuffer::FpgaFramebuffer::default(),
            video_info: None,
            should_quit: false,
        })
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

    pub fn frame_iter(&mut self) -> crate::framebuffer::FrameIter {
        self.framebuffer.update_type_from_core();
        crate::framebuffer::FrameIter::new(&self.framebuffer)
    }

    /// Send a file (ROM or BIOS) to the core on an index.
    pub fn load_file(
        &mut self,
        path: &Path,
        file_info: Option<LoadFileInfo>,
    ) -> Result<(), String> {
        info!(?path, ?file_info, "Loading file");
        let info = file_info.map_or_else(
            || MisterFpgaSendFileInfo::from_path(path, self),
            MisterFpgaSendFileInfo::from_file_info,
        )?;
        info!(?info, "info_send_file_info");

        let ext = path
            .extension()
            .unwrap_or(OsStr::new(""))
            .to_str()
            .unwrap_or("")
            .to_uppercase();

        let now = std::time::Instant::now();
        debug!("Sending file {:?} to core", path);

        let file = File::open(path).map_err(|e| e.to_string())?;
        let size = file.metadata().map_err(|e| e.to_string())?.len() as u32;

        self.start_send_file(info.index(), &ext, size)?;
        match info {
            MisterFpgaSendFileInfo::Memory { index, address } => {
                trace!(?index, ?address, ?ext, ?size, "File info (memory)");
                self.send_file_to_sdram_(size, address, file)?;
            }
            MisterFpgaSendFileInfo::Buffered { index } => {
                trace!(?index, ?ext, ?size, "File info (buffered)");
                self.send_file_to_buffer_(size, file)?;
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
        if let Some(video_info) = self.video_info {
            return Ok(video_info);
        }

        let video_info = VideoInfo::create(self.spi_mut())?;
        self.video_info = Some(video_info);
        Ok(video_info)
    }

    pub fn status_mask(&self) -> StatusBitMap {
        self.config().status_bit_map_mask()
    }

    pub fn status_pulse(&mut self, bit: usize) {
        let mut bits = *self.status_bits();
        bits.set(bit, true);
        self.send_status_bits(bits);

        bits.set(bit, false);
        self.send_status_bits(bits);
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

    pub fn menu_options(&self) -> &[ConfigMenu] {
        self.config().menu.as_slice()
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
    pub fn take_screenshot(&self) -> Result<DynamicImage, String> {
        self.framebuffer.take_screenshot()
    }

    pub fn framebuffer(&self) -> &crate::framebuffer::FpgaFramebuffer<DevMemMemoryMapper> {
        &self.framebuffer
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

    pub fn trigger_menu(&mut self, menu: &ConfigMenu) -> Result<bool, String> {
        match menu {
            ConfigMenu::HideIf(cond, sub) | ConfigMenu::DisableIf(cond, sub) => {
                if self.config().status_bit_map_mask().get(*cond as usize) {
                    self.trigger_menu(sub)
                } else {
                    Err("Cannot trigger menu".to_string())
                }
            }
            ConfigMenu::HideUnless(cond, sub) | ConfigMenu::DisableUnless(cond, sub) => {
                if !self.config().status_bit_map_mask().get(*cond as usize) {
                    self.trigger_menu(sub)
                } else {
                    Err("Cannot trigger menu".to_string())
                }
            }
            ConfigMenu::Option { bits, choices, .. } => {
                let (from, to) = (bits.start, bits.end);
                let mut bits = *self.status_bits();
                let max = choices.len();
                let value = bits.get_range(from..to) as usize;
                bits.set_range(from..to, ((value + 1) % max) as u32);
                self.send_status_bits(bits);
                Ok(true)
            }
            ConfigMenu::Trigger { index, .. } => {
                self.status_pulse(*index as usize);
                Ok(true)
            }
            ConfigMenu::PageItem(_, sub) => self.trigger_menu(sub),

            // TODO: see if we can implement more (like Load File).
            _ => Ok(false),
        }
    }
}

impl Core for MisterFpgaCore {
    fn init(&mut self) -> Result<(), Error> {
        self.soft_reset();
        self.fpga
            .spi_mut()
            .execute(user_io::SetMemorySize::from_fpga().unwrap())
            .map_err(Error::Message)?;

        // Initialize the framebuffer.
        if !self.is_menu {
            self.fpga
                .spi_mut()
                .execute(user_io::SetFramebufferToCore)
                .map_err(Error::Message)?;
        }

        let options = Config::base().into_inner();

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

        self.spi_mut().execute(switches)?;

        video::init(&options);
        video::init_mode(&options, self, self.is_menu);
        self.framebuffer.update_type_from_core();

        Ok(())
    }

    fn name(&self) -> &str {
        self.config.name.as_str()
    }

    fn reset(&mut self) -> Result<(), Error> {
        self.soft_reset();
        Ok(())
    }

    fn volume(&self) -> Result<u8, Error> {
        Ok(self.volume)
    }

    fn set_volume(&mut self, volume: u8) -> Result<(), Error> {
        self.volume = volume;
        self.send_volume(Volume::scaled(volume))
            .map_err(Error::Message)?;
        Ok(())
    }

    fn set_rtc(&mut self, time: SystemTime) -> Result<(), Error> {
        self.fpga
            .spi_mut()
            .execute(UserIoRtc::from(time))
            .map_err(Error::Message)?;
        Ok(())
    }

    fn screenshot(&self) -> Result<DynamicImage, Error> {
        self.take_screenshot().map_err(Error::Message)
    }

    fn save_state_mut(&mut self, slot: usize) -> Result<Option<&mut dyn SaveState>, Error> {
        let manager = self.save_states_mut();
        if let Some(manager) = manager {
            let slots = manager.slots_mut();
            if slot >= slots.len() {
                Ok(None)
            } else {
                Ok(Some(&mut slots[slot]))
            }
        } else {
            Ok(None)
        }
    }

    fn save_state(&self, slot: usize) -> Result<Option<&dyn SaveState>, Error> {
        let manager = self.save_states();
        if let Some(manager) = manager {
            let slots = manager.slots();
            if slot >= slots.len() {
                Ok(None)
            } else {
                Ok(Some(&slots[slot]))
            }
        } else {
            Ok(None)
        }
    }

    fn mounted_file_mut(&mut self, slot: usize) -> Result<Option<&mut dyn MountedFile>, Error> {
        match self.cards.get_mut(slot) {
            Some(Some(card)) => Ok(Some(card.as_mounted())),
            _ => Ok(None),
        }
    }

    fn send_rom(&mut self, rom: Rom) -> Result<(), Error> {
        match rom {
            Rom::Memory(_, _) => Err(Error::Message(
                "Memory ROMs are not supported yet.".to_string(),
            )),
            Rom::File(path) => self.load_file(&path, None).map_err(Error::Message),
        }
    }

    fn send_bios(&mut self, _bios: Bios) -> Result<(), Error> {
        todo!()
    }

    fn key_up(&mut self, key: Scancode) -> Result<(), Error> {
        self.key_up(key);
        Ok(())
    }

    fn key_down(&mut self, key: Scancode) -> Result<(), Error> {
        self.key_down(key);
        Ok(())
    }

    fn keys_set(&mut self, _keys: ScancodeSet) -> Result<(), Error> {
        todo!()
    }

    fn keys(&self) -> Result<ScancodeSet, Error> {
        todo!()
    }

    fn gamepad_button_up(&mut self, index: usize, button: Button) -> Result<(), Error> {
        self.gamepad_button_up(index as u8, button as u8);
        Ok(())
    }

    fn gamepad_button_down(&mut self, index: usize, button: Button) -> Result<(), Error> {
        self.gamepad_button_down(index as u8, button as u8);
        Ok(())
    }

    fn gamepad_buttons_set(&mut self, _index: usize, _buttons: ButtonSet) -> Result<(), Error> {
        todo!()
    }

    fn gamepad_buttons(&self, _index: usize) -> Result<Option<ButtonSet>, Error> {
        todo!()
    }

    fn settings(&self) -> Result<CoreSettings, Error> {
        Ok(self.config.as_core_settings(self.status_bits()))
    }

    fn trigger(&mut self, id: SettingId) -> Result<(), Error> {
        if let Some(ConfigMenu::Trigger { index, .. }) = self
            .menu_options()
            .iter()
            .filter_map(ConfigMenu::as_trigger)
            .find(|item| item.setting_id() == Some(id))
        {
            self.status_pulse(*index as usize);
        }

        Ok(())
    }

    fn file_select(&mut self, id: SettingId, path: String) -> Result<(), Error> {
        if let Some(info) = self
            .menu_options()
            .iter()
            .filter_map(ConfigMenu::as_load_file_info)
            .find(|info| info.setting_id() == id)
            .cloned()
        {
            self.load_file(&Path::new(&path), Some(info))
                .map_err(Error::Message)?;
            self.end_send_file()?;
            self.poll_mounts()?;
        }
        Ok(())
    }

    fn int_option(&mut self, id: SettingId, value: u32) -> Result<u32, Error> {
        if let Some(ConfigMenu::Option { bits, choices, .. }) = self
            .menu_options()
            .iter()
            .filter_map(ConfigMenu::as_option)
            .find(|item| item.setting_id() == Some(id))
        {
            let (from, to) = (bits.start, bits.end);
            let mut bits = *self.status_bits();
            let max = choices.len();
            bits.set_range(from..to, (value as usize % max) as u32);
            let new_value = bits.get_range(from..to);
            self.send_status_bits(bits);
            Ok(new_value)
        } else {
            Ok(0)
        }
    }

    fn bool_option(&mut self, id: SettingId, value: bool) -> Result<bool, Error> {
        if let Some(ConfigMenu::Option { bits, .. }) = self
            .menu_options()
            .iter()
            .filter_map(ConfigMenu::as_option)
            .find(|item| item.setting_id() == Some(id))
        {
            let (from, to) = (bits.start, bits.end);
            let mut bits = *self.status_bits();
            bits.set_range(from..to, if value { 1 } else { 0 });
            let new_value = bits.get_range(from..to) != 0;
            self.send_status_bits(bits);
            Ok(new_value)
        } else {
            Ok(false)
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn quit(&mut self) {
        self.should_quit = true;
    }

    fn should_quit(&self) -> bool {
        self.should_quit
    }
}

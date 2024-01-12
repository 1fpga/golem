use crate::core::buttons::ButtonMap;
use crate::core::file::SdCard;
use crate::fpga::feature::SpiFeatureSet;
use crate::fpga::{IntoLowLevelSpiCommand, SpiCommand, SpiCommandExt};
use crate::keyboard::Ps2Scancode;
use crate::types::StatusBitMap;
use bitfield::bitfield;
use chrono::{Datelike, NaiveDateTime, Timelike};
use std::mem::transmute;
use std::time::SystemTime;
use tracing::trace;

/// User IO commands.
#[derive(Debug, Clone, Copy, PartialEq, strum::Display)]
enum UserIoCommands {
    // UserIoStatus = 0x00,
    // UserIoButtonSwitch = 0x01,
    UserIoJoystick0 = 0x02,
    UserIoJoystick1 = 0x03,
    // UserIoMouse = 0x04,
    UserIoKeyboard = 0x05,
    // UserIoKeyboardOsd = 0x06,
    UserIoJoystick2 = 0x10,
    UserIoJoystick3 = 0x11,
    UserIoJoystick4 = 0x12,
    UserIoJoystick5 = 0x13,

    UserIoGetString = 0x14,

    // Unused, reserved.
    #[allow(dead_code)]
    UserIoSetStatus = 0x15,

    /// Read status of sd card emulation
    UserIoGetSdStat = 0x16,

    UserIoSetSdConf = 0x19,

    /// Set sd card status
    UserIoSetSdStat = 0x1C,

    /// Send info about mounted image
    UserIoSetSdInfo = 0x1D,

    UserIoSetStatus32Bits = 0x1E,

    /// Transmit RTC (time struct, including seconds) to the core.
    UserIoRtc = 0x22,

    UserIoGetStatusBits = 0x29,
}

impl IntoLowLevelSpiCommand for UserIoCommands {
    #[inline]
    fn into_ll_spi_command(self) -> (SpiFeatureSet, u16) {
        (SpiFeatureSet::IO, self as u16)
    }
}

enum UserIoSectorRead {
    /// Read a sector including a ACK.
    Read(u16),

    /// Write a sector.
    Write(u16),
}

impl IntoLowLevelSpiCommand for UserIoSectorRead {
    #[inline]
    fn into_ll_spi_command(self) -> (SpiFeatureSet, u16) {
        (
            SpiFeatureSet::IO,
            match self {
                UserIoSectorRead::Read(ack) => 0x17 | ack,
                UserIoSectorRead::Write(ack) => 0x18 | ack,
            },
        )
    }
}

pub struct UserIoJoystick(u8, u32);

impl SpiCommand for UserIoJoystick {
    #[inline]
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        let command = match self.0 {
            0 => UserIoCommands::UserIoJoystick0,
            1 => UserIoCommands::UserIoJoystick1,
            2 => UserIoCommands::UserIoJoystick2,
            3 => UserIoCommands::UserIoJoystick3,
            4 => UserIoCommands::UserIoJoystick4,
            5 => UserIoCommands::UserIoJoystick5,
            _ => unreachable!(),
        };

        spi.command(command)
            .write(self.1 as u16)
            .write_nz((self.1 >> 16) as u16);

        Ok(())
    }
}

impl UserIoJoystick {
    #[inline]
    pub fn from_joystick_index(index: u8, map: &ButtonMap) -> Self {
        if index > 5 {
            panic!("Invalid joystick index");
        }

        Self(index, map.value())
    }
}

pub struct UserIoKeyboardKeyDown(u32);

impl From<Ps2Scancode> for UserIoKeyboardKeyDown {
    fn from(value: Ps2Scancode) -> Self {
        Self(value.as_u32())
    }
}

impl From<u32> for UserIoKeyboardKeyDown {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl SpiCommand for UserIoKeyboardKeyDown {
    #[inline]
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        spi.command(UserIoCommands::UserIoKeyboard)
            .write_cond_b(self.0 & 0x080000 != 0, 0xE0)
            .write_b((self.0 & 0xFF) as u8);

        Ok(())
    }
}

pub struct UserIoKeyboardKeyUp(u32);

impl From<Ps2Scancode> for UserIoKeyboardKeyUp {
    fn from(value: Ps2Scancode) -> Self {
        Self(value.as_u32())
    }
}

impl From<u32> for UserIoKeyboardKeyUp {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl SpiCommand for UserIoKeyboardKeyUp {
    #[inline]
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        spi.command(UserIoCommands::UserIoKeyboard)
            .write_b(0xF0)
            .write_b(self.0 as u8);

        Ok(())
    }
}

pub struct UserIoGetString<'a>(pub &'a mut String);

impl SpiCommand for UserIoGetString<'_> {
    #[inline]
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        let mut command = spi.command(UserIoCommands::UserIoGetString);

        let mut i = 0;
        loop {
            command.write_read_b(0, &mut i);
            if i == 0 || i > 127 {
                break;
            }
            self.0.push(i as char);
        }

        Ok(())
    }
}

/// Send the current system time to the core.
pub struct UserIoRtc(pub NaiveDateTime);

impl From<NaiveDateTime> for UserIoRtc {
    fn from(value: NaiveDateTime) -> Self {
        Self(value)
    }
}

impl From<SystemTime> for UserIoRtc {
    fn from(value: SystemTime) -> Self {
        let t = value.duration_since(std::time::UNIX_EPOCH).unwrap();
        let t = NaiveDateTime::from_timestamp_opt(t.as_secs() as i64, t.subsec_nanos()).unwrap();
        t.into()
    }
}

impl SpiCommand for UserIoRtc {
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        // MSM6242B layout, with 4 bits per digit of sec, min, hour, day, month, year (2 digits),
        // and the weekday.
        let mut rtc = [
            ((self.0.second() % 10) | (self.0.second() / 10) << 4) as u8,
            ((self.0.minute() % 10) | (self.0.minute() / 10) << 4) as u8,
            ((self.0.hour() % 10) | (self.0.hour() / 10) << 4) as u8,
            ((self.0.day() % 10) | (self.0.day() / 10) << 4) as u8,
            ((self.0.month() % 10) | (self.0.month() / 10) << 4) as u8,
            ((self.0.year() % 10) | ((self.0.year() / 10) % 10) << 4) as u8,
            self.0.weekday().num_days_from_sunday() as u8,
            0x40,
        ];

        spi.command(UserIoCommands::UserIoRtc)
            .write_buffer_b(&mut rtc);

        Ok(())
    }
}

impl UserIoRtc {
    pub fn now() -> Self {
        chrono::Local::now().naive_local().into()
    }
}

/// Transmit seconds since Unix epoch.
pub struct Timestamp(NaiveDateTime);

impl From<NaiveDateTime> for Timestamp {
    fn from(value: NaiveDateTime) -> Self {
        Self(value)
    }
}

impl From<SystemTime> for Timestamp {
    fn from(value: SystemTime) -> Self {
        let t = value.duration_since(std::time::UNIX_EPOCH).unwrap();
        let t = NaiveDateTime::from_timestamp_opt(t.as_secs() as i64, t.subsec_nanos()).unwrap();
        t.into()
    }
}

impl SpiCommand for Timestamp {
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        spi.command(UserIoCommands::UserIoRtc)
            .write(self.0.timestamp() as u16)
            .write((self.0.timestamp() >> 16) as u16);

        Ok(())
    }
}

/// Get the status bits.
pub struct GetStatusBits<'a>(pub &'a mut StatusBitMap, pub &'a mut u8);

impl SpiCommand for GetStatusBits<'_> {
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        let mut stchg = 0;
        let mut command = spi.command_read(UserIoCommands::UserIoGetStatusBits, &mut stchg);

        if ((stchg & 0xF0) == 0xA0) && (stchg as u8 & 0x0F) != *self.1 {
            *self.1 = stchg as u8 & 0x0F;
            for word in self.0.as_mut_raw_slice() {
                command.write_read(0u16, word);
            }

            drop(command);

            self.0.set(0, false);
            SetStatusBits(self.0).execute(spi)?;
        }

        Ok(())
    }
}

/// Send the status bits.
pub struct SetStatusBits<'a>(pub &'a StatusBitMap);

impl SpiCommand for SetStatusBits<'_> {
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        let bits16 = self.0.as_raw_slice();

        spi.command(UserIoCommands::UserIoSetStatus32Bits)
            .write_buffer_w(&bits16[0..4])
            .write_buffer_cond_w(self.0.has_extra(), &bits16[4..]);
        Ok(())
    }
}

const CSD: [u8; 16] = [
    0xf1, 0x40, 0x40, 0x0a, 0x80, 0x7f, 0xe5, 0xe9, 0x00, 0x00, 0x59, 0x5b, 0x32, 0x00, 0x0e, 0x40,
];
const CID: [u8; 16] = [
    0x3e, 0x00, 0x00, 0x34, 0x38, 0x32, 0x44, 0x00, 0x00, 0x73, 0x2f, 0x6f, 0x93, 0x00, 0xc7, 0xcd,
];

/// Send SD card configuration (CSD, CID).
pub struct SetSdConf {
    wide: bool,
    csd: [u8; 16],
    cid: [u8; 16],
}

impl SpiCommand for SetSdConf {
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        let mut command = spi.command(UserIoCommands::UserIoSetSdConf);

        if self.wide {
            command.write_buffer_w(unsafe { transmute::<_, &[u16; 8]>(&self.csd) });
            command.write_buffer_w(unsafe { transmute::<_, &[u16; 8]>(&self.cid) });
        } else {
            command.write_buffer_b(&self.csd);
            command.write_buffer_b(&self.cid);
        }

        // SDHC permanently.
        command.write_b(1);

        Ok(())
    }
}

impl Default for SetSdConf {
    fn default() -> Self {
        Self {
            wide: false,
            csd: CSD,
            cid: CID,
        }
    }
}

impl SetSdConf {
    pub const fn with_wide(self, wide: bool) -> Self {
        Self { wide, ..self }
    }

    pub const fn with_size(self, size: u64) -> Self {
        let mut csd = self.csd;
        let cid = self.cid;

        let size = size as u32;
        csd[6] = (size >> 16) as u8;
        csd[7] = (size >> 8) as u8;
        csd[8] = size as u8;

        Self {
            wide: self.wide,
            csd,
            cid,
        }
    }
}

#[derive(Default, Debug)]
pub struct SetSdInfo {
    io_version: u8,
    size: u64,
}

impl From<&SdCard> for SetSdInfo {
    fn from(value: &SdCard) -> Self {
        Self {
            io_version: 0,
            size: value.size(),
        }
    }
}

impl SpiCommand for SetSdInfo {
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        let mut command = spi.command(UserIoCommands::UserIoSetSdInfo);

        trace!(?self, "SetSdInfo");

        if self.io_version != 0 {
            command
                .write(self.size as u16)
                .write((self.size >> 16) as u16)
                .write((self.size >> 32) as u16)
                .write((self.size >> 48) as u16);
        } else {
            command
                .write_b(self.size as u8)
                .write_b((self.size >> 8) as u8)
                .write_b((self.size >> 16) as u8)
                .write_b((self.size >> 24) as u8)
                .write_b((self.size >> 32) as u8)
                .write_b((self.size >> 40) as u8)
                .write_b((self.size >> 48) as u8)
                .write_b((self.size >> 56) as u8);
        }

        Ok(())
    }
}

impl SetSdInfo {
    pub const fn with_io_version(self, io_version: u8) -> Self {
        Self { io_version, ..self }
    }

    pub const fn with_size(self, size: u64) -> Self {
        Self { size, ..self }
    }
}

#[derive(Default, Debug)]
pub struct SetSdStat {
    writable: bool,
    index: u8,
}

impl SpiCommand for SetSdStat {
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        spi.command(UserIoCommands::UserIoSetSdStat)
            .write_b((1 << self.index) | if self.writable { 0 } else { 0x80 });
        Ok(())
    }
}

impl SetSdStat {
    pub const fn with_writable(self, writable: bool) -> Self {
        Self { writable, ..self }
    }

    pub const fn with_index(self, index: u8) -> Self {
        Self { index, ..self }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, strum::FromRepr)]
#[repr(u8)]
pub enum SdOp {
    #[default]
    Noop = 0,
    Read = 1,
    Write = 2,
    ReadWrite = 3,
}

impl From<bool> for SdOp {
    fn from(value: bool) -> Self {
        if !value {
            Self::Read
        } else {
            Self::Write
        }
    }
}

impl From<u8> for SdOp {
    fn from(value: u8) -> Self {
        Self::from_repr(value).unwrap()
    }
}

impl From<SdOp> for u8 {
    fn from(value: SdOp) -> Self {
        value as u8
    }
}

impl SdOp {
    #[inline]
    pub fn is_read(&self) -> bool {
        (*self as u8) & 1 != 0
    }

    #[inline]
    pub fn is_write(&self) -> bool {
        *self == SdOp::Write
    }
}

bitfield! {
    struct SdStatus(u16);
    impl Debug;

    u8;

    /// The semantics of this bit are unknown, but it's used to separate
    /// between 2 code paths.
    pub check, set_check: 15;

    /// Number of blocks to read, minus 1.
    block_count_, _: 14, 9;

    /// The size of a block, by 128 bytes.
    block_size_, _: 8, 6;

    /// The disk to read them from.
    pub disk, set_disk: 6, 2;

    /// The operation (read/write).
    pub from into SdOp, op, set_op: 2, 0;
}

impl SdStatus {
    pub fn block_count(&self) -> u32 {
        self.block_count_() as u32 + 1
    }

    /// This is the block size, between 128B and 16KiB.
    pub fn block_size(&self) -> usize {
        128 << self.block_size_() as usize
    }
}

#[derive(Default, Debug)]
pub struct SdStatOutput {
    pub disk: u8,
    pub op: SdOp,
    pub ack: u16,
    pub lba: u64,
    pub size: usize,
    pub block_count: u32,
    pub block_size: usize,
}

pub struct GetSdStat<'a>(pub &'a mut SdStatOutput);

impl SpiCommand for GetSdStat<'_> {
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        let mut c = 0;
        let mut command = spi.command_read(UserIoCommands::UserIoGetSdStat, &mut c);

        let status = SdStatus(c);

        if status.check() {
            self.0.disk = status.disk();
            self.0.ack = (self.0.disk as u16) << 8;
            self.0.op = status.op();

            (self.0.block_count, self.0.block_size) = (status.block_count(), status.block_size());

            command.write(0u16);
            self.0.lba = ((command.write_get(0u16) as u32)
                | ((command.write_get(0u16) as u32) << 16)) as u64;
        } else {
            let c = command.write_get(0u16);

            // TODO: WTF.
            if !((c & 0xF0) == 0x50 && (c & 0x3F03) != 0) {
                return Ok(());
            }

            self.0.lba =
                (((command.write_get(0u16) as u32) << 16) | command.write_get(0u16) as u32) as u64;

            // Check if the core requests the configuration.
            if c & 0x0C == 0x0C {
                command.execute(SetSdConf::default())?;
            }

            // Figure out which disk to update.
            // TODO: this is a bit hacky, but it works. Clean up at some point.
            (self.0.disk, self.0.op) = if c & 0x0003 != 0 {
                (0, SdOp::from(c & 1 == 0))
            } else if c & 0x0900 != 0 {
                (1, SdOp::from(c & 0x0100 == 0))
            } else if c & 0x1200 != 0 {
                (2, SdOp::from(c & 0x0200 == 0))
            } else if c & 0x2400 != 0 {
                (3, SdOp::from(c & 0x0400 == 0))
            } else {
                return Err(format!("Invalid status: {:04X}", c));
            };

            self.0.ack = if c & 4 != 0 {
                0
            } else {
                (self.0.disk as u16 + 1) << 8
            };
            self.0.block_size = 512;
            self.0.block_count = 1;
        }
        self.0.size = self.0.block_size * self.0.block_count as usize;
        Ok(())
    }
}

pub struct SdRead<'a> {
    data: &'a [u8],
    wide: bool,
    ack: u16,
}

impl SpiCommand for SdRead<'_> {
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        let mut command = spi.command(UserIoSectorRead::Read(self.ack));

        if self.wide {
            command.write_buffer_w(unsafe { transmute::<_, &[u16]>(self.data) });
        } else {
            command.write_buffer_b(self.data);
        }
        Ok(())
    }
}

impl<'a> SdRead<'a> {
    pub fn new(data: &'a [u8], wide: bool, ack: u16) -> Self {
        Self { data, wide, ack }
    }
}

pub struct SdWrite<'a> {
    data: &'a mut Vec<u8>,
    wide: bool,
    ack: u16,
}

impl SpiCommand for SdWrite<'_> {
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        let mut command = spi.command(UserIoSectorRead::Write(self.ack));

        if self.wide {
            command.read_buffer_w(unsafe { transmute::<_, &mut [u16]>(self.data.as_mut_slice()) });
        } else {
            command.read_buffer_b(self.data.as_mut_slice());
        }

        Ok(())
    }
}

impl<'a> SdWrite<'a> {
    pub fn new(data: &'a mut Vec<u8>, wide: bool, ack: u16) -> Self {
        Self { data, wide, ack }
    }
}

#[test]
pub fn sd_status() {
    let status_bits = 0b1001_0010_0001_0010u16;
    let status = SdStatus(status_bits);

    assert_eq!(status.block_count_(), 0b1001);
    assert_eq!(status.block_size_(), 0b0);
    assert_eq!(status.disk(), 0b0100);
    assert_eq!(status.op(), SdOp::Write);
}

#[test]
pub fn sd_status_1() {
    let status_bits = 0x8081u16;
    let status = SdStatus(status_bits);

    assert!(status.check());
    assert_eq!(status.block_count(), 1);
    assert_eq!(status.block_size(), 512);
    assert_eq!(status.disk(), 0);
}

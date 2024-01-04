use crate::core::buttons::ButtonMap;
use crate::fpga::feature::SpiFeature;
use crate::fpga::{IntoLowLevelSpiCommand, SpiCommand, SpiCommandExt};
use crate::keyboard::Ps2Scancode;
use crate::types::StatusBitMap;
use chrono::{Datelike, NaiveDateTime, Timelike};
use std::time::SystemTime;

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

    UserIoSetStatus32Bits = 0x1E,

    /// Transmit RTC (time struct, including seconds) to the core.
    UserIoRtc = 0x22,

    UserIoGetStatusBits = 0x29,
}

impl IntoLowLevelSpiCommand for UserIoCommands {
    #[inline]
    fn into_ll_spi_command(self) -> (SpiFeature, u16) {
        (SpiFeature::IO, self as u16)
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
        eprintln!("UserIoKeyboardKeyDown: {:08x}", self.0);
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
pub struct GetStatusBits<'a>(pub &'a mut StatusBitMap);

impl SpiCommand for GetStatusBits<'_> {
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        let mut stchg = 0;
        let mut command = spi.command_read(UserIoCommands::UserIoGetStatusBits, &mut stchg);

        if ((stchg & 0xF0) == 0xA0) && (stchg & 0x0F) != 0 {
            for word in self.0.as_mut_raw_slice() {
                command.write_read(0u16, word);
            }
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
            .write_buffer(&bits16[0..4])
            .write_buffer_cond(self.0.has_extra(), &bits16[4..]);
        Ok(())
    }
}

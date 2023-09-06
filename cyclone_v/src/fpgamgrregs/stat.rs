use crate::fpgamgrregs::ctrl;
use bitfield::bitfield;
use core::fmt;

/// Status Register Mode.
#[derive(fmt::Debug, Clone, Copy, PartialEq)]
pub enum StatusRegisterMode {
    /// 0x0 - FPGA Powered Off
    PoweredOff = 0x0,
    /// 0x1 - FPGA in Reset Phase
    ResetPhase = 0x1,
    /// 0x2 - FPGA in Configuration Phase
    ConfigPhase = 0x2,
    /// 0x3 - FPGA in Initialization Phase. In CVP configuration, this state indicates IO
    /// configuration has completed.
    InitPhase = 0x3,
    /// 0x4 - FPGA in User Mode
    UserMode = 0x4,
    /// 0x5 - FPGA state has not yet been determined. This only occurs briefly after reset.
    Undetermined = 0x5,
}

impl From<u32> for StatusRegisterMode {
    fn from(value: u32) -> StatusRegisterMode {
        match value {
            0x0 => StatusRegisterMode::PoweredOff,
            0x1 => StatusRegisterMode::ResetPhase,
            0x2 => StatusRegisterMode::ConfigPhase,
            0x3 => StatusRegisterMode::InitPhase,
            0x4 => StatusRegisterMode::UserMode,
            0x5 => StatusRegisterMode::Undetermined,
            _ => unreachable!(),
        }
    }
}

impl From<StatusRegisterMode> for u8 {
    fn from(value: StatusRegisterMode) -> u8 {
        value as u8
    }
}

impl From<StatusRegisterMode> for u32 {
    fn from(value: StatusRegisterMode) -> u32 {
        value as u32
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfigurationMode {
    /// 16-bit Passive Parallel with Fast Power on Reset Delay;
    /// No AES Encryption;
    /// No Data Compression.
    /// CDRATIO must be programmed to x1.
    Passive16FastPower = 0x00,

    /// 16-bit Passive Parallel with Fast Power on Reset Delay;
    /// With AES Encryption;
    /// No Data Compression.
    /// CDRATIO must be programmed to x2
    Passive16FastPowerAES = 0x01,

    /// 16-bit Passive Parallel with Fast Power on Reset Delay;
    /// AES Optional;
    /// With Data Compression. CDRATIO must be programmed to x4
    Passive16FastPowerCompressed = 0x02,

    /// Reserved
    _Reserved0x03 = 0x03,

    /// 16-bit Passive Parallel with Slow Power on Reset Delay;
    /// No AES Encryption;
    /// No Data Compression.
    /// CDRATIO must be programmed to x1
    Parallel16SlowPower = 0x04,

    /// 16-bit Passive Parallel with Slow Power on Reset Delay;
    /// With AES Encryption;
    /// No Data Compression.
    /// CDRATIO must be programmed to x2
    Parallel16SlowPowerAES = 0x05,

    /// 16-bit Passive Parallel with Slow Power on Reset Delay;
    /// AES Optional;
    /// With Data Compression. CDRATIO must be programmed to x4
    Parallel16SlowPowerCompressed = 0x06,

    /// Reserved
    _Reserved0x07 = 0x07,

    /// 32-bit Passive Parallel with Fast Power on Reset Delay;
    /// No AES Encryption;
    /// No Data Compression.
    /// CDRATIO must be programmed to x1
    Parallel32FastPower = 0x08,

    /// 32-bit Passive Parallel with Fast Power on Reset Delay;
    /// With AES Encryption;
    /// No Data Compression.
    /// CDRATIO must be programmed to x4
    Parallel32FastPowerAES = 0x09,

    /// 32-bit Passive Parallel with Fast Power on Reset Delay;
    /// AES Optional;
    /// With Data Compression. CDRATIO must be programmed to x8
    Parallel32FastPowerCompressed = 0x0A,

    /// Reserved
    _Reserved0x0B = 0x0B,

    /// 32-bit Passive Parallel with Slow Power on Reset Delay;
    /// No AES Encryption;
    /// No Data Compression.
    /// CDRATIO must be programmed to x1
    Parallel32SlowPower = 0x0C,

    /// 32-bit Passive Parallel with Slow Power on Reset Delay;
    /// With AES Encryption;
    /// No Data Compression.
    /// CDRATIO must be programmed to x4
    Parallel32SlowPowerAES = 0x0D,

    /// 32-bit Passive Parallel with Slow Power on Reset Delay;
    /// AES Optional;
    /// With Data Compression.
    /// CDRATIO must be programmed to x8
    Parallel32SlowPowerCompressed = 0x0E,

    /// Reserved
    _Reserved0x0F = 0x0F,

    /// Reserved
    _Reserved0x10 = 0x10,

    /// Reserved
    _Reserved0x11 = 0x11,

    /// Reserved
    _Reserved0x12 = 0x12,

    /// Reserved
    _Reserved0x13 = 0x13,

    /// Reserved
    _Reserved0x14 = 0x14,

    /// Reserved
    _Reserved0x15 = 0x15,

    /// Reserved
    _Reserved0x16 = 0x16,

    /// Reserved
    _Reserved0x17 = 0x17,

    /// Reserved
    _Reserved0x18 = 0x18,

    /// Reserved
    _Reserved0x19 = 0x19,

    /// Reserved
    _Reserved0x1A = 0x1A,

    /// Reserved
    _Reserved0x1B = 0x1B,

    /// Reserved
    _Reserved0x1C = 0x1C,

    /// Reserved
    _Reserved0x1D = 0x1D,

    /// Reserved
    _Reserved0x1E = 0x1E,

    /// Reserved
    _Reserved0x1F = 0x1F,
}

impl ConfigurationMode {
    #[inline]
    pub fn is_32_bits(&self) -> bool {
        *self as u32 & 0x08 != 0
    }

    #[inline]
    pub fn cd_ratio(&self) -> ctrl::FpgaCtrlCdRatio {
        match (self.is_32_bits(), (*self as u32) & 0x03) {
            (false, 0x00) => ctrl::FpgaCtrlCdRatio::X1,
            (false, 0x01) => ctrl::FpgaCtrlCdRatio::X2,
            (false, 0x02) => ctrl::FpgaCtrlCdRatio::X4,
            (true, 0x00) => ctrl::FpgaCtrlCdRatio::X1,
            (true, 0x01) => ctrl::FpgaCtrlCdRatio::X4,
            (true, 0x02) => ctrl::FpgaCtrlCdRatio::X8,
            // ConfigurationMode::Passive16FastPower => ctrl::FpgaCtrlCdRatio::X1,
            // ConfigurationMode::Passive16FastPowerAES => ctrl::FpgaCtrlCdRatio::X2,
            // ConfigurationMode::Passive16FastPowerCompressed => ctrl::FpgaCtrlCdRatio::X4,
            // ConfigurationMode::Parallel16SlowPower => ctrl::FpgaCtrlCdRatio::X1,
            // ConfigurationMode::Parallel16SlowPowerAES => ctrl::FpgaCtrlCdRatio::X2,
            // ConfigurationMode::Parallel16SlowPowerCompressed => ctrl::FpgaCtrlCdRatio::X4,
            // ConfigurationMode::Parallel32FastPower => ctrl::FpgaCtrlCdRatio::X1,
            // ConfigurationMode::Parallel32FastPowerAES => ctrl::FpgaCtrlCdRatio::X4,
            // ConfigurationMode::Parallel32FastPowerCompressed => ctrl::FpgaCtrlCdRatio::X8,
            // ConfigurationMode::Parallel32SlowPower => ctrl::FpgaCtrlCdRatio::X1,
            // ConfigurationMode::Parallel32SlowPowerAES => ctrl::FpgaCtrlCdRatio::X4,
            // ConfigurationMode::Parallel32SlowPowerCompressed => ctrl::FpgaCtrlCdRatio::X8,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn is_aes(&self) -> bool {
        matches!(
            self,
            ConfigurationMode::Passive16FastPowerAES
                | ConfigurationMode::Passive16FastPowerCompressed
                | ConfigurationMode::Parallel16SlowPowerAES
                | ConfigurationMode::Parallel16SlowPowerCompressed
                | ConfigurationMode::Parallel32FastPowerAES
                | ConfigurationMode::Parallel32FastPowerCompressed
                | ConfigurationMode::Parallel32SlowPowerAES
                | ConfigurationMode::Parallel32SlowPowerCompressed
        )
    }

    #[inline]
    pub fn is_compressed(&self) -> bool {
        matches!(
            self,
            ConfigurationMode::Passive16FastPowerCompressed
                | ConfigurationMode::Parallel16SlowPowerCompressed
                | ConfigurationMode::Parallel32FastPowerCompressed
                | ConfigurationMode::Parallel32SlowPowerCompressed
        )
    }
}

impl From<u32> for ConfigurationMode {
    fn from(value: u32) -> Self {
        match value {
            0x00 => ConfigurationMode::Passive16FastPower,
            0x01 => ConfigurationMode::Passive16FastPowerAES,
            0x02 => ConfigurationMode::Passive16FastPowerCompressed,
            // 0x03 => ModeSelection::_Reserved0x03,
            0x04 => ConfigurationMode::Parallel16SlowPower,
            0x05 => ConfigurationMode::Parallel16SlowPowerAES,
            0x06 => ConfigurationMode::Parallel16SlowPowerCompressed,
            // 0x07 => ModeSelection::_Reserved0x07,
            0x08 => ConfigurationMode::Parallel32FastPower,
            0x09 => ConfigurationMode::Parallel32FastPowerAES,
            0x0A => ConfigurationMode::Parallel32FastPowerCompressed,
            // 0x0B => ModeSelection::_Reserved0x0B,
            0x0C => ConfigurationMode::Parallel32SlowPower,
            0x0D => ConfigurationMode::Parallel32SlowPowerAES,
            0x0E => ConfigurationMode::Parallel32SlowPowerCompressed,
            // 0x0F => ModeSelection::_Reserved0x0F,
            _ => unreachable!(),
        }
    }
}

bitfield! {
    pub struct StatusRegister(u32);
    impl Debug;

    /// This read-only field allows software to observe the MSEL inputs from the device pins.
    /// The MSEL pins define the FPGA configuration mode.
    pub into ConfigurationMode, msel, _: 7, 3;

    /// Reports FPGA state.
    /// 0x0 - FPGA Powered Off
    /// 0x1 - FPGA in Reset Phase
    /// 0x2 - FPGA in Configuration Phase
    /// 0x3 - FPGA in Initialization Phase. In CVP configuration, this state indicates IO configuration has completed.
    /// 0x4 - FPGA in User Mode
    /// 0x5 - FPGA state has not yet been determined. This only occurs briefly after reset.
    pub from into StatusRegisterMode, mode, set_mode: 2, 0;
}

#[test]
fn status_register_works() {
    let status = StatusRegister(0);
    assert_eq!(status.msel(), ConfigurationMode::Passive16FastPower);
    assert_eq!(status.mode(), StatusRegisterMode::PoweredOff);

    let status = StatusRegister(0x9);
    assert_eq!(status.msel(), ConfigurationMode::Passive16FastPowerAES);
    assert_eq!(status.mode(), StatusRegisterMode::ResetPhase);

    let status = StatusRegister(0x00000062);
    assert_eq!(status.msel(), ConfigurationMode::Parallel32SlowPower);
    assert_eq!(status.mode(), StatusRegisterMode::ConfigPhase);
}

use bitfield::bitfield;

/// Controls whether the FPGA configuration pins or HPS FPGA Manager drive configuration inputs
/// to the CB.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FpgaCtrlEn {
    /// FPGA configuration pins drive configuration inputs to the CB. Used when FPGA is configured
    /// by means other than the HPS.
    FpgaConfigurationPins = 0,

    /// FPGA Manager drives configuration inputs to the CB. Used when HPS configures the FPGA.
    FpgaManager = 1,
}

impl From<u32> for FpgaCtrlEn {
    fn from(b: u32) -> Self {
        match b {
            0 => FpgaCtrlEn::FpgaConfigurationPins,
            1 => FpgaCtrlEn::FpgaManager,
            _ => unreachable!(),
        }
    }
}

impl From<FpgaCtrlEn> for u32 {
    fn from(f: FpgaCtrlEn) -> Self {
        f as Self
    }
}

/// This field drives the active-low Chip Enable (nCE) signal to the CB. It should be set to 0
/// (configuration enabled) before CTRL.EN is set. This field only effects the FPGA if CTRL.EN is
/// 1.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FpgaCtrlNce {
    /// Configuration enabled.
    Enabled = 0,

    /// Configuration disabled.
    Disabled = 1,
}

impl From<u32> for FpgaCtrlNce {
    fn from(b: u32) -> Self {
        match b {
            0 => FpgaCtrlNce::Enabled,
            1 => FpgaCtrlNce::Disabled,
            _ => unreachable!(),
        }
    }
}

impl From<FpgaCtrlNce> for u32 {
    fn from(f: FpgaCtrlNce) -> Self {
        f as Self
    }
}

/// This field controls the Clock to Data Ratio (CDRATIO) for Normal Configuration and Partial
/// Reconfiguration data transfer from the AXI Slave to the FPGA. For Normal Configuration, the
/// value in this field must be set to be consistent to the implied CD ratio of the MSEL setting.
/// For Partial Reconfiguration, the value in this field must be set to the same clock to data
/// ratio in the options bits in the Normal Configuration file.
#[derive(Debug, Clone, Copy)]
pub enum FpgaCtrlCdRatio {
    /// 1:1 clock to data ratio.
    X1 = 0,

    /// 1:2 clock to data ratio.
    X2 = 1,

    /// 1:4 clock to data ratio.
    X4 = 2,

    /// 1:8 clock to data ratio.
    X8 = 3,
}

impl From<u32> for FpgaCtrlCdRatio {
    fn from(value: u32) -> Self {
        match value {
            0 => FpgaCtrlCdRatio::X1,
            1 => FpgaCtrlCdRatio::X2,
            2 => FpgaCtrlCdRatio::X4,
            3 => FpgaCtrlCdRatio::X8,
            _ => unreachable!(),
        }
    }
}

impl From<FpgaCtrlCdRatio> for u32 {
    fn from(f: FpgaCtrlCdRatio) -> Self {
        f as Self
    }
}

/// This field determines the Configuration Passive Parallel data bus width when HPS
/// configures the FPGA. Only 32-bit Passive Parallel or 16-bit Passive Parallel are
/// supported. When HPS does Normal Configuration, configuration should use 32-bit Passive
/// Parallel Mode. The external pins MSEL must be set appropriately for the configuration
/// selected. For Partial Reconfiguration, 16-bit Passive Parallel must be used.
#[derive(Debug, Clone, Copy)]
pub enum FpgaCtrlCfgWidth {
    /// 16-bit Passive Parallel.
    Passive16Bit = 0,
    /// 32-bit Passive Parallel.
    Passive32Bit = 1,
}

impl From<u32> for FpgaCtrlCfgWidth {
    fn from(value: u32) -> Self {
        match value {
            0 => FpgaCtrlCfgWidth::Passive16Bit,
            1 => FpgaCtrlCfgWidth::Passive32Bit,
            x => unreachable!("Invalid value for FpgaCtrlCfgWidth: {}", x),
        }
    }
}

impl From<FpgaCtrlCfgWidth> for u32 {
    fn from(f: FpgaCtrlCfgWidth) -> Self {
        f as Self
    }
}

bitfield! {
    /// Allows HPS to control FPGA configuration. The NCONFIGPULL, NSTATUSPULL, and CONFDONEPULL
    /// fields drive signals to the FPGA Control Block that are logically ORed into their
    /// respective pins. These signals are always driven independent of the value of EN. The
    /// polarity of the NCONFIGPULL, NSTATUSPULL, and CONFDONEPULL fields is inverted relative
    /// to their associated pins. The MSEL (external pins), CDRATIO and CFGWDTH signals determine
    /// the mode of operation for Normal Configuration. For Partial Reconfiguration, CDRATIO is
    /// used to set the appropriate clock to data ratio, and CFGWDTH should always be set to
    /// 16-bit Passive Parallel. AXICFGEN is used to enable transfer of configuration data by
    /// enabling or disabling DCLK during data transfers.
    pub struct FpgaConfigurationControl(u32);
    impl Debug;

    u32;

    /// Controls whether the FPGA configuration pins or HPS FPGA Manager drive configuration inputs
    /// to the CB.
    pub from into FpgaCtrlEn, en, set_en: 0, 0;

    /// This field drives the active-low Chip Enable (nCE) signal to the CB. It should be set to
    /// 0 (configuration enabled) before CTRL.EN is set. This field only effects the FPGA if
    /// CTRL.EN is 1.
    pub from into FpgaCtrlNce, nce, set_nce: 1, 1;

    /// The nCONFIG input is used to put the FPGA into its reset phase. If the FPGA was configured,
    /// its operation stops and it will have to be configured again to start operation.
    pub nconfigpull, set_nconfigpull: 2;

    /// Pulls down nSTATUS input to the CB
    pub nstatuspull, set_nstatuspull: 3;

    /// Pulls down CONF_DONE input to the CB
    pub confdonepull, set_confdonepull: 4;

    /// This field is used to assert PR_REQUEST to request partial reconfiguration while the FPGA
    /// is in User Mode. This field only affects the FPGA if CTRL.EN is 1.
    pub prreq, set_prreq: 5;

    /// This field controls the Clock to Data Ratio (CDRATIO) for Normal Configuration and Partial
    /// Reconfiguration data transfer from the AXI Slave to the FPGA. For Normal Configuration,
    /// the value in this field must be set to be consistent to the implied CD ratio of the MSEL
    /// setting. For Partial Reconfiguration, the value in this field must be set to the same
    /// clock to data ratio in the options bits in the Normal Configuration file.
    pub from into FpgaCtrlCdRatio, cdratio, set_cdratio: 7, 6;

    /// There are strict SW initialization steps for configuration, partial configuration and
    /// error cases. When SW is sending configuration files, this bit must be set before the
    /// file is transferred on the AXI bus. This bit enables the DCLK during the AXI configuration
    /// data transfers. Note, the AXI and configuration datapaths remain active irregardless of
    /// the state of this bit. Simply, if the AXI slave is enabled, the DCLK to the CB will be
    /// active. If disabled, the DCLK to the CB will not be active. So AXI transfers destined to
    /// the FPGA Manager when AXIEN is 0, will complete normally from the HPS perspective. This
    /// field only affects the FPGA if CTRL.EN is 1.
    pub axicfgen, set_axicfgen: 8;

    /// This field determines the Configuration Passive Parallel data bus width when HPS
    /// configures the FPGA. Only 32-bit Passive Parallel or 16-bit Passive Parallel are
    /// supported. When HPS does Normal Configuration, configuration should use 32-bit Passive
    /// Parallel Mode. The external pins MSEL must be set appropriately for the configuration
    /// selected. For Partial Reconfiguration, 16-bit Passive Parallel must be used.
    pub from into FpgaCtrlCfgWidth, cfgwdth, set_cfgwdth: 9, 9;
}

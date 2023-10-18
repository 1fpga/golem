pub use crate::utils::config_string::*;
use std::str::FromStr;

impl Config {
    /// Create a new config from the FPGA.
    /// This is disabled in Test as this module is still included in the test build.
    pub fn new(fpga: &mut crate::platform::de10::fpga::Fpga) -> Result<Self, &'static str> {
        let cfg_string = fpga.spi_mut().config_string();

        Self::from_str(&cfg_string)
    }
}

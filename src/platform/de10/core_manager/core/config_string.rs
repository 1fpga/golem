use crate::platform::de10::fpga::{Fpga, SpiCommands};

/// A component of a Core config string.
#[derive(Debug, Clone)]
pub enum ConfigComponent {
    /// The name of the core.
    CoreName(String),

    /// UART mode
    UartMode(String),

    /// Unknown config string. Can still be iterated on.
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct Config {
    components: Vec<ConfigComponent>,
}

impl Config {
    pub fn new(fpga: &mut Fpga) -> Self {
        let mut components = Vec::new();
        let cfgstring = fpga.spi_mut().config_string();

        eprintln!("cfgstring: {:?}", cfgstring);

        for (i, str) in cfgstring.split(";").enumerate() {
            eprintln!("str({i}): {:?}", str);
        }

        Self { components }
    }
}

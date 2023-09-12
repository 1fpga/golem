use crate::platform::de10::fpga::Fpga;

/// A component of a Core config string.
#[derive(Debug, Clone)]
pub enum ConfigComponent {
    /// The name of the core.
    CoreName(String),

    /// UART mode
    UartMode(String, String),
}

#[derive(Debug, Clone)]
pub struct Config {
    components: Vec<ConfigComponent>,
}

impl Config {
    pub fn new(fpga: &mut Fpga) -> Self {
        let mut components = Vec::new();

        Self { components }
    }
}

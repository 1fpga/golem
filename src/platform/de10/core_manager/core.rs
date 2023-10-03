use crate::platform::de10::fpga::{CoreInterfaceType, CoreType, Fpga};
use tracing::info;

mod config_string;

pub struct FpgaCore {
    fpga: Fpga,
    core_type: CoreType,
    spi_type: CoreInterfaceType,
    io_version: u8,
    config: config_string::Config,
}

impl FpgaCore {
    pub fn new(mut fpga: Fpga) -> Result<Self, &'static str> {
        let core_type = fpga.core_type().ok_or("Could not get core type.")?;
        let spi_type = fpga
            .core_interface_type()
            .ok_or("Could not get SPI type.")?;
        let io_version = fpga.core_io_version().ok_or("Could not get IO version.")?;
        let config = config_string::Config::new(&mut fpga);

        info!(?core_type, ?spi_type, io_version, "Core loaded:");

        Ok(FpgaCore {
            fpga,
            core_type,
            spi_type,
            io_version,
            config,
        })
    }
}

impl crate::platform::Core for FpgaCore {
    fn send_key(&mut self, key: u8) {
        todo!()
    }
}

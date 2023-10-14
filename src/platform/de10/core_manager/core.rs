use crate::platform::de10::core_manager::core::buttons::{ButtonMap, ButtonMapping, MisterButtons};
use crate::platform::de10::fpga::{CoreInterfaceType, CoreType, Fpga, SpiCommands, SpiFeature};
use tracing::info;

mod buttons;
mod config_string;

pub struct MisterFpgaCore {
    fpga: Fpga,
    core_type: CoreType,
    spi_type: CoreInterfaceType,
    io_version: u8,
    config: config_string::Config,

    mapping: ButtonMapping,
    map: ButtonMap,
}

impl MisterFpgaCore {
    pub fn new(mut fpga: Fpga) -> Result<Self, &'static str> {
        let core_type = fpga.core_type().ok_or("Could not get core type.")?;
        let spi_type = fpga
            .core_interface_type()
            .ok_or("Could not get SPI type.")?;
        let io_version = fpga.core_io_version().ok_or("Could not get IO version.")?;
        let config = config_string::Config::new(&mut fpga);

        info!(?core_type, ?spi_type, io_version, "Core loaded:");
        fpga.wait_for_ready();

        Ok(MisterFpgaCore {
            fpga,
            core_type,
            spi_type,
            io_version,
            config,
            mapping: ButtonMapping::sdl(),
            map: ButtonMap::new(),
        })
    }
}

impl crate::platform::Core for MisterFpgaCore {
    fn name(&self) -> &str {
        todo!()
    }

    fn send_key(&mut self, key: u8) {
        let spi = self.fpga.spi_mut();
        spi.command(SpiCommands::UserIoKeyboard);
        spi.write_b(key);
    }

    fn sdl_joy_button_down(&mut self, joystick_idx: u8, button: u8) {
        let mister_button: MisterButtons = self.mapping[button as usize];
        self.map.down(mister_button);
        let button_mask = self.map.mask();

        let spi = self.fpga.spi_mut();

        spi.command(SpiCommands::from_joystick_index(joystick_idx));
        spi.write(button_mask as u16);
        if button_mask >> 16 == 0 {
            spi.write((button_mask >> 16) as u16);
        }
        spi.disable(SpiFeature::IO);
    }
    fn sdl_joy_button_up(&mut self, joystick_idx: u8, button: u8) {
        let mister_button: MisterButtons = self.mapping[button as usize];
        self.map.up(mister_button);
        let button_mask = self.map.mask();

        let spi = self.fpga.spi_mut();

        let joystick = SpiCommands::from_joystick_index(joystick_idx);
        spi.command(joystick);
        spi.write((button_mask & 0xFFFF) as u16);
        if button_mask >> 16 == 0 {
            spi.write((button_mask >> 16) as u16);
        }
        spi.disable(SpiFeature::IO);
    }
}

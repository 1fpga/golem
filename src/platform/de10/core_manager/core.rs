use crate::platform::de10::core_manager::core::buttons::{ButtonMap, ButtonMapping, MisterButtons};
use crate::platform::de10::fpga::{CoreInterfaceType, CoreType, Fpga, SpiCommands};
use crate::types::StatusBitMap;
use crate::utils::config_string::ConfigMenu;
use tracing::{debug, info};

mod buttons;
mod config_string;

pub struct MisterFpgaCore {
    fpga: Fpga,
    pub core_type: CoreType,
    pub spi_type: CoreInterfaceType,
    pub io_version: u8,
    config: config_string::Config,

    mapping: ButtonMapping,
    map: ButtonMap,

    status: StatusBitMap,
}

impl MisterFpgaCore {
    pub fn new(mut fpga: Fpga) -> Result<Self, &'static str> {
        let core_type = fpga.core_type().ok_or("Could not get core type.")?;
        let spi_type = fpga
            .core_interface_type()
            .ok_or("Could not get SPI type.")?;
        let io_version = fpga.core_io_version().ok_or("Could not get IO version.")?;
        let config = config_string::Config::new(&mut fpga)?;

        info!(?core_type, ?spi_type, io_version, "Core loaded");
        info!(
            "Status bit map (mask):\n{}",
            config.status_bit_map_mask().debug_string()
        );
        fpga.wait_for_ready();

        Ok(MisterFpgaCore {
            fpga,
            core_type,
            spi_type,
            io_version,
            config,
            mapping: ButtonMapping::sdl(),
            map: ButtonMap::new(),
            status: Default::default(),
        })
    }
}

impl crate::platform::Core for MisterFpgaCore {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn menu_options(&self) -> &[ConfigMenu] {
        self.config.menu.as_slice()
    }

    fn status_bits(&self) -> StatusBitMap {
        self.status
    }

    fn set_status_bits(&mut self, bits: StatusBitMap) {
        debug!(?bits, "Setting status bits");
        let bits16 = bits.as_raw_slice();

        let command = self
            .fpga
            .spi_mut()
            .command(SpiCommands::SetStatus32Bits)
            .write_buffer(&bits16[0..4]);
        if bits.has_extra() {
            command.write_buffer(&bits16[4..]);
        }
        self.status = bits;
    }

    fn send_key(&mut self, key: u8) {
        self.fpga
            .spi_mut()
            .command(SpiCommands::UserIoKeyboard)
            .write_b(key);
    }

    fn sdl_joy_button_down(&mut self, joystick_idx: u8, button: u8) {
        let mister_button: MisterButtons = self.mapping[button as usize];
        self.map.down(mister_button);
        let button_mask = self.map.mask();

        let spi = self.fpga.spi_mut();

        let command = spi
            .command(SpiCommands::from_joystick_index(joystick_idx))
            .write(button_mask as u16);
        if button_mask >> 16 == 0 {
            command.write((button_mask >> 16) as u16);
        }
    }
    fn sdl_joy_button_up(&mut self, joystick_idx: u8, button: u8) {
        let mister_button: MisterButtons = self.mapping[button as usize];
        self.map.up(mister_button);
        let button_mask = self.map.mask();

        let spi = self.fpga.spi_mut();

        let command = spi
            .command(SpiCommands::from_joystick_index(joystick_idx))
            .write((button_mask & 0xFFFF) as u16);
        if button_mask >> 16 == 0 {
            command.write((button_mask >> 16) as u16);
        }
    }
}

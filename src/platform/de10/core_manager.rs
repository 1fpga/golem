use crate::platform::de10::fpga::{CoreInterfaceType, CoreType, Fpga};
use byteorder::{LittleEndian, ReadBytesExt};
use std::path::Path;
use tracing::info;

pub struct FpgaCore<'a> {
    fpga: &'a mut Fpga,
    core_type: CoreType,
    spi_type: CoreInterfaceType,
    io_version: u8,
}

impl<'a> FpgaCore<'a> {
    fn new(fpga: &'a mut Fpga) -> Result<Self, &'static str> {
        let core_type = fpga.core_type().ok_or("Could not get core type.")?;
        let spi_type = fpga
            .core_interface_type()
            .ok_or("Could not get SPI type.")?;
        let io_version = fpga.core_io_version().ok_or("Could not get IO version.")?;

        info!(?core_type, ?spi_type, io_version, "Core loaded:");

        Ok(FpgaCore {
            fpga,
            core_type,
            spi_type,
            io_version,
        })
    }
}

impl<'a> crate::platform::Core for FpgaCore<'a> {
    fn send_key(&mut self, key: u8) {
        todo!()
    }
}

pub struct CoreManager {
    fpga: Fpga,
}

impl CoreManager {
    pub fn new(fpga: Fpga) -> Self {
        Self { fpga }
    }

    pub fn fpga(&self) -> &Fpga {
        &self.fpga
    }

    pub fn fpga_mut(&mut self) -> &mut Fpga {
        &mut self.fpga
    }
}

impl crate::platform::CoreManager for CoreManager {
    type Core<'a> = FpgaCore<'a>;

    fn load_program(&mut self, path: impl AsRef<Path>) -> Result<FpgaCore<'_>, String> {
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        let program = bytes.as_slice();

        if program.as_ptr() as usize % 4 != 0 {
            return Err("Program is not aligned to 4 bytes".to_string());
        }

        let program = if &program[..6] != b"MiSTer" {
            program
        } else {
            let start = 16;
            let size = (&program[12..]).read_u32::<LittleEndian>().unwrap() as usize;
            &program[start..start + size]
        };

        self.fpga.core_reset().expect("Could not reset the Core");
        self.fpga.load_rbf(program).expect("Could not load RBF");

        let core = FpgaCore::new(&mut self.fpga).expect("Could not create a Core");
        // unsafe {
        //     crate::platform::de10::user_io::user_io_init(
        //         "\0".as_ptr() as *const _,
        //         std::ptr::null(),
        //     );
        // }

        Ok(core)
    }

    fn show_menu(&mut self) {
        self.fpga_mut().osd_enable();
    }

    fn hide_menu(&mut self) {
        self.fpga_mut().osd_disable();
    }
}

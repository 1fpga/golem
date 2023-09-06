use crate::platform::de10::fpga;
use byteorder::{LittleEndian, ReadBytesExt};
use std::path::Path;

pub struct CoreManager {
    fpga: fpga::Fpga,
}

impl CoreManager {
    pub fn new(fpga: fpga::Fpga) -> Self {
        Self { fpga }
    }

    pub fn fpga(&self) -> &fpga::Fpga {
        &self.fpga
    }

    pub fn fpga_mut(&mut self) -> &mut fpga::Fpga {
        &mut self.fpga
    }
}

impl crate::platform::CoreManager for CoreManager {
    fn load_program(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
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

        self.fpga.core_reset().unwrap();
        self.fpga.load_rbf(program).unwrap();

        Ok(())
    }
}

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

extern "C" {
    fn fpga_load_rbf(_: *const u8, _: *const u8, _: *const u8) -> i32;
}

impl crate::platform::CoreManager for CoreManager {
    fn load_program(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        // let path_str = path.as_ref().to_string_lossy().to_string();
        // let c_str_path = std::ffi::CString::new(path_str).unwrap();
        // unsafe {
        //     fpga_load_rbf(
        //         c_str_path.as_ptr() as *const u8,
        //         std::ptr::null(),
        //         std::ptr::null(),
        //     );
        // }

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

        let mut soc = self.fpga.soc_mut();
        let fpga_manager = soc.fpga_manager_mut();

        /// Core Reset.
        let gpo = fpga_manager.gpo() & (!0xC000_0000);
        fpga_manager.set_gpo(gpo | 0x4000_0000);

        fpga_manager.load_rbf(program).unwrap();

        Ok(())
    }
}

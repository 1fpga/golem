use crate::fpga::{FpgaError, MisterFpga};

pub trait Program {
    fn load(&self, fpga: &mut MisterFpga) -> Result<(), FpgaError>;
}

impl Program for &[u8] {
    fn load(&self, fpga: &mut MisterFpga) -> Result<(), FpgaError> {
        fpga.load_rbf_bytes(self)
    }
}

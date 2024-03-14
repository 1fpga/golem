use mister_fpga::core::MisterFpgaCore;
use mister_fpga::fpga::MisterFpga;

pub async fn get_core() -> Result<MisterFpgaCore, String> {
    let core = MisterFpgaCore::new(MisterFpga::init().unwrap())?;

    Ok(core)
}

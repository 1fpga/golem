use crate::ffi::fpga::socfpga::SocFpga;
use crate::shmem::shmem_map_c;
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::cell::RefCell;
use std::ffi::{c_char, c_int};
use tracing::debug;

mod socfpga;

extern "C" {
    pub fn fpga_io_init() -> c_int;

    pub fn fpga_spi(word: u16) -> u16;
    pub fn fpga_spi_en(mask: u32, en: u32);
    pub fn is_fpga_ready(quick: c_int) -> c_int;
    pub fn fpga_wait_to_reset();

    pub fn fpga_load_rbf(name: *const c_char, cfg: *const c_char, xml: *const c_char) -> c_int;
}

#[derive(Debug, Default)]
struct FpgaContext {
    pub soc_fpga: SocFpga,
}

thread_local! {
    static INITIALIZED: RefCell<bool> = RefCell::new(false);
    static CONTEXT: RefCell<FpgaContext> = RefCell::new(FpgaContext::default());
}

#[derive(Debug)]
pub struct Fpga;

impl Fpga {
    pub fn init() -> Result<Self, ()> {
        INITIALIZED.with(|initialized| {
            if !*initialized.borrow() {
                debug!("Initializing FPGA");
                *initialized.borrow_mut() = true;

                CONTEXT.with(|ctx| {
                    let mut ctx = ctx.borrow_mut();
                    eprintln!("{:?}", ctx);
                    let mut manager = ctx.soc_fpga.fpga_manager_mut();
                    manager.set_gpo(0);
                });

                // unsafe {
                //     fpga_io_init();
                // }
            }
        });
        Ok(Self)
    }
}

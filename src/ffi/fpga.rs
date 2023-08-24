use std::cell::RefCell;
use std::ffi::{c_char, c_int};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use strum::{EnumIter, FromRepr};
use tracing::{debug, error, info};

mod socfpga;
use socfpga::SocFpga;

extern "C" {
    pub fn fpga_spi(word: u16) -> u16;
    pub fn fpga_spi_en(mask: u32, en: u32);
    pub fn is_fpga_ready(quick: c_int) -> c_int;
    pub fn fpga_wait_to_reset();

    pub fn fpga_load_rbf(name: *const c_char, cfg: *const c_char, xml: *const c_char) -> c_int;

    pub fn fpgamgr_test_fpga_ready() -> c_int;

    pub fn reboot(cold: c_int);
}

mod ffi {
    use super::FPGA_SINGLETON;
    use std::ffi::c_int;
    use tracing::error;

    #[no_mangle]
    extern "C" fn fpga_core_id() -> c_int {
        unsafe {
            FPGA_SINGLETON
                .clone()
                .unwrap()
                .core_type()
                .map(|v| v as c_int)
                .unwrap_or_else(|| {
                    error!("FPGA core type mismatch");
                    -1
                })
        }
    }

    #[no_mangle]
    extern "C" fn fpga_get_fio_size() -> c_int {
        unsafe {
            match FPGA_SINGLETON.clone().unwrap().core_interface_type() {
                Some(super::CoreInterfaceType::SpiBus8Bit) => 0,
                Some(super::CoreInterfaceType::SpiBus16Bit) => 1,
                _ => -1,
            }
        }
    }

    #[no_mangle]
    extern "C" fn fpga_get_io_version() -> c_int {
        unsafe {
            FPGA_SINGLETON
                .clone()
                .unwrap()
                .core_io_version()
                .map(|v| v as c_int)
                .unwrap_or(-1)
        }
    }

    #[no_mangle]
    extern "C" fn fpga_wait_to_reset() {
        unsafe {
            FPGA_SINGLETON.clone().unwrap().wait_to_reset();
        }
    }
}

static mut INITIALIZED: AtomicBool = AtomicBool::new(false);

// TODO: Remove this when we're done re-writing fpga_io.cpp
// This is needed for the FFI implementations to work.
// Also, remove the Rc<RefCell<_>> below.
static mut FPGA_SINGLETON: Option<Fpga> = None;

/// FPGA core type.
#[derive(Debug, Eq, PartialEq, EnumIter, FromRepr)]
#[repr(u8)]
pub enum CoreType {
    /// Core type value should be unlikely to be returned by broken cores.
    CoreTypeUnknown = 0x55,

    /// Generic Core.
    CoreTypeGeneric = 0xA4,

    /// Sharp MZ Series.
    CoreTypeSharpMz = 0xA7,

    /// Generic Core using dual SDRAM.
    CoreTypeGenericDualSdram = 0xA8,
}

/// The interface type of the core.
#[derive(Debug, Eq, PartialEq, EnumIter, FromRepr)]
#[repr(u8)]
pub enum CoreInterfaceType {
    /// 8-bit SPI bus.
    SpiBus8Bit = 0,

    /// 16-bit SPI bus.
    SpiBus16Bit = 1,
}

#[derive(Debug, Clone)]
pub struct Fpga {
    soc: Rc<RefCell<SocFpga>>,
}

impl Fpga {
    pub fn init() -> Option<Self> {
        unsafe {
            if INITIALIZED.load(Ordering::Relaxed) {
                error!("FPGA already initialized. This will be an error.");
                // TODO: do this.
                // return Err(());
                return Some(FPGA_SINGLETON.clone().unwrap());
            }
            info!("Initializing FPGA");

            INITIALIZED.store(true, Ordering::Relaxed);

            let mut soc = SocFpga::default();
            let manager = soc.fpga_manager_mut();
            manager.set_gpo(0);
            let s = Self {
                soc: Rc::new(RefCell::new(soc)),
            };
            FPGA_SINGLETON = Some(s.clone());
            Some(s)
        }
    }

    pub fn core_type(&mut self) -> Option<CoreType> {
        let mut soc = self.soc.borrow_mut();
        let manager = soc.fpga_manager_mut();

        let gpo = manager.gpo() & 0x7FFF_FFFF;
        manager.set_gpo(0);
        let core_type: u32 = manager.gpi();
        manager.set_gpo(gpo | 0x80000000);

        if (core_type & 0xFFFFFF00) != 0x5CA62300 {
            error!("FPGA core type mismatch");
            None
        } else {
            CoreType::from_repr((core_type & 0xFF) as u8)
        }
    }

    pub fn core_interface_type(&self) -> Option<CoreInterfaceType> {
        let soc = self.soc.borrow();
        let manager = soc.fpga_manager();

        CoreInterfaceType::from_repr((manager.gpi() >> 16 & 1) as u8)
    }

    pub fn core_io_version(&self) -> Option<u8> {
        let soc = self.soc.borrow();
        let manager = soc.fpga_manager();

        let version = manager.gpi() >> 18 & 0b00000011;
        if version == 0 {
            error!("FPGA core IO version mismatch");
            None
        } else {
            Some(version as u8)
        }
    }

    #[inline]
    pub fn is_ready(&self) -> bool {
        unsafe { fpgamgr_test_fpga_ready() != 0 }
    }

    #[inline]
    fn is_ready_quick(&self) -> bool {
        (self.soc.borrow().fpga_manager().gpi() as i32) >= 0
    }

    #[inline]
    pub(super) fn wait_to_reset(&mut self) {
        let mut soc = self.soc.borrow_mut();
        let manager = soc.fpga_manager_mut();

        debug!("FPGA is not ready. JTAG uploading?");
        info!("Waiting for FPGA to be ready...");

        // Send the reset signal to the FPGA.
        let gpo = manager.gpo() & (!0xC0000000);
        manager.set_gpo(gpo | 0x40000000);

        while !self.is_ready() {
            std::thread::sleep(Duration::from_millis(10));
        }

        unsafe {
            reboot(0);
        }
    }

    /// Wait for the FPGA to be ready. This requires a mutable reference if the
    /// FPGA is not ready.
    pub fn wait_for_ready(&mut self) {
        while !self.is_ready_quick() {
            self.wait_to_reset();
        }
    }
}

impl Drop for Fpga {
    fn drop(&mut self) {
        unsafe {
            INITIALIZED.store(false, Ordering::Relaxed);
        }
    }
}

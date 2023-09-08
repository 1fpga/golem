use static_assertions::const_assert_eq;

pub mod ctrl;
pub mod gpio_ext_porta;
pub mod stat;

crate::declare_volatile_struct! {
    /// The FPGA Manager is a hardware block that manages the FPGA configuration
    /// process. It is responsible for configuring the FPGA with the image stored
    /// in the flash memory. It also provides a mechanism to reconfigure the FPGA
    /// with a new image stored in the flash memory.
    ///
    /// This structure must be aligned exactly with the hardware registers.
    /// Accessor methods should be used to access the registers with volatile
    /// calls to prevent the compiler from optimizing away the reads and writes.
    ///
    /// See https://www.intel.com/content/www/us/en/programmable/hps/cyclone-v/sfo1410067808053.html#sfo1410067808053
    #[repr(C)]
    pub struct FpgaManagerRegs {
        // FPGA Manager Module
        /// Status Register
        stat: stat::StatusRegister, // 0x00
        /// Control Register
        ctrl: ctrl::FpgaConfigurationControl,
        /// Data Clock Count Register
        dclkcnt: u32,
        /// Data Clock Status Register
        dclkstat: u32,
        /// General Purpose Output Register
        gpo: u32,
        /// General Purpose Input Register (RO)
        gpi: u32,
        /// Miscellaneous Input Register (RO)
        misci: u32,

        // Padding
        [padding] _pad_0x1c_0x82c: [u32; 517],

        // Configuration Monitor (MON) Registers
        /// GPIO Interrupt Enable Register
        gpio_inten: u32,
        /// GPIO Interrupt Mask Register
        gpio_intmask: u32,
        /// GPIO Interrupt Type Level Register
        gpio_inttype_level: u32,
        /// GPIO Interrupt Polarity Register
        gpio_int_polarity: u32,
        /// GPIO Interrupt Status Register
        gpio_intstatus: u32,
        /// GPIO Raw Interrupt Status Register
        gpio_raw_intstatus: u32,

        // Padding
        [padding] _pad_0x848: u32,

        /// GPIO Port A Clear Interrupt Register
        gpio_porta_eoi: u32,
        /// GPIO Port A External Port Register
        gpio_ext_porta: gpio_ext_porta::GpioExtPortA,

        // Padding
        [padding] _pad_0x854_0x85c: [u32; 3],

        /// GPIO Level Synchronization Register
        gpio_ls_sync: u32,

        // Padding
        [padding] _pad_0x864_0x868: [u32; 2],

        /// GPIO Version ID Code Register
        gpio_ver_id_code: u32,
        /// GPIO Configuration Register 2
        gpio_config_reg2: u32,
        /// GPIO Configuration Register 1
        gpio_config_reg1: u32,
    }
}

const_assert_eq!(core::mem::size_of::<FpgaManagerRegs>(), 0x878);

#[test]
fn update_ctrl() {
    let mut region = vec![0; 0x1000000];
    let mapper = crate::memory::RegionMemoryMapper::new(&mut region);
    let mut soc = crate::SocFpga::new(mapper);

    let mgr = soc.regs_mut();

    assert_eq!(mgr.ctrl().en(), ctrl::FpgaCtrlEn::FpgaConfigurationPins);
    mgr.update_ctrl(|ctrl| ctrl.set_en(ctrl::FpgaCtrlEn::FpgaManager));
    assert_eq!(mgr.ctrl().en(), ctrl::FpgaCtrlEn::FpgaManager);
}

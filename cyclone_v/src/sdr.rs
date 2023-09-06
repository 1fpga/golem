use memoffset::offset_of;
use static_assertions::const_assert_eq;

pub mod ctrlcfg;
pub mod fpgaportrst;

crate::declare_volatile_struct! {
    /// Address map for the SDRAM Interface registers
    #[repr(C)]
    pub struct SdramCtrl {
        [padding] _reserved_0x0_0x5000: [u8; 0x5000],

        /// Controller Configuration Register
        ctrlcfg: ctrlcfg::CtrlCfg,

        /// DRAM Timings 1 Register
        dramtiming1: u32,

        /// DRAM Timings 2 Register
        dramtiming2: u32,

        /// DRAM Timings 3 Register
        dramtiming3: u32,

        /// DRAM Timings 4 Register
        dramtiming4: u32,

        /// Lower Power Timing Register
        lowpwrtiming: u32,

        /// ODT Control Register
        dramodt: u32,

        [padding] _pad_0x501c_0x502c: [u32; 4],

        /// DRAM Address Widths Register
        dramaddrw: u32,

        /// DRAM Interface Data Width Register
        dramifwidth: u32,

        [padding] _pad_0x5034_0x5038: [u32; 1],

        /// DRAM Status Register
        dramsts: u32,

        /// ECC Interrupt Register
        dramintr: u32,

        /// ECC Single Bit Error Count Register
        sbecount: u32,

        /// ECC Double Bit Error Count Register
        dbecount: u32,

        /// ECC Error Address Register
        erraddr: u32,

        /// ECC Auto-correction Dropped Count Register
        dropcount: u32,

        /// ECC Auto-correction Dropped Address Register
        dropaddr: u32,

        /// Low Power Control Register
        lowpwreq: u32,

        /// Low Power Acknowledge Register
        lowpwrack: u32,

        /// Static Configuration Register
        staticcfg: u32,

        /// Memory Controller Width Register
        ctrlwidth: u32,

        [padding] _pad_0x5064_0x507c: [u32; 6],

        /// Port Configuration Register
        portcfg: u32,

        /// FPGA Ports Reset Control Register
        fpgaportrst: fpgaportrst::FpgaPortRst,

        [padding] _pad_0x5084_0x508c: [u32; 2],

        /// Memory Protection Port Default Register
        protportdefault: u32,

        /// Memory Protection Address Register
        protruleaddr: u32,

        /// Memory Protection ID Register
        protruleid: u32,

        /// Memory Protection Rule Data Register
        protruledata: u32,

        /// Memory Protection Rule Read-Write Register
        protrulerdwr: u32,

        /// Scheduler priority Register
        mppriority: u32,

        /// Port Sum of Weight Register[1/4]
        mpweight_0_4: u32,

        /// Port Sum of Weight Register[2/4]
        mpweight_1_4: u32,

        /// Port Sum of Weight Register[3/4]
        mpweight_2_4: u32,

        /// Port Sum of Weight Register[4/4]
        mpweight_3_4: u32,

        [padding] _pad_0x50c0_0x50e0: [u32; 11],

        /// Controller Command Pool Priority Remap Register
        remappriority: u32,
    }
}

// Fail to compile if the register layout changes.
const_assert_eq!(offset_of!(SdramCtrl, ctrlcfg), 0x5000);
const_assert_eq!(offset_of!(SdramCtrl, erraddr), 0x5048);
const_assert_eq!(offset_of!(SdramCtrl, portcfg), 0x507C);
const_assert_eq!(offset_of!(SdramCtrl, protruledata), 0x5098);
const_assert_eq!(offset_of!(SdramCtrl, remappriority), 0x50E0);

const_assert_eq!(core::mem::size_of::<SdramCtrl>(), 0x50E4);

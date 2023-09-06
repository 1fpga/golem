use static_assertions::const_assert_eq;
crate::declare_volatile_struct! {
    /// Registers in the Reset Manager module
    #[repr(C)]
    pub struct ResetManager {
        /// Status Register
        stat: u32,

        /// Control Register
        ctrl: u32,

        /// Reset Cycles Count Register
        counts: u32,

        [padding] _pad_0x0c_0x10: [u32; 1],

        /// MPU Module Reset Register
        mpumodrst: u32,

        /// Peripheral Module Reset Register
        permodrst: u32,

        /// Peripheral 2 Module Reset Register
        per2modrst: u32,

        /// Bridge Module Reset Register
        brgmodrst: u32,

        /// Miscellaneous Module Reset Register
        miscmodrst: u32,

        [padding] _pad_0x24_0x54: [u32; 12],

        /// Test Scratch Register
        tstscratch: u32,
    }
}

const_assert_eq!(core::mem::size_of::<ResetManager>(), 0x58);

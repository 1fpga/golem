use bitfield::bitfield;

/// This bit field selects the order for address interleaving. Programming this field with
/// different values gives different mappings between the AXI or Avalon-MM address and the
/// SDRAM address. Program this field with the following binary values to select the ordering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddrOrder {
    /// Bank interleaved with no rank (chip select) interleaving.
    ChipRowBankColumn = 0,

    /// No interleaving.
    ChipBankRowColumn = 1,

    /// Bank interleaved with rank (chip select) interleaving.
    RowChipBankColumn = 2,
}

impl From<u32> for AddrOrder {
    fn from(value: u32) -> Self {
        match value {
            0 => AddrOrder::ChipRowBankColumn,
            1 => AddrOrder::ChipBankRowColumn,
            2 => AddrOrder::RowChipBankColumn,
            _ => unreachable!(),
        }
    }
}

impl From<AddrOrder> for u32 {
    fn from(value: AddrOrder) -> Self {
        value as u32
    }
}

/// This bit field selects the memory type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemType {
    /// DDR2 SDRAM.
    Ddr2 = 1,

    /// DDR3 SDRAM.
    Ddr3 = 2,

    /// LPDDR2 SDRAM.
    LpDdr2 = 4,
}

impl From<u32> for MemType {
    fn from(value: u32) -> Self {
        match value {
            1 => MemType::Ddr2,
            2 => MemType::Ddr3,
            4 => MemType::LpDdr2,
            _ => unreachable!(),
        }
    }
}

impl From<MemType> for u32 {
    fn from(value: MemType) -> Self {
        value as u32
    }
}

bitfield! {
    /// The Controller Configuration Register determines the behavior of the controller.
    pub struct CtrlCfg(u32);
    impl Debug;

    /// Set to a one to enable the controller to issue burst terminate commands. This must only
    /// be set when the DRAM memory type is LPDDR2.
    pub bursttermen, set_bursttermen: 25;

    /// Set to a one to enable the controller to issue burst interrupt commands. This must only be
    /// set when the DRAM memory type is LPDDR2.
    pub burstintren, set_burstintren: 24;

    /// Set to a one to enable DRAM operation if no DM pins are connected.
    pub nodmpins, set_nodmpins: 23;

    /// Enables DQS tracking in the PHY.
    pub dqstrken, set_dqstrken: 22;

    /// Specifies the number of DRAM burst transactions an individual transaction will allow to
    /// reorder ahead of it before its priority is raised in the memory controller.
    pub starvelimit, set_starvelimit: 21, 16;

    /// This bit controls whether the controller can re-order operations to optimize SDRAM
    /// bandwidth. It should generally be set to a one.
    pub reorderen, set_reorderen: 15;

    /// Enable the deliberate insertion of double bit errors in data written to memory. This
    /// should only be used for testing purposes.
    pub gendbe, set_gendbe: 14;

    /// Enable the deliberate insertion of single bit errors in data written to memory. This
    /// should only be used for testing purposes.
    pub gensbe, set_gensbe: 13;

    /// Set to a one to enable ECC overwrites. ECC overwrites occur when a correctable ECC error
    /// is seen and cause a new read/modify/write to be scheduled for that location to clear the
    /// ECC error.
    pub cfg_enable_ecc_code_overwrites, set_cfg_enable_ecc_code_overwrites: 12;

    /// Enable auto correction of the read data returned when single bit error is detected.
    pub ecccorren, set_ecccorren: 11;

    /// Enable the generation and checking of ECC. This bit must only be set if the memory
    /// connected to the SDRAM interface is 24 or 40 bits wide. If you set this, you must clear
    /// the useeccasdata field in the staticcfg register.
    pub eccen, set_eccen: 10;

    /// This bit field selects the order for address interleaving. Programming this field with
    /// different values gives different mappings between the AXI or Avalon-MM address and the
    /// SDRAM address. Program this field with the following binary values to select the ordering.
    pub from into AddrOrder, addrorder, set_addrorder: 9, 8;

    /// Configures burst length as a static decimal value. Legal values are valid for JEDEC
    /// allowed DRAM values for the DRAM selected in cfg_type. For DDR3, this should be programmed
    /// with 8 (binary "01000"), for DDR2 it can be either 4 or 8 depending on the exact DRAM chip.
    /// LPDDR2 can be programmed with 4, 8, or 16 and LPDDR can be programmed with 2, 4, or 8.
    /// You must also program the membl field in the staticcfg register.
    pub membl, set_membl: 7, 3;

    /// This bit field selects the memory type.
    pub from into MemType, memtype, set_memtype: 2, 0;
}

use bitfield::bitfield;
bitfield! {
    /// The L3 interconnect has separate address maps for the various L3 Masters. Generally, the
    /// addresses are the same for most masters. However, the sparse interconnect of the L3 switch
    /// causes some masters to have holes in their memory maps. The remap bits are not mutually
    /// exclusive. Each bit can be set independently and in combinations. Priority for the bits
    /// is determined by the bit offset: lower offset bits take precedence over higher offset bits.
    pub struct Remap(u32);
    impl Debug;

    /// Controls whether the Lightweight HPS2FPGA AXI Bridge is visible to L3 masters or not.
    pub lwhps2fpga, set_lwhps2fpga: 4;

    /// Controls whether the HPS2FPGA AXI Bridge is visible to L3 masters or not.
    pub hps2fpga, set_hps2fpga: 3;

    /// Controls the mapping of address 0x0 for L3 masters other than the MPU. Determines
    /// whether address 0x0 for these masters is mapped to the SDRAM or on-chip RAM. Only
    /// affects the following masters: DMA controllers (standalone and those built in to
    /// peripherals), FPGA-to-HPS Bridge, and DAP.
    pub nonmpuzero, set_nonmpuzero: 1;

    /// Controls whether address 0x0 for the MPU L3 master is mapped to the Boot ROM or On-chip
    /// RAM. This field only has an effect on the MPU L3 master.
    pub mpuzero, set_mpuzero: 0;
}

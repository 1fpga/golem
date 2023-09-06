use memoffset::offset_of;
use static_assertions::const_assert_eq;

crate::declare_volatile_struct! {
    /// Registers in the System Manager module.
    #[repr(C)]
    pub struct SystemManagerModule {
        /// System Manager Module
        siliconid1: u32,
        siliconid2: u32,
        [padding] _pad_0x8_0xf: [u32; 2],
        wddbg: u32,
        bootinfo: u32,
        hpsinfo: u32,
        parityinj: u32,

        /// FPGA Interface Group
        fpgaintfgrp_gbl: u32,
        fpgaintfgrp_indiv: u32,
        fpgaintfgrp_module: u32,
        [padding] _pad_0x2c_0x2f: u32,

        /// Scan Manager Group
        scanmgrgrp_ctrl: u32,
        [padding] _pad_0x34_0x3f: [u32; 3],

        /// Freeze Control Group
        frzctrl_vioctrl: u32,
        [padding] _pad_0x44_0x4f: [u32; 3],
        frzctrl_hioctrl: u32,
        frzctrl_src: u32,
        frzctrl_hwctrl: u32,
        [padding] _pad_0x5c_0x5f: u32,

        /// EMAC Group
        emacgrp_ctrl: u32,
        emacgrp_l3master: u32,
        [padding] _pad_0x68_0x6f: [u32; 2],

        /// DMA Controller Group
        dmagrp_ctrl: u32,
        dmagrp_persecurity: u32,
        [padding] _pad_0x78_0x7f: [u32; 2],

        /// Preloader (initial software) Group
        iswgrp_handoff: [u32; 8],
        [padding] _pad_0xa0_0xbf: [u32; 8],

        /// Boot ROM Code Register Group
        romcodegrp_ctrl: u32,
        romcodegrp_cpu1startaddr: u32,
        romcodegrp_initswstate: u32,
        romcodegrp_initswlastld: u32,
        romcodegrp_bootromswstate: u32,
        [padding] _pad_0xd4_0xdf: [u32; 3],

        /// Warm Boot from On-Chip RAM Group
        romcodegrp_warmramgrp_enable: u32,
        romcodegrp_warmramgrp_datastart: u32,
        romcodegrp_warmramgrp_length: u32,
        romcodegrp_warmramgrp_execution: u32,
        romcodegrp_warmramgrp_crc: u32,
        [padding] _pad_0xf4_0xff: [u32; 3],

        /// Boot ROM Hardware Register Group
        romhwgrp_ctrl: u32,
        [padding] _pad_0x104_0x107: u32,

        /// SDMMC Controller Group
        sdmmcgrp_ctrl: u32,
        sdmmcgrp_l3master: u32,

        /// NAND Flash Controller Register Group
        nandgrp_bootstrap: u32,
        nandgrp_l3master: u32,

        /// USB Controller Group
        usbgrp_l3master: u32,
        [padding] _pad_0x11c_0x13f: [u32; 9],

        /// ECC Management Register Group
        eccgrp_l2: u32,
        eccgrp_ocram: u32,
        eccgrp_usb0: u32,
        eccgrp_usb1: u32,
        eccgrp_emac0: u32,
        eccgrp_emac1: u32,
        eccgrp_dma: u32,
        eccgrp_can0: u32,
        eccgrp_can1: u32,
        eccgrp_nand: u32,
        eccgrp_qspi: u32,
        eccgrp_sdmmc: u32,
        [padding] _pad_0x170_0x3ff: [u32; 164],

        /// Pin Mux Control Group
        emacio: [u32; 20],
        flashio: [u32; 12],
        generalio: [u32; 28],
        [padding] _pad_0x4f0_0x4ff: [u32; 4],
        mixed1io: [u32; 22],
        mixed2io: [u32; 8],
        gplinmux: [u32; 23],
        gplmux: [u32; 71],
        nandusefpga: u32,
        [padding] _pad_0x6f4: u32,
        rgmii1usefpga: u32,
        [padding] _pad_0x6fc_0x700: [u32; 2],
        i2c0usefpga: u32,
        sdmmcusefpga: u32,
        [padding] _pad_0x70c_0x710: [u32; 2],
        rgmii0usefpga: u32,
        [padding] _pad_0x718_0x720: [u32; 3],
        i2c3usefpga: u32,
        i2c2usefpga: u32,
        i2c1usefpga: u32,
        spim1usefpga: u32,
        [padding] _pad_0x734: u32,
        spim0usefpga: u32,
    }
}

const_assert_eq!(offset_of!(SystemManagerModule, spim0usefpga), 0x738);

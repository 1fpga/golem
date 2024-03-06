use memoffset::offset_of;
use static_assertions::const_assert_eq;

pub mod remap;

crate::declare_volatile_struct! {
    /// Registers to control L3 interconnect settings.
    /// TODO: refactor this to include more sub-types (like in the documentation).
    #[repr(C)]
    pub struct L3Regs {
        /// The L3 interconnect has separate address maps for the various L3 Masters. Generally,
        /// the addresses are the same for most masters. However, the sparse interconnect of the
        /// L3 switch causes some masters to have holes in their memory maps. The remap bits are
        /// not mutually exclusive. Each bit can be set independently and in combinations. Priority
        /// for the bits is determined by the bit offset: lower offset bits take precedence over
        /// higher offset bits.
        [writeonly] remap: remap::Remap,

        [padding] _pad_0x4_0x8: [u32; 1],

        // Security Register Group
        l4main: u32,
        l4sp: u32,
        l4mp: u32,
        l4osc1: u32,
        l4spim: u32,
        stm: u32,
        lwhps2fpgaregs: u32,
        [padding] _pad_0x24_0x28: [u32; 1],
        usb1: u32,
        nanddata: u32,
        [padding] _pad_0x30_0x80: [u32; 20],
        usb0: u32,
        nandregs: u32,
        qspidata: u32,
        fpgamgrdata: u32,
        hps2fpgaregs: u32,
        acp: u32,
        rom: u32,
        ocram: u32,
        sdrdata: u32,

        [padding] _pad_0xa4_0x1fd0: [u32; 1995],

        /* ID Register Group */
        periph_id_4: u32,
        [padding] _pad_0x1fd4_0x1fe0: [u32; 3],
        periph_id_0: u32,
        periph_id_1: u32,
        periph_id_2: u32,
        periph_id_3: u32,
        comp_id_0: u32,
        comp_id_1: u32,
        comp_id_2: u32,
        comp_id_3: u32,
        [padding] _pad_0x2000_0x2008: [u32; 2],

        /* L4 MAIN */
        l4main_fn_mod_bm_iss: u32,

        [padding] _pad_0x200c_0x3008: [u32; 1023],

        /* L4 SP */
        l4sp_fn_mod_bm_iss: u32,

        [padding] _pad_0x300c_0x4008: [u32; 1023],

        /* L4 MP */
        l4mp_fn_mod_bm_iss: u32,

        [padding] _pad_0x400c_0x5008: [u32; 1023],

        /* L4 OSC1 */
        l4osc_fn_mod_bm_iss: u32,

        [padding] _pad_0x500c_0x6008: [u32; 1023],

        /* L4 SPIM */
        l4spim_fn_mod_bm_iss: u32,

        [padding] _pad_0x600c_0x7008: [u32; 1023],

        /* STM */
        stm_fn_mod_bm_iss: u32,
        [padding] _pad_0x700c_0x7108: [u32; 63],
        stm_fn_mod: u32,

        [padding] _pad_0x710c_0x8008: [u32; 959],

        /* LWHPS2FPGA */
        lwhps2fpga_fn_mod_bm_iss: u32,
        [padding] _pad_0x800c_0x8108: [u32; 63],
        lwhps2fpga_fn_mod: u32,

        [padding] _pad_0x810c_0xa008: [u32; 1983],

        /* USB1 */
        usb1_fn_mod_bm_iss: u32,
        [padding] _pad_0xa00c_0xa044: [u32; 14],
        usb1_ahb_cntl: u32,

        [padding] _pad_0xa048_0xb008: [u32; 1008],

        /* NANDDATA */
        nanddata_fn_mod_bm_iss: u32,
        [padding] _pad_0xb00c_0xb108: [u32; 63],
        nanddata_fn_mod: u32,

        [padding] _pad_0xb10c_0x20008: [u32; 21439],

        /* USB0 */
        usb0_fn_mod_bm_iss: u32,
        [padding] _pad_0x2000c_0x20044: [u32; 14],
        usb0_ahb_cntl: u32,

        [padding] _pad_0x20048_0x21008: [u32; 1008],

        /* NANDREGS */
        nandregs_fn_mod_bm_iss: u32,
        [padding] _pad_0x2100c_0x21108: [u32; 63],
        nandregs_fn_mod: u32,

        [padding] _pad_0x2110c_0x22008: [u32; 959],

        /* QSPIDATA */
        qspidata_fn_mod_bm_iss: u32,
        [padding] _pad_0x2200c_0x22044: [u32; 14],
        qspidata_ahb_cntl: u32,

        [padding] _pad_0x22048_0x23008: [u32; 1008],

        /* FPGAMGRDATA */
        fpgamgrdata_fn_mod_bm_iss: u32,
        [padding] _pad_0x2300c_0x23040: [u32; 13],
        fpgamgrdata_wr_tidemark: u32,
        [padding] _pad_0x23044_0x23108: [u32; 49],
        fn_mod: u32,

        [padding] _pad_0x2310c_0x24008: [u32; 959],

        /* HPS2FPGA */
        hps2fpga_fn_mod_bm_iss: u32,
        [padding] _pad_0x2400c_0x24040: [u32; 13],
        hps2fpga_wr_tidemark: u32,
        [padding] _pad_0x24044_0x24108: [u32; 49],
        hps2fpga_fn_mod: u32,
        [padding] _pad_0x2410c_0x25008: [u32; 959],

        /* ACP */
        acp_fn_mod_bm_iss: u32,
        [padding] _pad_0x2500c_0x25108: [u32; 63],
        acp_fn_mod: u32,
        [padding] _pad_0x2510c_0x26008: [u32; 959],

        /* Boot ROM */
        bootrom_fn_mod_bm_iss: u32,
        [padding] _pad_0x2600c_0x26108: [u32; 63],
        bootrom_fn_mod: u32,
        [padding] _pad_0x2610c_0x27008: [u32; 959],

        /* On-chip RAM */
        ocram_fn_mod_bm_iss: u32,
        [padding] _pad_0x2700c_0x27040: [u32; 13],
        ocram_wr_tidemark: u32,
        [padding] _pad_0x27044_0x27108: [u32; 49],
        ocram_fn_mod: u32,
        [padding] _pad_0x2710c_0x42024: [u32; 27590],

        /* DAP */
        dap_fn_mod2: u32,
        dap_fn_mod_ahb: u32,
        [padding] _pad_0x4202c_0x42100: [u32; 53],
        dap_read_qos: u32,
        dap_write_qos: u32,
        dap_fn_mod: u32,
        [padding] _pad_0x4210c_0x43100: [u32; 1021],

        /* MPU */
        mpu_read_qos: u32,
        mpu_write_qos: u32,
        mpu_fn_mod: u32,
        [padding] _pad_0x4310c_0x44028: [u32; 967],

        /* SDMMC */
        sdmmc_fn_mod_ahb: u32,
        [padding] _pad_0x4402c_0x44100: [u32; 53],
        sdmmc_read_qos: u32,
        sdmmc_write_qos: u32,
        sdmmc_fn_mod: u32,
        [padding] _pad_0x4410c_0x45100: [u32; 1021],

        /* DMA */
        dma_read_qos: u32,
        dma_write_qos: u32,
        dma_fn_mod: u32,
        [padding] _pad_0x4510c_0x46040: [u32; 973],

        /* FPGA2HPS */
        fpga2hps_wr_tidemark: u32,
        [padding] _pad_0x46044_0x46100: [u32; 47],
        fpga2hps_read_qos: u32,
        fpga2hps_write_qos: u32,
        fpga2hps_fn_mod: u32,
        [padding] _pad_0x4610c_0x47100: [u32; 1021],

        /* ETR */
        etr_read_qos: u32,
        etr_write_qos: u32,
        etr_fn_mod: u32,
        [padding] _pad_0x4710c_0x48100: [u32; 1021],

        /* EMAC0 */
        emac0_read_qos: u32,
        emac0_write_qos: u32,
        emac0_fn_mod: u32,
        [padding] _pad_0x4810c_0x49100: [u32; 1021],

        /* EMAC1 */
        emac1_read_qos: u32,
        emac1_write_qos: u32,
        emac1_fn_mod: u32,
        [padding] _pad_0x4910c_0x4a028: [u32; 967],

        /* USB0 */
        usb0_fn_mod_ahb: u32,
        [padding] _pad_0x4a02c_0x4a100: [u32; 53],
        usb0_read_qos: u32,
        usb0_write_qos: u32,
        usb0_fn_mod: u32,
        [padding] _pad_0x4a10c_0x4b100: [u32; 1021],

        /* NAND */
        nand_read_qos: u32,
        nand_write_qos: u32,
        nand_fn_mod: u32,
        [padding] _pad_0x4b10c_0x4c028: [u32; 967],

        /* USB1 */
        usb1_fn_mod_ahb: u32,
        [padding] _pad_0x4c02c_0x4c100: [u32; 53],
        usb1_read_qos: u32,
        usb1_write_qos: u32,
        usb1_fn_mod: u32,

    }
}

const_assert_eq!(offset_of!(L3Regs, sdrdata), 0xA0);
const_assert_eq!(offset_of!(L3Regs, l4main_fn_mod_bm_iss), 0x2008);
const_assert_eq!(offset_of!(L3Regs, fpgamgrdata_fn_mod_bm_iss), 0x23008);
const_assert_eq!(offset_of!(L3Regs, usb1_read_qos), 0x4C100);
const_assert_eq!(core::mem::size_of::<L3Regs>(), 0x4C10C);

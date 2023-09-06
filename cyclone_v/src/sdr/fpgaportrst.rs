use bitfield::bitfield;
bitfield! {
    /// This register implements functionality to allow the CPU to control when the MPFE will
    /// enable the ports to the FPGA fabric.
    pub struct FpgaPortRst(u32);
    impl Debug;

    /// This register should be written to with a 1 to enable the selected FPGA port to exit reset.
    /// Writing a bit to a zero will stretch the port reset until the register is written. Read
    /// data ports are connected to bits 3:0, with read data port 0 at bit 0 to read data port 3
    /// at bit 3. Write data ports 0 to 3 are mapped to 4 to 7, with write data port 0 connected
    /// to bit 4 to write data port 3 at bit 7. Command ports are connected to bits 8 to 13, with
    /// command port 0 at bit 8 to command port 5 at bit 13. Expected usage would be to set all
    /// the bits at the same time but setting some bits to a zero and others to a one is supported.
    pub portrstn, set_portrstn: 13, 0;
}

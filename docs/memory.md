# Memory Model in MiSTer

The memory model in MiSTer defines memory as 2 zones:

1. The main HPS memory, usable by Linux.
   This is the first 512MB of the DDR3 on-board RAM.
   The kernel, firmware and all Linux applications use this memory.
   The size of this memory is a hard limitation, as the FPGA only has access to the last 512MB.
   The physical memory range is `0x00000000..0x20000000`.
2. The FPGA memory, usable by the FPGA.
   This is the last 512MB of the DDR3 on-board RAM.
   The FPGA can use this memory for whatever it wants.
   The HPS has access to this physical memory.
   The physical memory range is `0x20000000..0x40000000`.


Within the FPGA memory, the first 32MB are reserved for the Core's framebuffer itself.
The Cores should use Ascal (the AVALON SCALER provided by MiSTer's framework).



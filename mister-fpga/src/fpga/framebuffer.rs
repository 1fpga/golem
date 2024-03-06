// use cyclone-v::memory::{DevMemMemoryMapper, MemoryMapper};
//
// pub const FB_PIXEL_COUNT: usize = 1920 * 1080;
// pub const FB_SIZE: usize = FB_PIXEL_COUNT * 4 * 3;
// pub const FB_BASE_ADDRESS: usize = 0x2000_0000;
//
// pub struct FpgaFramebuffer<M: MemoryMapper> {
//     pub memory: M,
// }
//
// impl Default for FpgaFramebuffer<DevMemMemoryMapper> {
//     fn default() -> Self {
//         Self::new(
//             DevMemMemoryMapper::create(FB_BASE_ADDRESS, FB_SIZE)
//                 .expect("Could not mmap framebuffer."),
//         )
//     }
// }
//
// impl<M: MemoryMapper> FpgaFramebuffer<M> {
//     pub fn new(memory: M) -> Self {
//         Self { memory }
//     }
//
//     pub fn init(&mut self) {
//         // Spi::uio.command()
//         todo!()
//     }
// }

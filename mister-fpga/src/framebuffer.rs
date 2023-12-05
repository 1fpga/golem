use cyclone_v::memory::{DevMemMemoryMapper, MemoryMapper};
use tracing::info;

pub const FB_PIXEL_COUNT: usize = 1920 * 1080;
pub const FB_SIZE: usize = FB_PIXEL_COUNT * 4 * 3;
pub const FB_BASE_ADDRESS: usize = 0x2000_0000;
pub const BUFFER_SIZE: usize = 2048 * 1024 * 3;

const DE10_PAGE_SIZE: usize = 4096;

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Debug)]
#[repr(C)]
struct FbHeader {
    pub magic: u16,
    pub header_len: u16,
    pub width: u16,
    pub height: u16,
    pub line: u16,
    pub output_width: u16,
    pub output_height: u16,
}

pub struct FpgaFramebuffer<M: MemoryMapper> {
    memory: M,
    header: FbHeader,
    start: *const u8,
}

impl Default for FpgaFramebuffer<DevMemMemoryMapper> {
    fn default() -> Self {
        let address = FB_BASE_ADDRESS; // - DE10_PAGE_SIZE;
        let size = BUFFER_SIZE;
        let start = address & !(DE10_PAGE_SIZE - 1);
        let offset = start - address;
        let mapper =
            DevMemMemoryMapper::create(address, size).expect("Could not mmap framebuffer.");

        Self::new(mapper, offset).unwrap()
    }
}

impl<M: MemoryMapper> FpgaFramebuffer<M> {
    fn new(memory: M, offset: usize) -> Result<Self, &'static str> {
        let header: *const u8 = memory.as_ptr();

        let buffer = unsafe { std::slice::from_raw_parts(header, 16) };

        // Bytes are in big endian, but ARM is in little endian.
        let header = FbHeader {
            magic: (buffer[0] as u16) << 8 | (buffer[1] as u16),
            header_len: (buffer[2] as u16) << 8 | (buffer[3] as u16),
            width: (buffer[6] as u16) << 8 | (buffer[7] as u16),
            height: (buffer[8] as u16) << 8 | (buffer[9] as u16),
            line: (buffer[10] as u16) << 8 | (buffer[11] as u16),
            output_width: (buffer[12] as u16) << 8 | (buffer[13] as u16),
            output_height: (buffer[14] as u16) << 8 | (buffer[15] as u16),
        };

        if header.magic != 0x0101 {
            return Err("Invalid framebuffer header.");
        }
        info!("Header data: {:?}", header);
        let start = unsafe { memory.as_ptr::<u8>().add(offset) };

        Ok(Self {
            memory,
            header,
            start,
        })
    }

    pub fn take_screenshot(&mut self) -> Result<Image, String> {
        let height = self.header.height as usize;
        let width = self.header.width as usize;
        let line = self.header.line as usize;
        let fb = unsafe {
            std::slice::from_raw_parts(
                self.start.add(self.header.header_len as usize),
                height * width * 3,
            )
        };

        let mut data = vec![0; height * width * 3];
        for y in 0..height {
            let line = &fb[y * line..y * line + width * 3];
            data[y * width * 3..y * width * 3 + width * 3].copy_from_slice(line)
        }

        return Ok(Image {
            width: width as u32,
            height: height as u32,
            data,
        });
    }
}

use bitfield::bitfield;
use image::{DynamicImage, RgbImage};
use simple_endian::BigEndian;
use tracing::debug;

use cyclone_v::memory::{DevMemMemoryMapper, MemoryMapper};

pub const FB_BASE_ADDRESS: usize = 0x2000_0000;
pub const BUFFER_SIZE: usize = 2048 * 1024 * 3 * 4;

pub const SCALER_FB_TYPE: u8 = 0x01;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ScalerPixelFormat {
    /// 16-bit RGB565.
    RGB16 = 0,

    /// 24-bit RGB888.
    RGB24 = 1,

    /// 32-bit RGBA8888.
    /// The alpha channel is ignored.
    RGBA32 = 2,

    INVALID = 0xFF,
}

impl From<u8> for ScalerPixelFormat {
    fn from(value: u8) -> Self {
        match value {
            0 => ScalerPixelFormat::RGB16,
            1 => ScalerPixelFormat::RGB24,
            2 => ScalerPixelFormat::RGBA32,
            _ => ScalerPixelFormat::INVALID,
        }
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct ScalerAttributes(u16);
    impl Debug;
    u8;

    pub interlaced, _: 0;
    pub field_number, _: 1;
    pub horizontal_downscaled, _: 2;
    pub vertical_downscaled, _: 3;

    /// True if triple buffered.
    pub triple_buffered, _: 4;

    /// A Frame counter in the scaler image header. Although this is
    /// named "counter", it is more like a checksum and its value will
    /// not necessarily increment with each frame.
    pub frame_counter, _: 5, 3;
}

impl From<u16> for ScalerAttributes {
    fn from(value: u16) -> Self {
        ScalerAttributes(value)
    }
}

/// An internal type to represent the framebuffer header.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub(crate) struct FbHeader {
    ty: u8,
    scaler_pixel_format: u8,
    header_len: BigEndian<u16>,
    attributes: BigEndian<u16>,
    width: BigEndian<u16>,
    height: BigEndian<u16>,
    line: BigEndian<u16>,
    output_width: BigEndian<u16>,
    output_height: BigEndian<u16>,
}

impl FbHeader {
    #[inline]
    pub unsafe fn from_memory(memory: *const u8) -> Option<Self> {
        let header = (memory as *const FbHeader).read_volatile();
        if header.ty == SCALER_FB_TYPE {
            Some(header)
        } else {
            None
        }
    }

    #[allow(unused)]
    pub fn frame_checksum(&self) -> u8 {
        self.attributes().frame_counter()
    }

    #[inline]
    pub fn scaler_pixel_format(&self) -> ScalerPixelFormat {
        self.scaler_pixel_format.into()
    }

    #[inline]
    pub fn header_len(&self) -> u16 {
        self.header_len.into()
    }

    #[inline]
    pub fn attributes(&self) -> ScalerAttributes {
        let bytes: u16 = self.attributes.into();
        ScalerAttributes::from(bytes)
    }

    #[inline]
    pub fn width(&self) -> u16 {
        self.width.into()
    }

    #[inline]
    pub fn height(&self) -> u16 {
        self.height.into()
    }

    #[inline]
    pub fn line(&self) -> u16 {
        self.line.into()
    }

    #[allow(unused)]
    pub fn output_width(&self) -> u16 {
        self.output_width.into()
    }

    #[allow(unused)]
    pub fn output_height(&self) -> u16 {
        self.output_height.into()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FramebufferType {
    Single,
    TripleSmall,
    TripleLarge,
}

impl FramebufferType {
    #[inline]
    pub(crate) fn offset_of(&self, index: u8) -> Option<usize> {
        match (self, index) {
            (_, 0) => Some(0),
            (FramebufferType::TripleSmall, 1) => Some(0x0020_0000),
            (FramebufferType::TripleSmall, 2) => Some(0x0040_0000),
            (FramebufferType::TripleLarge, 1) => Some(0x0080_0000),
            (FramebufferType::TripleLarge, 2) => Some(0x0100_0000),
            _ => None,
        }
    }
}

/// An iterator that waits a frame.
pub struct FrameIter {
    frame_counters: [*const u8; 3],
}

impl FrameIter {
    pub fn new<M: MemoryMapper>(framebuffer: &FpgaFramebuffer<M>) -> Self {
        let header0 = framebuffer.offset_of(0).unwrap();
        let header1 = framebuffer.offset_of(1).unwrap_or(header0);
        let header2 = framebuffer.offset_of(2).unwrap_or(header0);

        unsafe {
            let ptr0 = framebuffer.memory.as_ptr::<u8>().add(header0).add(5);
            let ptr1 = framebuffer.memory.as_ptr::<u8>().add(header1).add(5);
            let ptr2 = framebuffer.memory.as_ptr::<u8>().add(header2).add(5);

            let frame_counters = [ptr0, ptr1, ptr2];

            Self { frame_counters }
        }
    }
}

impl Iterator for FrameIter {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let last: u8 = self.frame_counters.iter().map(|f| f.read_volatile()).sum();

            loop {
                let current: u8 = self.frame_counters.iter().map(|f| f.read_volatile()).sum();
                if current != last {
                    break;
                }
            }

            Some(())
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FpgaFramebuffer<M: MemoryMapper> {
    memory: M,

    ty_: Option<FramebufferType>,
}

impl Default for FpgaFramebuffer<DevMemMemoryMapper> {
    fn default() -> Self {
        // In MiSTer there is an alignment of the address to the page size.
        // We know the page size in advance, so we don't need to calculate
        // it.
        let address = FB_BASE_ADDRESS;
        let size = BUFFER_SIZE;
        let mapper =
            DevMemMemoryMapper::create(address, size).expect("Could not mmap framebuffer.");

        Self::new(mapper).unwrap()
    }
}

impl<M: MemoryMapper> FpgaFramebuffer<M> {
    fn new(memory: M) -> Result<Self, &'static str> {
        Ok(Self { memory, ty_: None })
    }

    pub(crate) fn update_type_from_core(&mut self) {
        let first = unsafe { self.header_offset(0) };
        self.ty_ = if !first
            .map(|h| h.attributes().triple_buffered())
            .unwrap_or_default()
        {
            Some(FramebufferType::Single)
        } else {
            let (small, large) = unsafe {
                (
                    self.header_offset(FramebufferType::TripleSmall.offset_of(1).unwrap()),
                    self.header_offset(FramebufferType::TripleLarge.offset_of(1).unwrap()),
                )
            };

            match (small, large) {
                (_, Some(_)) => Some(FramebufferType::TripleLarge),
                (Some(_), None) => Some(FramebufferType::TripleSmall),
                _ => None,
            }
        }
    }

    /// Unsafely acquire the framebuffer header from an offset in memory.
    unsafe fn header_offset(&self, offset: usize) -> Option<FbHeader> {
        FbHeader::from_memory(self.memory.as_ptr::<u8>().add(offset))
    }

    pub(crate) fn offset_of(&self, index: u8) -> Option<usize> {
        self.ty_.and_then(|ty| ty.offset_of(index))
    }

    #[allow(unused)]
    pub(crate) fn header(&self, index: u8) -> Option<FbHeader> {
        self.offset_of(index)
            .and_then(|offset| unsafe { self.header_offset(offset) })
    }

    fn first_header(&self) -> FbHeader {
        unsafe { self.header_offset(0).unwrap() }
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), String> {
        let header_len = self.first_header().header_len() as usize;
        self.memory
            .as_mut_range(header_len..(header_len + data.len()))
            .copy_from_slice(data);

        Ok(())
    }

    pub fn take_screenshot(&self) -> Result<DynamicImage, String> {
        // Bytes are in big endian, but ARM is in little endian.
        let header = self.first_header();

        debug!("Header data: {:?}", header);

        let height = header.height() as usize;
        let width = header.width() as usize;
        let line = header.line() as usize;
        let start = self.memory.as_ptr::<u8>();
        let fb = unsafe {
            std::slice::from_raw_parts(start.add(header.header_len() as usize), line * height * 3)
        };

        // TODO: add support for RGBA and RGB565.
        let mut img = match header.scaler_pixel_format() {
            ScalerPixelFormat::RGB16 => {
                return Err("RGB565 not supported.".to_string());
            }
            ScalerPixelFormat::RGB24 => RgbImage::new(width as u32, height as u32),
            ScalerPixelFormat::RGBA32 => {
                return Err("RGBA32 not supported.".to_string());
            }
            ScalerPixelFormat::INVALID => {
                return Err("Invalid Scaler PixelFormat.".to_string());
            }
        };

        for y in 0..height {
            let line = &fb[y * line..y * line + width * 3];
            img.get_mut(y * width * 3..y * width * 3 + width * 3)
                .unwrap()
                .copy_from_slice(line);
        }

        Ok(DynamicImage::ImageRgb8(img))
    }
}

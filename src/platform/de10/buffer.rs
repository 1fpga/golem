use crate::macguiver::buffer::DrawBuffer;
use embedded_graphics::pixelcolor::raw::ToBytes;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;

mod sizes {
    use embedded_graphics::geometry::Size;

    /// The size of the title bar.
    pub const TITLE: Size = Size::new(256, 15);

    /// The size of the main OSD display. We never resize it to be smaller,
    /// instead reusing the size for other information.
    pub const MAIN: Size = Size::new(256, 16 * 8);
}

/// The OSD Display target for embedded graphics.
///
/// This code is very similar to the SimulationDisplay provided by the embedded-graphics-simulator
/// crate, except for how the index of a pixel is calculated, because of how we
/// transfer the data to the core.
#[derive(Debug)]
pub struct OsdDisplayView {
    /// The inner framebuffer. Even in case where we run on the embedded
    /// platform itself, we use the SimulatorDisplay as a framebuffer (and
    /// disable the SDL portion).
    pub inner: DrawBuffer<BinaryColor>,

    /// An offset of the display in the framebuffer. This is only used
    /// on embedded platforms for the title bar. The title bar is the
    /// shown height, but when rendering into line arrays we need to
    /// skip pixels.
    offset_y: usize,

    /// The number of lines in this display.
    lines: u32,

    /// The top line of the display.
    top_line: u32,
}

impl OsdDisplayView {
    pub fn line_iter(&self) -> impl Iterator<Item = u32> {
        self.top_line..self.top_line + self.lines
    }
}

impl OsdDisplayView {
    fn with_offset(mut self, offset_y: usize) -> Self {
        Self { offset_y, ..self }
    }

    fn with_top_line(mut self, top_line: u32) -> Self {
        Self { top_line, ..self }
    }

    fn with_lines(mut self, lines: u32) -> Self {
        Self { lines, ..self }
    }

    /// Creates a new display filled with a color. There is no need to create an OsdDisplay
    /// with custom sizes or color, so this method should not be exposed publicly, except
    /// for tests.
    ///
    /// This constructor can be used if `C` doesn't implement `From<BinaryColor>` or another
    /// default color is wanted.
    fn with_default_color_(size: Size, default_color: BinaryColor) -> Self {
        Self {
            inner: DrawBuffer::with_default_color(size, default_color),
            offset_y: 0,
            lines: size.height / 8,
            top_line: 0,
        }
    }

    /// Returns the color of the pixel at a point.
    ///
    /// # Panics
    ///
    /// Panics if `point` is outside the display.
    #[inline]
    pub fn get_pixel(&self, point: Point) -> BinaryColor {
        self.inner.get_pixel(point)
    }
}

impl OsdDisplayView {
    /// Creates a title bar display for the MiSTer.
    pub fn title() -> Self {
        Self::with_default_color_(sizes::TITLE, BinaryColor::Off)
            .with_offset(4)
            .with_top_line(16)
            .with_lines(3)
    }

    /// Creates the main view of the OSD.
    pub fn main() -> Self {
        Self::with_default_color_(sizes::MAIN, BinaryColor::Off)
    }
}

impl OsdDisplayView {
    /// Get a binary line array from the buffer (a single line).
    pub fn get_binary_line_array(&self, line: u32) -> Vec<u8> {
        let inner = &self.inner;
        let height = inner.size().height as i32;
        let y = ((line - self.top_line) * 8) as i32 - self.offset_y as i32;

        let mut line_buffer = vec![0u8; inner.size().width as usize];
        let off = BinaryColor::Off;

        let px = |x, y| {
            if y >= 0 && y < height && inner.get_pixel(Point::new(x, y)) != off {
                1u8
            } else {
                0u8
            }
        };

        for x in 0..256 {
            line_buffer[x as usize] = px(x, y)
                + (px(x, y + 1) << 1)
                + (px(x, y + 2) << 2)
                + (px(x, y + 3) << 3)
                + (px(x, y + 4) << 4)
                + (px(x, y + 5) << 5)
                + (px(x, y + 6) << 6)
                + (px(x, y + 7) << 7);
        }
        line_buffer
    }
}

impl OsdDisplayView {
    /// Converts the display content to big endian raw data.
    #[inline]
    pub fn to_be_bytes(&self) -> Vec<u8> {
        self.inner.to_be_bytes()
    }

    /// Converts the display content to little endian raw data.
    #[inline]
    pub fn to_le_bytes(&self) -> Vec<u8> {
        self.inner.to_le_bytes()
    }

    /// Converts the display content to native endian raw data.
    #[inline]
    pub fn to_ne_bytes(&self) -> Vec<u8> {
        self.inner.to_ne_bytes()
    }
}

impl Dimensions for OsdDisplayView {
    fn bounding_box(&self) -> Rectangle {
        self.inner.bounding_box()
    }
}

impl DrawTarget for OsdDisplayView {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.inner.draw_iter(pixels)
    }
}

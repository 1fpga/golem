//! Module that implements a BinaryColor display for embedded-graphics for the OSD
//! display on the MiSTer FPGA.
//!
//! The OsdDisplay does not implement any Drawable, instead relying on the `send`
//! method to send the data to the FPGA itself. It does not keep any internal
//! buffers, and is light weigh.
use crate::fpga::{osd_io, MisterFpga};
use embedded_graphics::image::GetPixel;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;

pub mod sizes {
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
pub struct OsdDisplay {
    /// The size of the display, in pixels.
    size: Size,

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

impl OsdDisplay {
    fn line_iter(&self) -> impl Iterator<Item = u32> {
        self.top_line..self.top_line + self.lines
    }

    /// Send the buffer to the OSD.
    pub fn send<B: GetPixel<Color = BinaryColor> + OriginDimensions>(
        &self,
        fpga: &mut MisterFpga,
        buffer: &B,
    ) {
        let size = buffer.size();
        // Send everything to the scaler. We could optimize by only sending differences,
        // but it isn't needed and would add complexity.
        for line in self.line_iter() {
            let line_buffer = self.get_binary_line_array(buffer, line, size);
            fpga.spi_mut()
                .execute(osd_io::OsdIoWriteLine(line as u8, &line_buffer))
                .unwrap();
        }
    }

    fn with_offset(self, offset_y: usize) -> Self {
        Self { offset_y, ..self }
    }

    fn with_top_line(self, top_line: u32) -> Self {
        Self { top_line, ..self }
    }

    fn with_lines(self, lines: u32) -> Self {
        Self { lines, ..self }
    }

    /// Creates a new display filled with a color. There is no need to create an OsdDisplay
    /// with custom sizes or color, so this method should not be exposed publicly, except
    /// for tests.
    ///
    /// This constructor can be used if `C` doesn't implement `From<BinaryColor>` or another
    /// default color is wanted.
    fn new(size: Size) -> Self {
        Self {
            size,
            offset_y: 0,
            lines: size.height / 8,
            top_line: 0,
        }
    }

    /// Creates a title bar display for the MiSTer.
    pub fn title() -> Self {
        Self::new(sizes::TITLE)
            .with_offset(4)
            .with_top_line(16)
            .with_lines(3)
    }

    /// Creates the main view of the OSD.
    pub fn main() -> Self {
        Self::new(sizes::MAIN)
    }

    /// Get a binary line array from the buffer (a single line).
    fn get_binary_line_array<B: GetPixel<Color = BinaryColor>>(
        &self,
        buffer: &B,
        line: u32,
        size: Size,
    ) -> Vec<u8> {
        let height = size.height as i32;
        let y = ((line - self.top_line) * 8) as i32 - self.offset_y as i32;

        let mut line_buffer = vec![0u8; size.width as usize];
        let px = |x, y| {
            if y >= 0 && y < height && buffer.pixel(Point::new(x, y)) == Some(BinaryColor::On) {
                1u8
            } else {
                0u8
            }
        };

        for x in 0..(size.width as i32) {
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

impl OriginDimensions for OsdDisplay {
    fn size(&self) -> Size {
        self.size
    }
}

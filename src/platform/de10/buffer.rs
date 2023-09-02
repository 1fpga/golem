use crate::macguiver::buffer::DrawBuffer;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;

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
    pub fn line_iter(&self) -> impl Iterator<Item = u32> {
        self.top_line..self.top_line + self.lines
    }

    pub fn send(&self, buffer: &DrawBuffer<BinaryColor>) {
        use super::{osd, spi};

        // Send everything to the scaler.
        for line in self.line_iter() {
            unsafe {
                let line_buffer = self.get_binary_line_array(buffer, line);
                spi::spi_osd_cmd_cont(osd::OSD_CMD_WRITE | (line as u8));
                spi::spi_write(line_buffer.as_ptr(), 256, 0);
                spi::DisableOsd();
            }
        }
    }
}

impl OsdDisplay {
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
    pub fn get_binary_line_array(&self, buffer: &DrawBuffer<BinaryColor>, line: u32) -> Vec<u8> {
        let height = buffer.size().height as i32;
        let y = ((line - self.top_line) * 8) as i32 - self.offset_y as i32;

        let mut line_buffer = vec![0u8; buffer.size().width as usize];
        let off = BinaryColor::Off;

        let px = |x, y| {
            if y >= 0 && y < height && buffer.get_pixel(Point::new(x, y)) != off {
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

impl OriginDimensions for OsdDisplay {
    fn size(&self) -> Size {
        self.size
    }
}

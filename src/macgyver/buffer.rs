use embedded_graphics::pixelcolor::raw::ToBytes;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use std::convert::TryFrom;
use std::fmt::{Debug, Formatter};

/// A buffer that can be drawn to in EmbeddedDisplay. Most widgets will draw
/// directly to this.
pub struct DrawBuffer<C> {
    size: Size,
    pixels: Box<[C]>,
}

impl<C> Debug for DrawBuffer<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DrawBuffer")
            .field("size", &self.size)
            .finish()
    }
}

impl<C: PixelColor> DrawBuffer<C> {
    /// Creates a buffer filled with a color.
    pub fn with_default_color(size: Size, default_color: C) -> Self {
        Self {
            size,
            pixels: vec![default_color; size.width as usize * size.height as usize]
                .into_boxed_slice(),
        }
    }

    /// Returns the color of the pixel at a point.
    pub fn get_pixel(&self, point: Point) -> C {
        let index = self.point_to_index(point);

        self.pixels[index.expect("Point is outside the buffer size")]
    }

    fn point_to_index(&self, point: Point) -> Option<usize> {
        let (x, y) = <(u32, u32)>::try_from(point).ok()?;
        if x < self.size.width && y < self.size.height {
            Some((x + y * self.size.width) as usize)
        } else {
            None
        }
    }

    pub fn size(&self) -> Size {
        self.size
    }
}

impl<C: PixelColor + From<BinaryColor>> DrawBuffer<C> {
    /// Creates a buffer filled with the equivalent of black (`BinaryColor::Off`).
    pub fn new(size: Size) -> Self {
        Self::with_default_color(size, C::from(BinaryColor::Off))
    }

    /// Creates a buffer that's the size of the OSD title in a MiSTer legacy setup.
    pub fn osd_title() -> Self {
        Self::new(Size::new(320, 16))
    }
}

impl<C> DrawBuffer<C>
where
    C: PixelColor + ToBytes,
    <C as ToBytes>::Bytes: AsRef<[u8]>,
{
    /// Converts the display content to big endian raw data.
    pub fn to_be_bytes(&self) -> Vec<u8> {
        self.to_bytes(ToBytes::to_be_bytes)
    }

    /// Converts the display content to little endian raw data.
    pub fn to_le_bytes(&self) -> Vec<u8> {
        self.to_bytes(ToBytes::to_le_bytes)
    }

    /// Converts the display content to native endian raw data.
    pub fn to_ne_bytes(&self) -> Vec<u8> {
        self.to_bytes(ToBytes::to_ne_bytes)
    }

    fn to_bytes<F>(&self, pixel_to_bytes: F) -> Vec<u8>
    where
        F: Fn(C) -> C::Bytes,
    {
        let mut bytes = Vec::new();

        if C::Raw::BITS_PER_PIXEL >= 8 {
            for pixel in self.pixels.iter() {
                bytes.extend_from_slice(pixel_to_bytes(*pixel).as_ref())
            }
        } else {
            let pixels_per_byte = 8 / C::Raw::BITS_PER_PIXEL;

            for row in self.pixels.chunks(self.size.width as usize) {
                for byte_pixels in row.chunks(pixels_per_byte) {
                    let mut value = 0;

                    for pixel in byte_pixels {
                        value <<= C::Raw::BITS_PER_PIXEL;
                        value |= pixel.to_be_bytes().as_ref()[0];
                    }

                    value <<= C::Raw::BITS_PER_PIXEL * (pixels_per_byte - byte_pixels.len());

                    bytes.push(value);
                }
            }
        }

        bytes
    }
}

impl<C: PixelColor> Drawable for DrawBuffer<C> {
    type Color = C;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        target.draw_iter(self.pixels.iter().enumerate().map(|(i, &c)| {
            let x = (i as u32) % self.size.width;
            let y = (i as u32) / self.size.width;
            Pixel(Point::new(x as i32, y as i32), c)
        }))
    }
}

impl<C: PixelColor> DrawTarget for DrawBuffer<C> {
    type Color = C;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels.into_iter() {
            if let Some(index) = self.point_to_index(point) {
                self.pixels[index] = color;
            }
        }

        Ok(())
    }
}

impl<C> OriginDimensions for DrawBuffer<C> {
    fn size(&self) -> Size {
        self.size
    }
}

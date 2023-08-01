use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::widgets::Widget;
use bitvec::order::Lsb0;
use bitvec::slice::BitSlice;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::{BinaryColor, PixelColor};
use embedded_graphics::{Drawable, Pixel};
use image::{GenericImageView, ImageFormat};
use std::fmt::Debug;
use std::io::{BufReader, Read, Seek};
use std::path::Path;

/// Creates a widget that displays an image.
#[derive(Debug)]
pub struct ImageWidget<C> {
    buffer: DrawBuffer<C>,
}

impl<C: PixelColor + From<BinaryColor>> ImageWidget<C> {
    pub fn new(buffer: DrawBuffer<C>) -> Self {
        Self { buffer }
    }

    pub fn empty() -> Self {
        Self::new(DrawBuffer::new(Size::zero()))
    }

    fn from_image_io_reader<T: Read + Seek>(
        reader: image::io::Reader<BufReader<T>>,
    ) -> Result<Self, String> {
        let img = reader.decode().unwrap();
        let (w, h) = img.dimensions();
        let mut buffer = DrawBuffer::new(Size::new(w, h));
        let img = img.into_luma8();

        buffer
            .draw_iter(img.pixels().enumerate().map(|(index, p)| {
                Pixel(
                    Point::new(index as i32 % w as i32, index as i32 / w as i32),
                    match p.0[0] {
                        0u8 => C::from(BinaryColor::Off),
                        _ => C::from(BinaryColor::On),
                    },
                )
            }))
            .unwrap();

        Ok(Self { buffer })
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let img = image::io::Reader::open(path.as_ref())?;
        Ok(Self::from_image_io_reader(img).unwrap())
    }

    pub fn from_png(data: impl Read + Seek) -> Self {
        let data = BufReader::new(data);
        let decoder = image::io::Reader::with_format(data, ImageFormat::Png);
        Self::from_image_io_reader(decoder).unwrap()
    }
}

impl ImageWidget<BinaryColor> {
    pub fn from_bin(data: impl AsRef<[u8]>, width: u32) -> Result<Self, String> {
        let data = data.as_ref();
        let height = data.len() as u32 / width * 8;
        let size = Size::new(width, height);
        let mut buffer = DrawBuffer::new(size);
        let slice = BitSlice::<_, Lsb0>::from_slice(data);

        buffer
            .draw_iter(slice.iter().enumerate().map(|(index, bit)| {
                Pixel(
                    Point::new(index as i32 % width as i32, index as i32 / width as i32),
                    BinaryColor::from(*bit),
                )
            }))
            .unwrap();

        Ok(Self { buffer })
    }
}

impl<C: PixelColor + Debug> Widget for ImageWidget<C> {
    type Color = C;

    fn size_hint(&self, _parent_size: Size) -> Size {
        self.buffer.size()
    }

    fn update(&mut self) {}

    fn draw(&self, target: &mut DrawBuffer<Self::Color>) {
        self.buffer.draw(target).unwrap();
    }
}

use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::platform::sdl::settings::OutputSettings;
use embedded_graphics::{
    pixelcolor::{raw::ToBytes, Rgb888, RgbColor},
    prelude::*,
    primitives::Rectangle,
};
use std::{convert::TryFrom, marker::PhantomData};

/// Output image.
///
/// An output image is the result of applying [`OutputSettings`] to a [`SimulatorDisplay`]. It can
/// be used to save a simulator display to a PNG file.
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OutputImage<C> {
    size: Size,
    pub(crate) data: Box<[u8]>,
    pub(crate) output_settings: OutputSettings,
    color_type: PhantomData<C>,
}

impl<C> OutputImage<C>
where
    C: PixelColor + From<Rgb888> + ToBytes,
    <C as ToBytes>::Bytes: AsRef<[u8]>,
{
    /// Creates a new output image.
    pub(crate) fn new<DisplayC>(size: Size, output_settings: &OutputSettings) -> Self
    where
        DisplayC: PixelColor + Into<Rgb888>,
    {
        let size = output_settings.framebuffer_size(size);

        // Create an empty pixel buffer, filled with the background color.
        let background_color = C::from(output_settings.theme.convert(Rgb888::BLACK)).to_be_bytes();
        let data = background_color
            .as_ref()
            .iter()
            .copied()
            .cycle()
            .take(size.width as usize * size.height as usize * background_color.as_ref().len())
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Self {
            size,
            data,
            output_settings: output_settings.clone(),
            color_type: PhantomData,
        }
    }

    /// Updates the image from a [`SimulatorDisplay`].
    pub fn update<DisplayC>(&mut self, display: &DrawBuffer<DisplayC>)
    where
        DisplayC: PixelColor + Into<Rgb888>,
    {
        let pixel_pitch = (self.output_settings.scale + self.output_settings.pixel_spacing) as i32;
        let pixel_size = Size::new(self.output_settings.scale, self.output_settings.scale);

        for p in display.bounding_box().points() {
            let raw_color = display.get_pixel(p).into();
            let themed_color = self.output_settings.theme.convert(raw_color);
            let output_color = C::from(themed_color).to_be_bytes();
            let output_color = output_color.as_ref();

            for p in Rectangle::new(p * pixel_pitch, pixel_size).points() {
                if let Ok((x, y)) = <(u32, u32)>::try_from(p) {
                    let start_index = (x + y * self.size.width) as usize * output_color.len();

                    self.data[start_index..start_index + output_color.len()]
                        .copy_from_slice(output_color)
                }
            }
        }
    }
}

impl<C> OriginDimensions for OutputImage<C> {
    fn size(&self) -> Size {
        self.size
    }
}

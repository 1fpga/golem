use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::platform::sdl::output::OutputImage;
use crate::macguiver::platform::sdl::SdlPlatform;
use crate::macguiver::platform::PlatformWindow;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::raw::ToBytes;
use embedded_graphics::pixelcolor::{PixelColor, Rgb888};
use sdl3::rect::Point as SdlPoint;

mod sdl_window;
use sdl_window::SdlWindow;

pub struct Window<C: PixelColor> {
    framebuffer: OutputImage<Rgb888>,
    inner: SdlWindow,

    phantom: std::marker::PhantomData<C>,
}

impl<C: PixelColor> Window<C> {
    pub fn size(&self) -> Size {
        self.inner.size()
    }

    pub fn position(&self) -> Point {
        let pt = self.inner.position();
        Point::new(pt.x, pt.y)
    }

    pub fn set_position(&mut self, pos: Point) {
        self.inner.set_position(SdlPoint::new(pos.x, pos.y));
    }

    pub fn focus(&mut self) {
        self.inner.focus();
    }
}

impl<C: PixelColor + From<Rgb888> + Into<Rgb888>> Window<C> {
    pub fn new(platform: &mut SdlPlatform<C>, title: &str, size: Size) -> Self
    where
        <<C as PixelColor>::Raw as ToBytes>::Bytes: AsRef<[u8]>,
        <C as embedded_graphics::prelude::PixelColor>::Raw: From<C>,
    {
        let framebuffer = OutputImage::new::<C>(size, &platform.init_state.output_settings);
        let inner = SdlWindow::new(platform, title, size);

        Self {
            framebuffer,
            inner,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<C> PlatformWindow for Window<C>
where
    C: PixelColor + Into<Rgb888> + From<Rgb888>,
    <<C as PixelColor>::Raw as ToBytes>::Bytes: AsRef<[u8]>,
    <C as PixelColor>::Raw: From<C>,
{
    type Color = C;

    fn update(&mut self, display: &DrawBuffer<Self::Color>) {
        let framebuffer = &mut self.framebuffer;
        let sdl_window = &mut self.inner;

        framebuffer.update(display);
        sdl_window.update(framebuffer);
    }
}

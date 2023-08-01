use crate::macguiver::platform::sdl::output::OutputImage;
use crate::macguiver::platform::sdl::SdlPlatform;
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::{PixelColor, Size},
};
use sdl3::{
    pixels::PixelFormatEnum,
    render::{Canvas, Texture, TextureCreator},
    video::WindowContext,
};

pub struct SdlWindow {
    window: Canvas<sdl3::video::Window>,
    window_texture: SdlWindowTexture,
    size: Size,
}

impl SdlWindow {
    pub(super) fn new<C>(platform: &mut SdlPlatform<C>, title: &str, size: Size) -> Self
    where
        C: PixelColor + Into<Rgb888>,
    {
        let output_settings = &platform.init_state.output_settings;
        let size = output_settings.framebuffer_size(size);

        let window = platform.with(|ctx| {
            ctx.video()
                .unwrap()
                .window(title, size.width, size.height)
                .position_centered()
                .build()
                .unwrap()
        });

        let canvas = window.into_canvas().build().unwrap();

        let window_texture = SdlWindowTextureBuilder {
            texture_creator: canvas.texture_creator(),
            texture_builder: |creator: &TextureCreator<WindowContext>| {
                creator
                    .create_texture_streaming(PixelFormatEnum::RGB24, size.width, size.height)
                    .unwrap()
            },
        }
        .build();

        Self {
            window: canvas,
            window_texture,
            size,
        }
    }

    pub fn update(&mut self, framebuffer: &OutputImage<Rgb888>) {
        let width = self.size.width as usize * 3;
        self.window_texture.with_mut(|fields| {
            fields
                .texture
                .update(None, framebuffer.data.as_ref(), width)
                .unwrap();
        });

        self.window
            .copy(self.window_texture.borrow_texture(), None, None)
            .unwrap();
        self.window.present();
    }
}

#[ouroboros::self_referencing]
struct SdlWindowTexture {
    texture_creator: TextureCreator<WindowContext>,
    #[borrows(texture_creator)]
    #[covariant]
    texture: Texture<'this>,
}

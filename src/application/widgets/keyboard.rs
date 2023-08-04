use crate::macguiver::events::keyboard::KeycodeMap;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::{Baseline, Text};
use embedded_graphics::Drawable;
use embedded_layout::View;

#[derive(Debug)]
pub struct KeyboardTesterWidget {
    position: Point,
    state: KeycodeMap,
}

impl KeyboardTesterWidget {
    pub fn new() -> Self {
        Self {
            state: Default::default(),
            position: Point::zero(),
        }
    }

    pub fn set_state(&mut self, state: KeycodeMap) {
        self.state = state;
    }
}

impl View for KeyboardTesterWidget {
    fn translate_impl(&mut self, by: Point) {
        self.position += by;
    }

    fn bounds(&self) -> Rectangle {
        Rectangle::new(self.position, Size::new(100, 100))
    }
}

impl Drawable for KeyboardTesterWidget {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let keys = self.state.to_string();
        let text = Text::with_baseline(
            &keys,
            Point::new(0, 0),
            MonoTextStyle::new(
                &embedded_graphics::mono_font::ascii::FONT_6X9,
                BinaryColor::On,
            ),
            Baseline::Top,
        );

        text.draw(target)?;
        Ok(())
    }
}

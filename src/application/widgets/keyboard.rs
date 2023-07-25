use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::events::keyboard::KeycodeMap;
use crate::macguiver::widgets::Widget;
use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::text::{Baseline, Text};
use embedded_graphics::Drawable;

#[derive(Debug)]
pub struct KeyboardTesterWidget {
    state: KeycodeMap,
}

impl KeyboardTesterWidget {
    pub fn new() -> Self {
        Self {
            state: Default::default(),
        }
    }

    pub fn set_state(&mut self, state: KeycodeMap) {
        self.state = state;
    }
}

impl Widget for KeyboardTesterWidget {
    type Color = BinaryColor;

    fn draw(&self, target: &mut DrawBuffer<Self::Color>) {
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

        text.draw(target).unwrap();
    }
}

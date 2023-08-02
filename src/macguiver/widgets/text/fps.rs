use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::widgets::Widget;
use embedded_graphics::geometry::{Dimensions, Point, Size};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::text::{Baseline, Text};
use embedded_graphics::Drawable;
use std::collections::VecDeque;
use std::time::Instant;

#[derive(Debug)]
pub struct FpsCounter<const N: usize = 200> {
    ticks: VecDeque<Instant>,
    style: MonoTextStyle<'static, BinaryColor>,
}

impl<const C: usize> FpsCounter<C> {
    pub fn new(style: MonoTextStyle<'static, BinaryColor>) -> Self {
        Self {
            ticks: VecDeque::with_capacity(C),
            style,
        }
    }

    pub fn tick(&mut self) {
        if self.ticks.len() == C {
            self.ticks.pop_front();
        }
        self.ticks.push_back(Instant::now());
    }

    pub fn fps(&self) -> usize {
        let now = Instant::now();
        let mut count = 0;

        for tick in self.ticks.iter() {
            if now.duration_since(*tick).as_secs() < 1 {
                count += 1;
            }
        }

        count
    }

    fn build_text<'a>(&self, text: &'a str) -> Text<'a, MonoTextStyle<'static, BinaryColor>> {
        Text::with_baseline(text, Point::new(0, 0), self.style, Baseline::Top)
    }
}

impl<const C: usize> Widget for FpsCounter<C> {
    type Color = BinaryColor;

    fn size_hint(&self, _parent_size: Size) -> Size {
        self.build_text("000 fps").bounding_box().size
    }

    fn update(&mut self) {
        self.tick();
    }

    fn draw(&self, target: &mut DrawBuffer<Self::Color>) {
        let fps = self.fps();
        self.build_text(&format!("{:3} fps", fps))
            .draw(target)
            .unwrap();
    }
}

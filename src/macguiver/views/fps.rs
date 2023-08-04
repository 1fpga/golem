use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::views::Widget;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, OriginDimensions, Point, Size};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::{Baseline, Text};
use embedded_graphics::transform::Transform;
use embedded_graphics::Drawable;
use num_traits::Zero;
use std::collections::VecDeque;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct FpsCounter<const N: usize = 200> {
    bounds: Rectangle,
    ticks: VecDeque<Instant>,
    style: MonoTextStyle<'static, BinaryColor>,
}

impl<const C: usize> FpsCounter<C> {
    pub fn new(style: MonoTextStyle<'static, BinaryColor>) -> Self {
        Self {
            bounds: Text::with_baseline("000 fps", Point::zero(), style, Baseline::Top)
                .bounding_box(),
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
        let fps = self
            .ticks
            .iter()
            .rev()
            .take_while(|tick| now.duration_since(**tick).as_secs().is_zero())
            .count();

        static mut I: u32 = 0;
        unsafe {
            I += 1;
            if I % 100 == 0 {
                eprintln!("fps: {fps}");
            }
        }
        fps
    }
}

impl<const N: usize> Dimensions for FpsCounter<N> {
    fn bounding_box(&self) -> Rectangle {
        self.bounds
    }
}

impl<const N: usize> Transform for FpsCounter<N> {
    fn translate(&self, by: Point) -> Self {
        let mut new = self.clone();
        Transform::translate_mut(&mut new, by);
        new
    }

    fn translate_mut(&mut self, by: Point) -> &mut Self {
        self.bounds.top_left += by;
        self
    }
}

impl<const C: usize> Widget for FpsCounter<C> {
    type Color = BinaryColor;

    fn update(&mut self) {
        self.tick();
    }

    fn draw(&self, target: &mut DrawBuffer<Self::Color>) {
        Drawable::draw(self, target).unwrap();
    }
}

impl<const C: usize> Drawable for FpsCounter<C> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        Text::with_baseline(
            &format!("{:3} fps", self.fps()),
            self.bounds.top_left,
            self.style,
            Baseline::Top,
        )
        .draw(target)?;
        Ok(())
    }
}

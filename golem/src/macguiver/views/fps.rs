use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, Point};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::renderer::TextRenderer;
use embedded_graphics::text::{Baseline, Text};
use embedded_graphics::transform::Transform;
use embedded_graphics::Drawable;
use num_traits::Zero;
use std::collections::VecDeque;
use std::time::Instant;

#[derive(Debug)]
pub struct FpsCounter<const N: usize = 200> {
    last_fps: usize,
    ticks: VecDeque<Instant>,
    style: MonoTextStyle<'static, BinaryColor>,
}

#[derive(Debug, Clone)]
pub struct FpsCounterView {
    position: Point,
    fps: usize,
    style: MonoTextStyle<'static, BinaryColor>,
}

impl FpsCounterView {
    pub(crate) fn from_fps_counter(fps_counter: &FpsCounter) -> Self {
        Self {
            position: Point::zero(),
            // We update this before drawing, so it's always the right value.
            fps: fps_counter.last_fps,
            style: fps_counter.style,
        }
    }
}

impl Dimensions for FpsCounterView {
    fn bounding_box(&self) -> Rectangle {
        self.style
            .measure_string("000 fps", self.position, Baseline::Top)
            .bounding_box
    }
}

impl Transform for FpsCounterView {
    fn translate(&self, by: Point) -> Self {
        let mut new = self.clone();
        new.translate_mut(by);
        new
    }

    fn translate_mut(&mut self, by: Point) -> &mut Self {
        self.position += by;
        self
    }
}

impl Drawable for FpsCounterView {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        Text::with_baseline(
            &format!("{:3} fps", self.fps),
            self.position,
            self.style,
            Baseline::Top,
        )
        .draw(target)?;

        Ok(())
    }
}

impl<const C: usize> FpsCounter<C> {
    pub fn new(style: MonoTextStyle<'static, BinaryColor>) -> Self {
        Self {
            last_fps: 0,
            ticks: VecDeque::with_capacity(C),
            style,
        }
    }

    pub fn reset(&mut self) {
        self.ticks.clear();
        self.last_fps = 0;
    }

    pub fn tick(&mut self) -> usize {
        if self.ticks.len() == C {
            self.ticks.pop_front();
        }
        self.ticks.push_back(Instant::now());
        self.fps()
    }

    pub fn fps(&self) -> usize {
        let now = Instant::now();
        let fps = self
            .ticks
            .iter()
            .rev()
            .take_while(|tick| now.duration_since(**tick).as_secs().is_zero())
            .count();

        fps
    }

    pub fn update(&mut self) -> bool {
        let fps = self.tick();
        if fps != self.last_fps {
            self.last_fps = fps;
            true
        } else {
            false
        }
    }
}

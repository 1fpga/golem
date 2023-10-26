use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, Point};
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::Size;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::transform::Transform;
use embedded_graphics::Drawable;

pub trait ShouldShow {
    fn should_show(&self) -> bool;
}

impl ShouldShow for bool {
    fn should_show(&self) -> bool {
        *self
    }
}

impl<F: Fn() -> bool> ShouldShow for F {
    fn should_show(&self) -> bool {
        self()
    }
}

pub struct OptionalView<S, I> {
    show: S,
    inner: I,
}

impl<I> From<Option<I>> for OptionalView<bool, I>
where
    I: Default,
{
    fn from(value: Option<I>) -> Self {
        match value {
            None => Self {
                show: false,
                inner: I::default(),
            },
            Some(inner) => Self { show: true, inner },
        }
    }
}

impl<S, I> OptionalView<S, I> {
    pub fn new(show: S, inner: I) -> Self {
        Self { show, inner }
    }
}

impl<I> OptionalView<bool, I> {
    pub fn toggle(&mut self) -> bool {
        self.show = !self.show;
        self.show
    }

    pub fn show(&mut self) {
        self.show = true;
    }

    pub fn hide(&mut self) {
        self.show = false;
    }
}

impl<S, I> Dimensions for OptionalView<S, I>
where
    S: ShouldShow,
    I: Dimensions,
{
    fn bounding_box(&self) -> Rectangle {
        let bb = self.inner.bounding_box();
        if self.show.should_show() {
            bb
        } else {
            // It is important for layouts to keep the top_left corner of the view
            // even if it is not shown. This is why we return a zero sized rectangle
            // properly aligned.
            Rectangle::new(bb.top_left, Size::zero())
        }
    }
}

impl<S, I> Transform for OptionalView<S, I>
where
    S: Copy,
    I: Transform,
{
    fn translate(&self, by: Point) -> Self {
        Self {
            inner: self.inner.translate(by),
            show: self.show,
        }
    }

    fn translate_mut(&mut self, by: Point) -> &mut Self {
        self.inner.translate_mut(by);
        self
    }
}

impl<C, S, I, O> Drawable for OptionalView<S, I>
where
    C: PixelColor,
    S: ShouldShow,
    O: Default,
    I: Drawable<Color = C, Output = O>,
{
    type Color = C;
    type Output = O;

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        if self.show.should_show() {
            self.inner.draw(target)
        } else {
            Ok(O::default())
        }
    }
}

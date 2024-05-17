use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::primitives::Rectangle;
use embedded_layout::View;
use embedded_menu::items::{Marker, MenuListItem};

pub struct OptionalMenuItem<I, R>
where
    R: Default,
    I: MenuListItem<R>,
{
    show: bool,
    item: I,
    _marker: core::marker::PhantomData<R>,
}

impl<I, R> From<Option<I>> for OptionalMenuItem<I, R>
where
    I: MenuListItem<R> + Default,
    R: Default,
{
    fn from(value: Option<I>) -> Self {
        OptionalMenuItem {
            show: value.is_some(),
            item: value.unwrap_or_default(),
            _marker: core::marker::PhantomData,
        }
    }
}

impl<I, R> Marker for OptionalMenuItem<I, R>
where
    I: MenuListItem<R>,
    R: Default,
{
}

impl<I, R> View for OptionalMenuItem<I, R>
where
    I: MenuListItem<R>,
    R: Default,
{
    fn translate_impl(&mut self, by: Point) {
        self.item.translate_impl(by)
    }

    fn bounds(&self) -> Rectangle {
        if self.show {
            self.item.bounds()
        } else {
            Rectangle::new(self.item.bounds().top_left, Size::zero())
        }
    }
}

impl<I, R> MenuListItem<R> for OptionalMenuItem<I, R>
where
    R: Default,
    I: MenuListItem<R>,
{
    fn value_of(&self) -> R {
        if self.show {
            self.item.value_of()
        } else {
            R::default()
        }
    }

    fn interact(&mut self) -> R {
        self.item.interact()
    }

    fn set_style(&mut self, style: &MonoTextStyle<'_, BinaryColor>) {
        self.item.set_style(style)
    }

    fn selectable(&self) -> bool {
        self.show && self.item.selectable()
    }

    fn draw_styled<DIS>(
        &self,
        style: &MonoTextStyle<'static, BinaryColor>,
        display: &mut DIS,
    ) -> Result<(), DIS::Error>
    where
        DIS: DrawTarget<Color = BinaryColor>,
    {
        if self.show {
            self.item.draw_styled(style, display)
        } else {
            Ok(())
        }
    }
}

impl<I, R> OptionalMenuItem<I, R>
where
    R: Default + Copy,
    I: MenuListItem<R>,
{
    pub fn new(show: bool, item: I) -> Self {
        Self {
            show,
            item,
            _marker: core::marker::PhantomData,
        }
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.show = visible;
    }

    pub fn inner(&self) -> &I {
        &self.item
    }
}

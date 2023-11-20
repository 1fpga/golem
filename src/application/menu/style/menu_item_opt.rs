use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::{PixelColor, Rgb888};
use embedded_graphics::primitives::Rectangle;
use embedded_layout::View;
use embedded_menu::interaction::InputAdapterSource;
use embedded_menu::selection_indicator::style::IndicatorStyle;
use embedded_menu::selection_indicator::SelectionIndicatorController;
use embedded_menu::{Marker, MenuItem, MenuStyle};

pub struct OptionalMenuItem<I, R>
where
    R: Default,
    I: MenuItem<R>,
{
    show: bool,
    item: I,
    _marker: core::marker::PhantomData<R>,
}

impl<I, R> From<Option<I>> for OptionalMenuItem<I, R>
where
    I: MenuItem<R> + Default,
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
    I: MenuItem<R>,
    R: Default,
{
}

impl<I, R> View for OptionalMenuItem<I, R>
where
    I: MenuItem<R>,
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

impl<I, R> MenuItem<R> for OptionalMenuItem<I, R>
where
    R: Default,
    I: MenuItem<R>,
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

    fn set_style<C, S, IT, P>(&mut self, style: &MenuStyle<C, S, IT, P, R>)
    where
        C: PixelColor,
        S: IndicatorStyle,
        IT: InputAdapterSource<R>,
        P: SelectionIndicatorController,
    {
        self.item.set_style(style)
    }

    fn title(&self) -> &str {
        if self.show {
            self.item.title()
        } else {
            ""
        }
    }

    fn details(&self) -> &str {
        if self.show {
            self.item.details()
        } else {
            ""
        }
    }

    fn value(&self) -> &str {
        if self.show {
            self.item.value()
        } else {
            ""
        }
    }

    fn selectable(&self) -> bool {
        self.show && self.item.selectable()
    }

    fn draw_styled<C, S, IT, P, DIS>(
        &self,
        style: &MenuStyle<C, S, IT, P, R>,
        display: &mut DIS,
    ) -> Result<(), DIS::Error>
    where
        C: PixelColor + From<Rgb888>,
        S: IndicatorStyle,
        IT: InputAdapterSource<R>,
        P: SelectionIndicatorController,
        DIS: DrawTarget<Color = C>,
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
    I: MenuItem<R>,
{
    pub fn new(show: bool, item: I) -> Self {
        Self {
            show,
            item,
            _marker: core::marker::PhantomData,
        }
    }
}

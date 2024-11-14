use embedded_graphics::draw_target::{DrawTarget, DrawTargetExt};
use embedded_graphics::geometry::{Dimensions, Point, Size};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::Drawable;
use embedded_layout::view_group::ViewGroup;
use embedded_layout::View;
use embedded_menu::collection::MenuItemCollection;
use embedded_menu::interaction::{InputAdapter, InputAdapterSource};
use embedded_menu::selection_indicator::style::IndicatorStyle;
use embedded_menu::selection_indicator::SelectionIndicatorController;
use embedded_menu::theme::Theme;
use embedded_menu::{Menu, MenuState};

pub struct SizedMenu<T, IT, VG, R, C, P, S>
where
    T: AsRef<str>,
    IT: InputAdapterSource<R>,
    C: Theme,
    P: SelectionIndicatorController,
    S: IndicatorStyle,
{
    rectangle: Rectangle,
    menu: Menu<T, IT, VG, R, P, S, C>,
}

impl<T, IT, VG, R, C, P, S> SizedMenu<T, IT, VG, R, C, P, S>
where
    T: AsRef<str>,
    IT: InputAdapterSource<R>,
    C: Theme,
    VG: MenuItemCollection<R>,
    P: SelectionIndicatorController,
    S: IndicatorStyle,
    R: Copy,
{
    pub(crate) fn state(&self) -> MenuState<IT::InputAdapter, P, S> {
        self.menu.state()
    }

    pub(crate) fn selected_value(&self) -> R {
        self.menu.selected_value()
    }
}

impl<T, IT, VG, R, C, P, S> SizedMenu<T, IT, VG, R, C, P, S>
where
    T: AsRef<str>,
    IT: InputAdapterSource<R>,
    C: Theme,
    P: SelectionIndicatorController,
    S: IndicatorStyle,
{
    pub fn new(size: Size, menu: Menu<T, IT, VG, R, P, S, C>) -> Self {
        Self {
            rectangle: Rectangle::new(Point::zero(), size),
            menu,
        }
    }
}

impl<T, IT, VG, R, C, P, S> SizedMenu<T, IT, VG, R, C, P, S>
where
    T: AsRef<str>,
    IT: InputAdapterSource<R>,
    VG: ViewGroup + MenuItemCollection<R>,
    C: Theme,
    P: SelectionIndicatorController,
    S: IndicatorStyle,
{
    pub fn interact(&mut self, input: <IT::InputAdapter as InputAdapter>::Input) -> Option<R> {
        self.menu.interact(input)
    }

    pub fn update(&mut self, display: &impl Dimensions) {
        self.menu.update(display);
    }
}

impl<T, IT, VG, R, C, P, S> View for SizedMenu<T, IT, VG, R, C, P, S>
where
    T: AsRef<str>,
    IT: InputAdapterSource<R>,
    C: Theme,
    P: SelectionIndicatorController,
    S: IndicatorStyle,
{
    fn translate_impl(&mut self, by: Point) {
        self.rectangle.top_left += by;
    }

    fn bounds(&self) -> Rectangle {
        self.rectangle
    }
}

impl<T, IT, VG, R, C, P, S> Drawable for SizedMenu<T, IT, VG, R, C, P, S>
where
    T: AsRef<str>,
    IT: InputAdapterSource<R>,
    VG: ViewGroup + MenuItemCollection<R>,
    C: Theme,
    P: SelectionIndicatorController,
    S: IndicatorStyle,
{
    type Color = C::Color;
    type Output = ();

    fn draw<D>(&self, display: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let mut sub = display.cropped(&self.rectangle);
        self.menu.draw(&mut sub)
    }
}

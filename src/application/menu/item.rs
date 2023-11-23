use crate::application::menu::style::{MenuReturn, SdlMenuAction, SectionSeparator};
use embedded_graphics::geometry::Size;
use embedded_graphics::primitives::{Line, PrimitiveStyle, StyledDrawable};
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::{DrawTarget, PixelColor, Point},
    primitives::Rectangle,
};
use embedded_layout::View;
use embedded_menu::{
    interaction::InputAdapterSource,
    items::MenuLine,
    selection_indicator::{style::IndicatorStyle, SelectionIndicatorController},
    Marker, MenuItem, MenuStyle,
};
use std::fmt::Debug;

pub struct SimpleMenuItem<T, D, M, R>
where
    T: AsRef<str>,
    D: AsRef<str>,
    M: AsRef<str>,
{
    pub title_text: T,
    pub details: D,
    pub return_value: R,
    pub marker: M,
    pub selectable: bool,
    pub disabled: bool,
    pub line: MenuLine,
}

impl<T, D, M, R> Marker for SimpleMenuItem<T, D, M, R>
where
    T: AsRef<str>,
    D: AsRef<str>,
    M: AsRef<str>,
{
}

impl<T, D, M, R> MenuItem<R> for SimpleMenuItem<T, D, M, R>
where
    T: AsRef<str>,
    D: AsRef<str>,
    M: AsRef<str>,
    R: Copy,
{
    fn value_of(&self) -> R {
        self.return_value
    }

    fn interact(&mut self) -> R {
        self.return_value
    }

    fn set_style<C, S, IT, P>(&mut self, style: &MenuStyle<C, S, IT, P, R>)
    where
        C: PixelColor,
        S: IndicatorStyle,
        IT: InputAdapterSource<R>,
        P: SelectionIndicatorController,
    {
        self.line = MenuLine::new(self.marker.as_ref(), style);
    }

    fn title(&self) -> &str {
        self.title_text.as_ref()
    }

    fn details(&self) -> &str {
        self.details.as_ref()
    }

    fn value(&self) -> &str {
        self.marker.as_ref()
    }

    fn selectable(&self) -> bool {
        self.disabled == false && self.selectable
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
        self.line.draw_styled(
            self.title_text.as_ref(),
            self.marker.as_ref(),
            style,
            display,
        )?;
        if self.disabled {
            let mut bound = self.line.bounds();
            bound.size.width = display.bounding_box().size.width;
            let h = bound.size.height as i32;

            for x in (-(h * 2)..bound.size.width as i32).step_by(h as usize) {
                Line::new(
                    Point::new(x, bound.top_left.y),
                    Point::new(x + h, bound.top_left.y + h),
                )
                .draw_styled(
                    &PrimitiveStyle::with_stroke(Rgb888::new(255, 255, 255).into(), 1),
                    display,
                )?;
            }
            for x in (0i32..(bound.size.width as i32 + 16)).step_by(h as usize) {
                Line::new(
                    Point::new(x, bound.top_left.y),
                    Point::new(x - h, bound.top_left.y + h),
                )
                .draw_styled(
                    &PrimitiveStyle::with_stroke(Rgb888::new(255, 255, 255).into(), 1),
                    display,
                )?;
            }

            // let y = (bound.size.height as i32 / 2 + bound.top_left.y);
            //
            // embedded_graphics::primitives::Rectangle::new(
            //     Point::new(bound.top_left.x, y),
            //     Point::new(display.bounding_box().size.width as i32, y),
            // )
            // .draw_styled(
            //     &PrimitiveStyle::with_stroke(Rgb888::new(255, 255, 255).into(), 1),
            //     display,
            // )?;
        }
        Ok(())
    }
}

impl<T, R> SimpleMenuItem<T, &'static str, &'static str, R>
where
    R: MenuReturn,
    T: AsRef<str>,
{
    pub fn new(title: T, value: R) -> Self {
        SimpleMenuItem {
            title_text: title,
            return_value: value,
            details: "",
            marker: "",
            selectable: value.is_selectable(),
            disabled: false,
            line: MenuLine::empty(),
        }
    }
}

impl<T, R> SimpleMenuItem<T, &'static str, &'static str, R>
where
    R: Default,
    T: AsRef<str>,
{
    pub fn unselectable(title: T) -> Self {
        SimpleMenuItem {
            title_text: title,
            return_value: R::default(),
            details: "",
            marker: "",
            selectable: false,
            disabled: false,
            line: MenuLine::empty(),
        }
    }
}

impl<T, D, M, R> SimpleMenuItem<T, D, M, R>
where
    T: AsRef<str>,
    D: AsRef<str>,
    M: AsRef<str>,
{
    pub fn with_marker<M2: AsRef<str>>(self, marker: M2) -> SimpleMenuItem<T, D, M2, R> {
        SimpleMenuItem {
            marker,
            title_text: self.title_text,
            return_value: self.return_value,
            details: self.details,
            line: self.line,
            selectable: self.selectable,
            disabled: self.disabled,
        }
    }

    pub fn disabled(self) -> Self {
        SimpleMenuItem {
            disabled: true,
            ..self
        }
    }
}

impl<T, D, M, R> View for SimpleMenuItem<T, D, M, R>
where
    T: AsRef<str>,
    D: AsRef<str>,
    M: AsRef<str>,
{
    fn translate_impl(&mut self, by: Point) {
        self.line.translate_mut(by);
    }

    fn bounds(&self) -> Rectangle {
        self.line.bounds()
    }
}

pub enum TextMenuItem<'a, R>
where
    R: MenuReturn + Copy,
{
    MenuItem(SimpleMenuItem<&'a str, &'a str, &'a str, SdlMenuAction<R>>),
    Separator(SectionSeparator),
    Empty(Point),
}

impl<'a, R> TextMenuItem<'a, R>
where
    R: MenuReturn + Copy,
{
    pub fn navigation_item(l: &'a str, v: &'a str, r: R) -> Self {
        Self::MenuItem(SimpleMenuItem::new(l, SdlMenuAction::Select(r)).with_marker(v))
    }

    pub fn separator() -> Self {
        TextMenuItem::Separator(SectionSeparator::new())
    }

    pub fn unselectable(title: &'a str) -> Self {
        TextMenuItem::MenuItem(SimpleMenuItem::unselectable(title))
    }

    pub fn empty() -> Self {
        TextMenuItem::Empty(Point::zero())
    }

    pub fn disabled(self) -> Self {
        if let TextMenuItem::MenuItem(item) = self {
            TextMenuItem::MenuItem(item.disabled())
        } else {
            self
        }
    }
}

impl<'a, R> Marker for TextMenuItem<'a, R> where R: MenuReturn + Copy {}

impl<'a, R> View for TextMenuItem<'a, R>
where
    R: MenuReturn + Copy,
{
    fn translate_impl(&mut self, by: Point) {
        match self {
            TextMenuItem::MenuItem(item) => item.translate_impl(by),
            TextMenuItem::Separator(item) => item.translate_impl(by),
            TextMenuItem::Empty(p) => {
                *p += by;
            }
        }
    }

    fn bounds(&self) -> Rectangle {
        match self {
            TextMenuItem::MenuItem(item) => item.bounds(),
            TextMenuItem::Separator(item) => item.bounds(),
            TextMenuItem::Empty(p) => Rectangle::new(*p, Size::zero()),
        }
    }
}

impl<'a, R> MenuItem<SdlMenuAction<R>> for TextMenuItem<'a, R>
where
    R: MenuReturn + Copy,
{
    fn value_of(&self) -> SdlMenuAction<R> {
        match self {
            TextMenuItem::MenuItem(item) => item.value_of(),
            TextMenuItem::Separator(item) => item.value_of(),
            TextMenuItem::Empty(_) => unreachable!(),
        }
    }

    fn interact(&mut self) -> SdlMenuAction<R> {
        match self {
            TextMenuItem::MenuItem(item) => item.interact(),
            TextMenuItem::Separator(item) => item.interact(),
            TextMenuItem::Empty(_) => unreachable!(),
        }
    }

    fn set_style<C, S, IT, P>(&mut self, style: &MenuStyle<C, S, IT, P, SdlMenuAction<R>>)
    where
        C: PixelColor,
        S: IndicatorStyle,
        IT: InputAdapterSource<SdlMenuAction<R>>,
        P: SelectionIndicatorController,
    {
        match self {
            TextMenuItem::MenuItem(item) => item.set_style(style),
            TextMenuItem::Separator(item) => item.set_style(style),
            TextMenuItem::Empty(_) => {}
        }
    }

    fn title(&self) -> &str {
        match self {
            TextMenuItem::MenuItem(item) => item.title(),
            TextMenuItem::Separator(_item) => "",
            TextMenuItem::Empty(_) => "",
        }
    }

    fn details(&self) -> &str {
        match self {
            TextMenuItem::MenuItem(item) => item.details(),
            TextMenuItem::Separator(_item) => "",
            TextMenuItem::Empty(_) => "",
        }
    }

    fn value(&self) -> &str {
        match self {
            TextMenuItem::MenuItem(item) => item.value(),
            TextMenuItem::Separator(_item) => "",
            TextMenuItem::Empty(_) => "",
        }
    }

    fn selectable(&self) -> bool {
        match self {
            TextMenuItem::MenuItem(item) => item.selectable(),
            TextMenuItem::Separator(_item) => false,
            TextMenuItem::Empty(_) => false,
        }
    }

    fn draw_styled<C, S, IT, P, DIS>(
        &self,
        style: &MenuStyle<C, S, IT, P, SdlMenuAction<R>>,
        display: &mut DIS,
    ) -> Result<(), DIS::Error>
    where
        C: PixelColor + From<Rgb888>,
        S: IndicatorStyle,
        IT: InputAdapterSource<SdlMenuAction<R>>,
        P: SelectionIndicatorController,
        DIS: DrawTarget<Color = C>,
    {
        match self {
            TextMenuItem::MenuItem(item) => item.draw_styled(style, display),
            TextMenuItem::Separator(item) => item.draw_styled(style, display),
            TextMenuItem::Empty(_) => Ok(()),
        }
    }
}

pub trait IntoTextMenuItem<'a, R>
where
    R: MenuReturn + Copy,
{
    fn to_menu_item(&'a self) -> TextMenuItem<'a, R>;
}

impl<'a, R> IntoTextMenuItem<'a, R> for TextMenuItem<'a, R>
where
    R: MenuReturn + Copy + Debug,
    Self: 'a,
{
    fn to_menu_item(&'a self) -> TextMenuItem<'a, R> {
        match self {
            TextMenuItem::MenuItem(i) => Self::navigation_item(
                i.title(),
                i.marker,
                match i.value_of() {
                    SdlMenuAction::Select(r) => r,
                    SdlMenuAction::Back => R::back().unwrap(),
                    x => unreachable!("Invalid action: {:?}", x),
                },
            ),
            TextMenuItem::Separator(_) => Self::Separator(SectionSeparator::new()),
            TextMenuItem::Empty(_) => Self::empty(),
        }
    }
}

impl<'a, R> IntoTextMenuItem<'a, R> for (&'a str, &'a str, R)
where
    R: MenuReturn + Copy + 'a,
{
    fn to_menu_item(&self) -> TextMenuItem<'a, R> {
        if self.0.is_empty() {
            TextMenuItem::empty()
        } else if self.0 == "-" {
            TextMenuItem::separator()
        } else if self.2.is_selectable() {
            TextMenuItem::MenuItem(
                SimpleMenuItem::new(self.0, SdlMenuAction::Select(self.2)).with_marker(self.1),
            )
        } else {
            TextMenuItem::MenuItem(SimpleMenuItem::unselectable(self.0).with_marker(self.1))
        }
    }
}

impl<'a, R> IntoTextMenuItem<'a, R> for (&'a str, &'a str)
where
    R: MenuReturn + Copy + Default,
{
    fn to_menu_item(&self) -> TextMenuItem<'a, R> {
        TextMenuItem::MenuItem(SimpleMenuItem::unselectable(self.0).with_marker(self.1))
    }
}

impl<'a, R> IntoTextMenuItem<'a, R> for (&'a str,)
where
    R: MenuReturn + Copy + Default,
{
    fn to_menu_item(&self) -> TextMenuItem<'a, R> {
        TextMenuItem::MenuItem(SimpleMenuItem::unselectable(self.0))
    }
}

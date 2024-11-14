use crate::application::menu::style::{MenuReturn, SdlMenuAction, SectionSeparator};
use embedded_graphics::geometry::Size;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::primitives::{Line, PrimitiveStyle, StyledDrawable};
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::{DrawTarget, Point},
    primitives::Rectangle,
};
use embedded_layout::View;
use embedded_menu::items::{MenuLine, MenuListItem};
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

impl<T, D, M, R> SimpleMenuItem<T, D, M, R>
where
    T: AsRef<str>,
    D: AsRef<str>,
    M: AsRef<str>,
{
    pub fn map_action<F, R2>(self, f: F) -> SimpleMenuItem<T, D, M, R2>
    where
        F: Fn(R) -> R2,
    {
        SimpleMenuItem {
            title_text: self.title_text,
            details: self.details,
            return_value: f(self.return_value),
            marker: self.marker,
            line: self.line,
            selectable: self.selectable,
            disabled: self.disabled,
        }
    }
}

impl<T, D, M, R> embedded_menu::items::Marker for SimpleMenuItem<T, D, M, R>
where
    T: AsRef<str>,
    D: AsRef<str>,
    M: AsRef<str>,
{
}

impl<T, D, M, R> MenuListItem<R> for SimpleMenuItem<T, D, M, R>
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

    fn set_style(&mut self, style: &MonoTextStyle<'_, BinaryColor>) {
        self.line = MenuLine::new(self.marker.as_ref(), style);
    }

    fn selectable(&self) -> bool {
        !self.disabled && self.selectable
    }

    fn draw_styled<DIS>(
        &self,
        style: &MonoTextStyle<'static, BinaryColor>,
        display: &mut DIS,
    ) -> Result<(), DIS::Error>
    where
        DIS: DrawTarget<Color = BinaryColor>,
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
        Self::MenuItem(SimpleMenuItem::unselectable(title))
    }

    pub fn unselectable_with_marker(title: &'a str, marker: &'a str) -> Self {
        Self::MenuItem(SimpleMenuItem::unselectable(title).with_marker(marker))
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

    pub fn title(&self) -> &str {
        match self {
            TextMenuItem::MenuItem(item) => item.title_text,
            TextMenuItem::Separator(_) => "",
            TextMenuItem::Empty(_) => "",
        }
    }
}

impl<'a, R1> TextMenuItem<'a, R1>
where
    R1: MenuReturn + Copy,
{
    pub fn map_action<F, R2>(self, f: F) -> TextMenuItem<'a, R2>
    where
        F: Fn(R1) -> R2,
        R2: MenuReturn + Copy,
    {
        match self {
            TextMenuItem::MenuItem(item @ SimpleMenuItem { return_value, .. }) => {
                match return_value {
                    SdlMenuAction::Select(x) => {
                        TextMenuItem::MenuItem(item.map_action(|_| SdlMenuAction::Select(f(x))))
                    }
                    action => TextMenuItem::MenuItem(SimpleMenuItem {
                        return_value: action.transmute().unwrap(),
                        title_text: item.title_text,
                        details: item.details,
                        marker: item.marker,
                        selectable: item.selectable,
                        disabled: item.disabled,
                        line: item.line,
                    }),
                }
            }
            TextMenuItem::Separator(item) => TextMenuItem::Separator(item),
            TextMenuItem::Empty(p) => TextMenuItem::Empty(p),
        }
    }
}

impl<'a, R> embedded_menu::items::Marker for TextMenuItem<'a, R> where R: MenuReturn + Copy {}

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

impl<'a, R> MenuListItem<SdlMenuAction<R>> for TextMenuItem<'a, R>
where
    R: MenuReturn + Copy,
{
    fn value_of(&self) -> SdlMenuAction<R> {
        match self {
            TextMenuItem::MenuItem(item) => item.value_of(),
            TextMenuItem::Separator(_item) => unreachable!(),
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

    fn set_style(&mut self, style: &MonoTextStyle<'_, BinaryColor>) {
        match self {
            TextMenuItem::MenuItem(item) => item.set_style(style),
            TextMenuItem::Separator(item) => {
                <SectionSeparator as MenuListItem<R>>::set_style(item, style)
            }
            TextMenuItem::Empty(_) => {}
        }
    }

    fn selectable(&self) -> bool {
        match self {
            TextMenuItem::MenuItem(item) => item.selectable(),
            TextMenuItem::Separator(_item) => false,
            TextMenuItem::Empty(_) => false,
        }
    }

    fn draw_styled<DIS>(
        &self,
        style: &MonoTextStyle<'static, BinaryColor>,
        display: &mut DIS,
    ) -> Result<(), DIS::Error>
    where
        DIS: DrawTarget<Color = BinaryColor>,
    {
        match self {
            TextMenuItem::MenuItem(item) => item.draw_styled(style, display),
            TextMenuItem::Separator(item) => {
                <SectionSeparator as MenuListItem<R>>::draw_styled(item, style, display)
            }
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
                i.title_text,
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

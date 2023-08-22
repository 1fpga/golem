use embedded_graphics::mono_font::ascii::FONT_8X13;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_menu::interaction::programmed::Programmed;
use embedded_menu::selection_indicator::style::Invert;
use embedded_menu::selection_indicator::AnimatedPosition;

use embedded_menu::{DisplayScrollbar, MenuStyle};

pub fn menu_style() -> MenuStyle<BinaryColor, Invert, Programmed, AnimatedPosition> {
    MenuStyle::new(BinaryColor::On)
        .with_animated_selection_indicator(2)
        .with_selection_indicator(Invert)
        .with_scrollbar_style(DisplayScrollbar::Auto)
        .with_title_font(&FONT_8X13)
}

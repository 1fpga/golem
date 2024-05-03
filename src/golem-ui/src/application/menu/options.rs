use embedded_graphics::mono_font::MonoFont;

use super::style::MenuReturn;
use super::GolemMenuState;

#[derive(Clone)]
pub struct TextMenuOptions<'a, R: MenuReturn + Copy> {
    pub(super) show_back_menu: bool,
    pub(super) back_label: Option<&'a str>,
    pub(super) show_sort: Option<bool>,
    pub(super) sort_by: Option<&'a str>,
    pub(super) detail_label: Option<&'a str>,
    pub(super) state: Option<GolemMenuState<R>>,
    pub(super) title_font: Option<&'static MonoFont<'static>>,

    /// Prefix items added to the menu before the categorized and sorted section of items.
    pub(super) prefix: &'a [(&'a str, &'a str, R)],
    /// Suffix items added to the menu after the categorized and sorted section of items.
    pub(super) suffix: &'a [(&'a str, &'a str, R)],
}

impl<'a, R: MenuReturn + Copy> Default for TextMenuOptions<'a, R> {
    fn default() -> Self {
        Self {
            show_back_menu: true,
            show_sort: Some(true),
            prefix: &[],
            suffix: &[],
            back_label: None,
            sort_by: None,
            detail_label: None,
            state: None,
            title_font: None,
        }
    }
}

impl<'a, R: MenuReturn + Copy> TextMenuOptions<'a, R> {
    pub fn with_back(self, label: &'a str) -> Self {
        Self {
            show_back_menu: true,
            back_label: Some(label),
            ..self
        }
    }

    pub fn with_back_menu(self, show: bool) -> Self {
        Self {
            show_back_menu: show,
            ..self
        }
    }

    pub fn with_show_sort(self, show: bool) -> Self {
        Self {
            show_sort: Some(show),
            ..self
        }
    }

    pub fn with_title_font(self, font: &'static MonoFont<'static>) -> Self {
        Self {
            title_font: Some(font),
            ..self
        }
    }

    pub fn with_details(self, label: &'a str) -> Self {
        Self {
            detail_label: Some(label),
            ..self
        }
    }

    pub fn with_sort(self, field: &'a str) -> Self {
        Self {
            sort_by: Some(field),
            ..self
        }
    }

    pub fn with_sort_opt(self, field: Option<&'a str>) -> Self {
        Self {
            sort_by: field,
            ..self
        }
    }

    pub fn with_state(self, state: Option<GolemMenuState<R>>) -> Self {
        Self { state, ..self }
    }

    pub fn with_prefix(self, prefix: &'a [(&'a str, &'a str, R)]) -> Self {
        Self { prefix, ..self }
    }

    pub fn with_suffix(self, suffix: &'a [(&'a str, &'a str, R)]) -> Self {
        Self { suffix, ..self }
    }
}

use crate::application::menu::style::{MenuStyleFontSize, MenuStyleOptions};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use strum::Display;

#[derive(Default, Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Display)]
pub enum DateTimeFormat {
    /// The default local format for datetime (respecting Locale).
    #[default]
    #[serde(rename = "default", alias = "Default")]
    Default,

    /// Short locale format.
    #[serde(rename = "short", alias = "Short")]
    Short,

    /// Only show the time.
    #[serde(rename = "timeOnly", alias = "TimeOnly", alias = "time")]
    TimeOnly,

    /// Hide the datetime.
    #[serde(rename = "hidden", alias = "Hidden", alias = "off")]
    Hidden,
}

impl DateTimeFormat {
    pub fn time_format(&self) -> &'static str {
        match self {
            DateTimeFormat::Default => "%c",
            DateTimeFormat::Hidden => "",
            DateTimeFormat::Short => "%x %X",
            DateTimeFormat::TimeOnly => "%X",
        }
    }
}

/// Settings for the UI.
#[derive(Debug, Default, Clone, Copy, Hash)]
pub struct UiSettings {
    show_fps: Option<bool>,

    invert_toolbar: Option<bool>,

    toolbar_datetime_format: Option<DateTimeFormat>,

    menu_font_size: Option<MenuStyleFontSize>,
}

impl UiSettings {
    pub fn show_fps(&self) -> bool {
        self.show_fps.unwrap_or(false)
    }

    pub fn set_show_fps(&mut self, show_fps: bool) {
        self.show_fps = Some(show_fps);
    }

    pub fn invert_toolbar(&self) -> bool {
        self.invert_toolbar.unwrap_or(false)
    }

    pub fn set_invert_toolbar(&mut self, invert_toolbar: bool) {
        self.invert_toolbar = Some(invert_toolbar);
    }

    pub fn toolbar_datetime_format(&self) -> DateTimeFormat {
        self.toolbar_datetime_format
            .unwrap_or(DateTimeFormat::Default)
    }

    pub fn set_toolbar_datetime_format(&mut self, toolbar_datetime_format: DateTimeFormat) {
        self.toolbar_datetime_format = Some(toolbar_datetime_format);
    }

    pub fn menu_font_size(&self) -> MenuStyleFontSize {
        self.menu_font_size.unwrap_or(MenuStyleFontSize::Medium)
    }

    pub fn set_menu_font_size(&mut self, menu_font_size: MenuStyleFontSize) {
        self.menu_font_size = Some(menu_font_size);
    }

    pub fn menu_style_options(&self) -> MenuStyleOptions {
        MenuStyleOptions {
            font_size: self.menu_font_size(),
        }
    }
}

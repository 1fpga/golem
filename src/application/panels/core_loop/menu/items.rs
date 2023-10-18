use crate::application::panels::core_loop::menu::CoreMenuAction;
use crate::utils::config_string::ConfigMenu;
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::{DrawTarget, PixelColor, Point},
    primitives::Rectangle,
};
use embedded_layout::View;
use embedded_menu::interaction::InputAdapterSource;
use embedded_menu::items::MenuLine;
use embedded_menu::selection_indicator::style::IndicatorStyle;
use embedded_menu::selection_indicator::SelectionIndicatorController;
use embedded_menu::{Marker, MenuItem, MenuStyle};
use std::convert::TryFrom;
use std::ops::Range;

pub struct ConfigMenuSelect {
    bits: Range<u8>,
    label: String,
    options: Vec<String>,
    selected: usize,
    line: MenuLine,
}

impl TryFrom<ConfigMenu> for ConfigMenuSelect {
    type Error = ();

    fn try_from(value: ConfigMenu) -> Result<Self, Self::Error> {
        match value {
            ConfigMenu::Option {
                bits,
                label,
                choices,
            } => Ok(Self {
                bits: bits.clone(),
                label: label.clone(),
                options: choices.clone(),
                selected: 0,
                line: MenuLine::empty(),
            }),
            _ => return Err(()),
        }
    }
}

impl Marker for ConfigMenuSelect {}

impl View for ConfigMenuSelect {
    fn translate_impl(&mut self, by: Point) {
        self.line.translate_impl(by)
    }

    fn bounds(&self) -> Rectangle {
        self.line.bounds()
    }
}

impl MenuItem<CoreMenuAction> for ConfigMenuSelect {
    fn interact(&mut self) -> CoreMenuAction {
        self.selected = (self.selected + 1) % self.options.len();
        CoreMenuAction::ToggleOption(self.bits.start, self.bits.end, self.selected)
    }

    fn set_style<C, S, IT, P>(&mut self, style: &MenuStyle<C, S, IT, P, CoreMenuAction>)
    where
        C: PixelColor,
        S: IndicatorStyle,
        IT: InputAdapterSource<CoreMenuAction>,
        P: SelectionIndicatorController,
    {
        if self.options.is_empty() {
            return;
        }

        let longest_str = self.options.iter().max_by_key(|s| s.len()).unwrap();
        self.line = MenuLine::new(longest_str, style);
    }

    fn title(&self) -> &str {
        self.label.as_str()
    }

    fn details(&self) -> &str {
        ""
    }

    fn value(&self) -> &str {
        self.options[self.selected].as_str()
    }

    fn draw_styled<C, ST, IT, P, DIS>(
        &self,
        style: &MenuStyle<C, ST, IT, P, CoreMenuAction>,
        display: &mut DIS,
    ) -> Result<(), DIS::Error>
    where
        C: PixelColor + From<Rgb888>,
        ST: IndicatorStyle,
        IT: InputAdapterSource<CoreMenuAction>,
        P: SelectionIndicatorController,
        DIS: DrawTarget<Color = C>,
    {
        self.line.draw_styled(
            self.label.as_str(),
            self.options[self.selected].as_str(),
            style,
            display,
        )
    }
}

impl ConfigMenuSelect {
    pub fn selected(&self) -> usize {
        self.selected
    }
    pub fn select(&mut self, index: usize) {
        self.selected = index.min(self.options.len() - 1);
    }
    pub fn bits(&self) -> &Range<u8> {
        &self.bits
    }
}

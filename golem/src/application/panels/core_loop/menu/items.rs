// use crate::application::panels::core_loop::menu::CoreMenuAction;
// use crate::utils::config_string::ConfigMenu;
// use embedded_graphics::{
//     pixelcolor::Rgb888,
//     prelude::{DrawTarget, PixelColor, Point},
//     primitives::Rectangle,
// };
// use embedded_layout::View;
// use embedded_menu::interaction::InputAdapterSource;
// use embedded_menu::items::MenuLine;
// use embedded_menu::selection_indicator::style::IndicatorStyle;
// use embedded_menu::selection_indicator::SelectionIndicatorController;
// use embedded_menu::{Marker, MenuItem, MenuStyle};
// use std::convert::TryFrom;
// use std::ops::Range;
//
// #[non_exhaustive]
// pub enum ConfigMenuSelectType {
//     Option {
//         bits: Range<u8>,
//         options: Vec<String>,
//         selected: usize,
//     },
// }
//
// impl ConfigMenuSelectType {
//     #[inline]
//     pub fn select(&mut self, index: usize) {
//         match self {
//             Self::Option {
//                 selected, options, ..
//             } => *selected = index.min(options.len() - 1),
//         }
//     }
//
//     #[inline]
//     pub fn bits(&self) -> Option<&Range<u8>> {
//         match self {
//             Self::Option { bits, .. } => Some(bits),
//         }
//     }
// }
//
// pub struct ConfigMenuSelect {
//     type_: ConfigMenuSelectType,
//     label: String,
//     line: MenuLine,
// }
//
// impl TryFrom<ConfigMenu> for ConfigMenuSelect {
//     type Error = ();
//
//     fn try_from(value: ConfigMenu) -> Result<Self, Self::Error> {
//         match value {
//             ConfigMenu::Option {
//                 bits,
//                 label,
//                 choices,
//             } => Ok(Self {
//                 type_: ConfigMenuSelectType::Option {
//                     bits: bits.clone(),
//                     options: choices.clone(),
//                     selected: 0,
//                 },
//                 label: label.clone(),
//                 line: MenuLine::empty(),
//             }),
//             _ => Err(()),
//         }
//     }
// }
//
// impl Marker for ConfigMenuSelect {}
//
// impl View for ConfigMenuSelect {
//     fn translate_impl(&mut self, by: Point) {
//         self.line.translate_impl(by)
//     }
//
//     fn bounds(&self) -> Rectangle {
//         self.line.bounds()
//     }
// }
//
// impl MenuItem<CoreMenuAction> for ConfigMenuSelect {
//     fn value_of(&self) -> CoreMenuAction {
//         match self {
//             Self {
//                 type_: ConfigMenuSelectType::Option { selected, bits, .. },
//                 ..
//             } => CoreMenuAction::ToggleOption(bits.start, bits.end, *selected),
//         }
//     }
//
//     fn interact(&mut self) -> CoreMenuAction {
//         match self {
//             Self {
//                 type_:
//                     ConfigMenuSelectType::Option {
//                         selected,
//                         options,
//                         bits,
//                         ..
//                     },
//                 ..
//             } => {
//                 *selected = (*selected + 1) % options.len();
//                 CoreMenuAction::ToggleOption(bits.start, bits.end, *selected)
//             }
//         }
//     }
//
//     fn set_style<C, S, IT, P>(&mut self, style: &MenuStyle<C, S, IT, P, CoreMenuAction>)
//     where
//         C: PixelColor,
//         S: IndicatorStyle,
//         IT: InputAdapterSource<CoreMenuAction>,
//         P: SelectionIndicatorController,
//     {
//         match &self.type_ {
//             ConfigMenuSelectType::Option { options, .. } => {
//                 let longest_str = options
//                     .iter()
//                     .max_by_key(|s| s.len())
//                     .map(|s| s.as_str())
//                     .unwrap_or("");
//                 self.line = MenuLine::new(longest_str, style);
//             }
//         }
//     }
//
//     fn title(&self) -> &str {
//         self.label.as_str()
//     }
//
//     fn details(&self) -> &str {
//         ""
//     }
//
//     fn value(&self) -> &str {
//         match &self.type_ {
//             ConfigMenuSelectType::Option {
//                 options, selected, ..
//             } => options[*selected].as_str(),
//         }
//     }
//
//     fn draw_styled<C, ST, IT, P, DIS>(
//         &self,
//         style: &MenuStyle<C, ST, IT, P, CoreMenuAction>,
//         display: &mut DIS,
//     ) -> Result<(), DIS::Error>
//     where
//         C: PixelColor + From<Rgb888>,
//         ST: IndicatorStyle,
//         IT: InputAdapterSource<CoreMenuAction>,
//         P: SelectionIndicatorController,
//         DIS: DrawTarget<Color = C>,
//     {
//         self.line
//             .draw_styled(self.label.as_str(), self.value_text(), style, display)
//     }
// }
//
// impl ConfigMenuSelect {
//     fn value_text(&self) -> &str {
//         match &self.type_ {
//             ConfigMenuSelectType::Option {
//                 options, selected, ..
//             } => options[*selected].as_str(),
//         }
//     }
//
//     pub fn select(&mut self, index: usize) {
//         self.type_.select(index)
//     }
//
//     pub fn bits(&self) -> Option<&Range<u8>> {
//         self.type_.bits()
//     }
// }

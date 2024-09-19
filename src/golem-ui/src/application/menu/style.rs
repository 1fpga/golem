use embedded_graphics::mono_font::ascii;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_menu::interaction::{
    Action, InputAdapter, InputAdapterSource, InputResult, InputState, Interaction, Navigation,
};
use embedded_menu::selection_indicator::AnimatedPosition;
use embedded_menu::theme::Theme;
use embedded_menu::{selection_indicator, DisplayScrollbar, MenuStyle};
use sdl3::event::Event;
use sdl3::gamepad::{Axis, Button};
use sdl3::keyboard::Keycode;
use std::marker::PhantomData;

mod menu_line;
pub use menu_line::*;

mod menu_item_opt;
pub use menu_item_opt::*;

const MENU_ITEMS_PER_PAGE: usize = 10;

/// The action performed by a user. This is used as the return value
/// for the menu.
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub enum SdlMenuAction<R> {
    /// The user pressed the "A" button equivalent.
    Select(R),

    /// The user pressed the "Show Option" button, by default "Y" on a Nintendo controller.
    ShowOptions,

    /// The user pressed a key from the keyboard that is not mapped to a menu action.
    /// This can be used to start a filter action, which is normally not possible
    /// using a controller.
    KeyPress(Keycode),

    /// Text was inputted. Can be up to 32 `c_char`, which can be up to 4 characters.
    TextInput([char; 4]),

    /// The user pressed the "B" button equivalent.
    #[default]
    Back,

    /// The user pressed the "X" button equivalent, which changes the sort order in
    /// a menu where that is enabled.
    ChangeSort,
}

impl<R> SdlMenuAction<R> {
    pub fn transmute<R2>(&self) -> Option<SdlMenuAction<R2>> {
        match self {
            SdlMenuAction::Select(_) => None,
            SdlMenuAction::ShowOptions => Some(SdlMenuAction::ShowOptions),
            SdlMenuAction::KeyPress(kc) => Some(SdlMenuAction::KeyPress(*kc)),
            SdlMenuAction::TextInput(t) => Some(SdlMenuAction::TextInput(*t)),
            SdlMenuAction::Back => Some(SdlMenuAction::Back),
            SdlMenuAction::ChangeSort => Some(SdlMenuAction::ChangeSort),
        }
    }

    pub fn as_text_input(&self) -> Option<String> {
        match self {
            SdlMenuAction::TextInput(t) => {
                let mut s = String::new();
                for c in t.iter() {
                    if *c == 0 as char {
                        break;
                    }
                    s.push(*c);
                }
                Some(s)
            }
            _ => None,
        }
    }
}

/// Return the values for different menu actions.
pub trait MenuReturn: Copy
where
    Self: Sized,
{
    /// Return true if the line should be selectable.
    fn is_selectable(&self) -> bool {
        true
    }

    /// Return a value for the "Back" action.
    fn back() -> Option<Self> {
        None
    }

    /// Value to return when using a "simple" menu and supporting details.
    fn details() -> Option<Self> {
        None
    }

    /// Value to return when doing `details()` on a selection item. By default disable
    /// details.
    fn into_details(self) -> Option<Self> {
        None
    }

    /// Value to return when toggling the sort option.
    fn sort() -> Option<Self> {
        None
    }
}

impl MenuReturn for () {}

impl<T> MenuReturn for SdlMenuAction<T>
where
    T: MenuReturn,
{
    fn back() -> Option<Self> {
        T::back().map(|b| Self::Select(b))
    }

    fn details() -> Option<Self> {
        Some(Self::ShowOptions)
    }

    fn into_details(self) -> Option<Self> {
        match self {
            Self::Select(r) => r.into_details().map(Self::Select),
            _ => None,
        }
    }

    fn sort() -> Option<Self> {
        Some(Self::ChangeSort)
    }
}

#[derive(Clone, Copy)]
pub struct SdlMenuInputAdapter<R: Copy> {
    _phantom: PhantomData<R>,
}

impl<R: Copy> Default for SdlMenuInputAdapter<R> {
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum AxisState {
    #[default]
    Idle,
    Up,
    Down,
}

impl<R: Copy> InputAdapter for SdlMenuInputAdapter<R> {
    type Input = Event;
    type Value = SdlMenuAction<R>;
    type State = (AxisState, AxisState);

    fn handle_input(
        &self,
        state: &mut Self::State,
        action: Self::Input,
    ) -> InputResult<Self::Value> {
        match action {
            Event::KeyDown {
                keycode: Some(code),
                ..
            } => match code {
                Keycode::Escape => Interaction::Action(Action::Return(SdlMenuAction::Back)).into(),
                Keycode::Return => Interaction::Action(Action::Select).into(),
                Keycode::Up => Interaction::Navigation(Navigation::Previous).into(),
                Keycode::Down => Interaction::Navigation(Navigation::Next).into(),
                Keycode::PageDown | Keycode::Right => {
                    Interaction::Navigation(Navigation::Forward(MENU_ITEMS_PER_PAGE)).into()
                }
                Keycode::PageUp | Keycode::Left => {
                    Interaction::Navigation(Navigation::Backward(MENU_ITEMS_PER_PAGE)).into()
                }
                Keycode::Home => Interaction::Navigation(Navigation::Beginning).into(),
                Keycode::End => Interaction::Navigation(Navigation::End).into(),

                Keycode::Tab | Keycode::KpTab => {
                    Interaction::Action(Action::Return(SdlMenuAction::ChangeSort)).into()
                }
                Keycode::Space | Keycode::KpSpace => {
                    Interaction::Action(Action::Return(SdlMenuAction::ShowOptions)).into()
                }

                kc => Interaction::Action(Action::Return(SdlMenuAction::KeyPress(kc))).into(),
            },

            Event::TextInput { text, .. } => {
                let mut ch = text.chars();
                let t = [
                    ch.next().unwrap_or(0 as char),
                    ch.next().unwrap_or(0 as char),
                    ch.next().unwrap_or(0 as char),
                    ch.next().unwrap_or(0 as char),
                ];
                Interaction::Action(Action::Return(SdlMenuAction::TextInput(t))).into()
            }

            Event::ControllerButtonDown { button, .. } => match button {
                Button::A => Interaction::Action(Action::Select).into(),
                Button::B => Interaction::Action(Action::Return(SdlMenuAction::Back)).into(),
                Button::X => Interaction::Action(Action::Return(SdlMenuAction::ShowOptions)).into(),
                Button::Y => Interaction::Action(Action::Return(SdlMenuAction::ChangeSort)).into(),
                Button::DPadUp => Interaction::Navigation(Navigation::Previous).into(),
                Button::DPadDown => Interaction::Navigation(Navigation::Next).into(),
                Button::DPadLeft => {
                    Interaction::Navigation(Navigation::Backward(MENU_ITEMS_PER_PAGE)).into()
                }
                Button::DPadRight => {
                    Interaction::Navigation(Navigation::Forward(MENU_ITEMS_PER_PAGE)).into()
                }

                _ => InputState::Idle.into(),
            },

            Event::ControllerAxisMotion { axis, value, .. } => match axis {
                Axis::LeftX => {
                    if value > i16::MAX / 2 {
                        if state.0 != AxisState::Up {
                            state.0 = AxisState::Up;
                            Interaction::Navigation(Navigation::Backward(MENU_ITEMS_PER_PAGE))
                                .into()
                        } else {
                            InputState::Idle.into()
                        }
                    } else if value < i16::MIN / 2 {
                        if state.0 != AxisState::Down {
                            state.0 = AxisState::Down;
                            Interaction::Navigation(Navigation::Forward(MENU_ITEMS_PER_PAGE)).into()
                        } else {
                            InputState::Idle.into()
                        }
                    } else {
                        state.0 = AxisState::Idle;
                        InputState::Idle.into()
                    }
                }
                Axis::LeftY => {
                    if value > i16::MAX / 2 {
                        if state.1 != AxisState::Up {
                            state.1 = AxisState::Up;
                            Interaction::Navigation(Navigation::Next).into()
                        } else {
                            InputState::Idle.into()
                        }
                    } else if value < i16::MIN / 2 {
                        if state.1 != AxisState::Down {
                            state.1 = AxisState::Down;
                            Interaction::Navigation(Navigation::Previous).into()
                        } else {
                            InputState::Idle.into()
                        }
                    } else {
                        state.1 = AxisState::Idle;
                        InputState::Idle.into()
                    }
                }
                _ => InputState::Idle.into(),
            },

            _ => InputState::Idle.into(),
        }
    }
}

impl<R: MenuReturn + Copy> InputAdapterSource<SdlMenuAction<R>> for SdlMenuInputAdapter<R> {
    type InputAdapter = Self;

    fn adapter(&self) -> Self::InputAdapter {
        *self
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SimpleSdlMenuInputAdapter<R: Copy> {
    _phantom: PhantomData<R>,
}

impl<R: Copy> Default for SimpleSdlMenuInputAdapter<R> {
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<R: Copy + MenuReturn> InputAdapterSource<R> for SimpleSdlMenuInputAdapter<R> {
    type InputAdapter = SimpleSdlMenuInputAdapter<R>;

    fn adapter(&self) -> Self::InputAdapter {
        *self
    }
}

impl<R: Copy + MenuReturn> InputAdapter for SimpleSdlMenuInputAdapter<R> {
    type Input = Event;
    type Value = R;
    type State = ();

    fn handle_input(
        &self,
        _state: &mut Self::State,
        action: Self::Input,
    ) -> InputResult<Self::Value> {
        match action {
            Event::KeyDown {
                keycode: Some(code),
                ..
            } => match code {
                Keycode::Escape => {
                    if let Some(b) = R::back() {
                        Interaction::Action(Action::Return(b)).into()
                    } else {
                        InputState::Idle.into()
                    }
                }
                Keycode::Return => Interaction::Action(Action::Select).into(),
                Keycode::Up => Interaction::Navigation(Navigation::Previous).into(),
                Keycode::Down => Interaction::Navigation(Navigation::Next).into(),
                Keycode::PageDown | Keycode::Right => {
                    Interaction::Navigation(Navigation::Forward(MENU_ITEMS_PER_PAGE)).into()
                }
                Keycode::PageUp | Keycode::Left => {
                    Interaction::Navigation(Navigation::Backward(MENU_ITEMS_PER_PAGE)).into()
                }
                Keycode::Home => Interaction::Navigation(Navigation::Beginning).into(),
                Keycode::End => Interaction::Navigation(Navigation::End).into(),
                _ => InputState::Idle.into(),
            },

            Event::ControllerButtonDown { button, .. } => match button {
                Button::A => Interaction::Action(Action::Select).into(),
                Button::B => {
                    if let Some(b) = R::back() {
                        Interaction::Action(Action::Return(b)).into()
                    } else {
                        InputState::Idle.into()
                    }
                }
                Button::X => {
                    if let Some(b) = R::details() {
                        Interaction::Action(Action::Return(b)).into()
                    } else {
                        InputState::Idle.into()
                    }
                }
                Button::Y => {
                    if let Some(b) = R::sort() {
                        Interaction::Action(Action::Return(b)).into()
                    } else {
                        InputState::Idle.into()
                    }
                }

                Button::DPadUp => Interaction::Navigation(Navigation::Previous).into(),
                Button::DPadDown => Interaction::Navigation(Navigation::Next).into(),
                Button::DPadRight => {
                    Interaction::Navigation(Navigation::Backward(MENU_ITEMS_PER_PAGE)).into()
                }
                Button::DPadLeft => {
                    Interaction::Navigation(Navigation::Forward(MENU_ITEMS_PER_PAGE)).into()
                }

                _ => InputState::Idle.into(),
            },

            _ => InputState::Idle.into(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize, Hash, Eq)]
pub enum MenuStyleFontSize {
    #[serde(rename = "small", alias = "Small")]
    Small,

    #[default]
    #[serde(rename = "medium", alias = "Medium")]
    Medium,

    #[serde(rename = "large", alias = "Large")]
    Large,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct MenuStyleOptions {
    pub(crate) font_size: MenuStyleFontSize,
}

impl MenuStyleOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_font_size(mut self, font_size: MenuStyleFontSize) -> Self {
        self.font_size = font_size;
        self
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct SimpleMenuTheme;

impl Theme for SimpleMenuTheme {
    type Color = BinaryColor;

    fn text_color(&self) -> Self::Color {
        BinaryColor::On
    }

    fn selected_text_color(&self) -> Self::Color {
        BinaryColor::Off
    }

    fn selection_color(&self) -> Self::Color {
        BinaryColor::On
    }
}

pub use selection_indicator::style::rectangle::Rectangle as RectangleIndicator;
use serde::{Deserialize, Serialize};

fn menu_style_inner<I: InputAdapterSource<R> + Default, R>(
    options: MenuStyleOptions,
) -> MenuStyle<RectangleIndicator, I, AnimatedPosition, R, SimpleMenuTheme> {
    let font = match options.font_size {
        MenuStyleFontSize::Small => &ascii::FONT_5X8,
        MenuStyleFontSize::Medium => &ascii::FONT_6X10,
        MenuStyleFontSize::Large => &ascii::FONT_8X13_BOLD,
    };

    MenuStyle::new(SimpleMenuTheme)
        .with_input_adapter(I::default())
        .with_animated_selection_indicator(2)
        .with_selection_indicator(RectangleIndicator)
        .with_scrollbar_style(DisplayScrollbar::Auto)
        .with_title_font(&ascii::FONT_8X13)
        .with_font(font)
}

pub fn menu_style<R: MenuReturn + Copy>(
    options: MenuStyleOptions,
) -> MenuStyle<
    RectangleIndicator,
    SdlMenuInputAdapter<R>,
    AnimatedPosition,
    SdlMenuAction<R>,
    SimpleMenuTheme,
> {
    menu_style_inner(options)
}

pub fn menu_style_simple<R: MenuReturn + Copy>(
    options: MenuStyleOptions,
) -> MenuStyle<RectangleIndicator, SimpleSdlMenuInputAdapter<R>, AnimatedPosition, R, SimpleMenuTheme>
{
    menu_style_inner(options)
}

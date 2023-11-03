use embedded_graphics::mono_font::ascii;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_menu::interaction::{
    Action, InputAdapter, InputAdapterSource, InputResult, InputState, Interaction, Navigation,
};
use embedded_menu::selection_indicator::style::Invert;
use embedded_menu::selection_indicator::AnimatedPosition;
use embedded_menu::{DisplayScrollbar, MenuStyle};
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use std::marker::PhantomData;

mod menu_line;
pub use menu_line::*;

mod menu_item_opt;
pub use menu_item_opt::*;

const MENU_ITEMS_PER_PAGE: usize = 10;

/// The action performed by a user. This is used as the return value
/// for the menu.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum SdlMenuAction<R> {
    /// The user pressed the "A" button equivalent.
    Select(R),

    /// The user pressed the "Show Option" button, by default "Y" on a Nintendo controller.
    ShowOptions,

    /// The user pressed a key from the keyboard that is not mapped to a menu action.
    /// This can be used to start a filter action, which is normally not possible
    /// using a controller.
    KeyPress(Keycode),

    /// The user pressed the "B" button equivalent.
    #[default]
    Back,

    /// The user pressed the "X" button equivalent, which changes the sort order in
    /// a menu where that is enabled.
    ChangeSort,
}

/// Return the values for different menu actions.
pub trait MenuReturn: Copy
where
    Self: Sized,
{
    /// Return a value for the "Back" action.
    fn back() -> Option<Self> {
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

impl<R: Copy> SdlMenuInputAdapter<R> {
    pub fn new() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<R: Copy> InputAdapter for SdlMenuInputAdapter<R> {
    type Input = Event;
    type Value = SdlMenuAction<R>;
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

                kc if kc.name().len() == 1 => {
                    Interaction::Action(Action::Return(SdlMenuAction::KeyPress(kc))).into()
                }
                _ => InputState::Idle.into(),
            },

            Event::JoyButtonDown { button_idx, .. } => match button_idx {
                // A
                0 => Interaction::Action(Action::Select).into(),

                // B
                1 => Interaction::Action(Action::Return(SdlMenuAction::Back)).into(),

                // Up
                11 => Interaction::Navigation(Navigation::Previous).into(),

                // Down
                12 => Interaction::Navigation(Navigation::Next).into(),

                // Right
                13 => Interaction::Navigation(Navigation::Backward(MENU_ITEMS_PER_PAGE)).into(),

                // Left
                14 => Interaction::Navigation(Navigation::Forward(MENU_ITEMS_PER_PAGE)).into(),

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

            Event::JoyButtonDown { button_idx, .. } => match button_idx {
                // A
                0 => Interaction::Action(Action::Select).into(),

                // B
                1 => {
                    if let Some(b) = R::back() {
                        Interaction::Action(Action::Return(b)).into()
                    } else {
                        InputState::Idle.into()
                    }
                }

                // Shoulder Left
                9 => Interaction::Navigation(Navigation::Beginning).into(),

                // Shoulder Right
                10 => Interaction::Navigation(Navigation::End).into(),

                // Up
                11 => Interaction::Navigation(Navigation::Previous).into(),

                // Down
                12 => Interaction::Navigation(Navigation::Next).into(),

                // Right
                13 => Interaction::Navigation(Navigation::Backward(MENU_ITEMS_PER_PAGE)).into(),

                // Left
                14 => Interaction::Navigation(Navigation::Forward(MENU_ITEMS_PER_PAGE)).into(),

                _ => InputState::Idle.into(),
            },

            _ => InputState::Idle.into(),
        }
    }
}

pub fn menu_style<R: MenuReturn + Copy>(
) -> MenuStyle<BinaryColor, Invert, SdlMenuInputAdapter<R>, AnimatedPosition, SdlMenuAction<R>> {
    MenuStyle::new(BinaryColor::On)
        .with_input_adapter(SdlMenuInputAdapter::default())
        .with_animated_selection_indicator(2)
        .with_selection_indicator(Invert)
        .with_scrollbar_style(DisplayScrollbar::Auto)
        .with_title_font(&ascii::FONT_9X15_BOLD)
        .with_font(&ascii::FONT_6X10)
}

pub fn menu_style_simple<R: MenuReturn + Copy>(
) -> MenuStyle<BinaryColor, Invert, SimpleSdlMenuInputAdapter<R>, AnimatedPosition, R> {
    MenuStyle::new(BinaryColor::On)
        .with_input_adapter(SimpleSdlMenuInputAdapter::default())
        .with_animated_selection_indicator(2)
        .with_selection_indicator(Invert)
        .with_scrollbar_style(DisplayScrollbar::Auto)
        .with_title_font(&ascii::FONT_9X15_BOLD)
        .with_font(&ascii::FONT_6X10)
}

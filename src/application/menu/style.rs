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

/// Return the values for different menu actions.
pub trait MenuReturn {
    /// Return a value for the "Back" action.
    fn back() -> Self;
}

#[derive(Clone, Copy)]
pub struct SdlMenuInputAdapter<R: MenuReturn + Copy> {
    _phantom: PhantomData<R>,
}

impl<R: MenuReturn + Copy> Default for SdlMenuInputAdapter<R> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<R: MenuReturn + Clone + Copy> InputAdapter for SdlMenuInputAdapter<R> {
    type Input = Event;
    type Value = R;
    type State = ();

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
                Keycode::Escape => Interaction::Action(Action::Return(R::back())).into(),
                Keycode::Return => Interaction::Action(Action::Select).into(),
                Keycode::Up => Interaction::Navigation(Navigation::Previous).into(),
                Keycode::Down => Interaction::Navigation(Navigation::Next).into(),
                Keycode::PageDown | Keycode::Right => {
                    Interaction::Navigation(Navigation::Forward(12)).into()
                }
                Keycode::PageUp | Keycode::Left => {
                    Interaction::Navigation(Navigation::Backward(12)).into()
                }
                Keycode::Home => Interaction::Navigation(Navigation::Beginning).into(),
                Keycode::End => Interaction::Navigation(Navigation::End).into(),
                _ => InputState::Idle.into(),
            },
            _ => InputState::Idle.into(),
        }
    }
}

impl<R: MenuReturn + Clone + Copy> InputAdapterSource<R> for SdlMenuInputAdapter<R> {
    type InputAdapter = Self;

    fn adapter(&self) -> Self::InputAdapter {
        *self
    }
}

pub fn menu_style<R: MenuReturn + Copy>(
) -> MenuStyle<BinaryColor, Invert, SdlMenuInputAdapter<R>, AnimatedPosition, R> {
    MenuStyle::new(BinaryColor::On)
        .with_input_adapter(SdlMenuInputAdapter::default())
        .with_animated_selection_indicator(2)
        .with_selection_indicator(Invert)
        .with_scrollbar_style(DisplayScrollbar::Auto)
        .with_title_font(&ascii::FONT_9X15_BOLD)
        .with_font(&ascii::FONT_6X9)
}

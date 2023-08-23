use embedded_graphics::mono_font::ascii::FONT_8X13;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_menu::interaction::{InputAdapter, InputState, InteractionType};
use embedded_menu::selection_indicator::style::Invert;
use embedded_menu::selection_indicator::AnimatedPosition;
use embedded_menu::{DisplayScrollbar, MenuStyle};
use sdl3::event::Event;
use sdl3::keyboard::Keycode;

#[derive(Clone, Copy)]
pub struct SdlMenuInputAdapter;

impl InputAdapter for SdlMenuInputAdapter {
    type Input = Event;
    type State = ();

    fn handle_input(&self, _state: &mut Self::State, action: Self::Input) -> InputState {
        match action {
            Event::KeyDown {
                keycode: Some(code),
                ..
            } => match code {
                Keycode::Escape => InputState::Active(InteractionType::SelectItem(usize::MAX)),
                Keycode::Return => InputState::Active(InteractionType::Select),
                Keycode::Up => InputState::Active(InteractionType::Previous),
                Keycode::Down => InputState::Active(InteractionType::Next),
                Keycode::PageUp | Keycode::Right => InputState::Active(InteractionType::Forward(7)),
                Keycode::PageDown | Keycode::Left => {
                    InputState::Active(InteractionType::Backward(7))
                }
                Keycode::Home => InputState::Active(InteractionType::Beginning),
                Keycode::End => InputState::Active(InteractionType::End),
                _ => InputState::Idle,
            },
            _ => InputState::Idle,
        }
    }
}

pub fn menu_style() -> MenuStyle<BinaryColor, Invert, SdlMenuInputAdapter, AnimatedPosition> {
    MenuStyle::new(BinaryColor::On)
        .with_input_adapter(SdlMenuInputAdapter)
        .with_animated_selection_indicator(2)
        .with_selection_indicator(Invert)
        .with_scrollbar_style(DisplayScrollbar::Auto)
        .with_title_font(&FONT_8X13)
}

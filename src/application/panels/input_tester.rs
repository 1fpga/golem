use crate::application::widgets::keyboard::KeyboardTesterWidget;
use crate::application::TopLevelViewType;
use crate::macguiver::application::Application;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::Drawable;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;

pub fn input_tester(app: &mut impl Application<Color = BinaryColor>) -> TopLevelViewType {
    let mut widget = KeyboardTesterWidget::new();

    app.event_loop(|app, mut state| {
        let buffer = app.main_buffer();
        buffer.clear(BinaryColor::Off).unwrap();
        widget.draw(buffer).unwrap();

        for ev in state.events() {
            match ev {
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => match code {
                    Keycode::Escape => {
                        return Some(TopLevelViewType::Quit);
                    }
                    _ => {
                        widget.insert(code.into());
                    }
                },
                Event::KeyUp {
                    keycode: Some(code),
                    ..
                } => {
                    widget.remove(code.into());
                }
                _ => {}
            }
        }
        None
    })
}

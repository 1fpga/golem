use crate::application::TopLevelViewType;
use crate::macguiver::application::Application;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::Drawable;
use embedded_menu::interaction::InteractionType;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;

pub fn settings_panel(app: &mut impl Application<Color = BinaryColor>) -> TopLevelViewType {
    let mut menu = app.settings().menu();

    app.event_loop(move |app, state| {
        let buffer = app.main_buffer();
        buffer.clear(BinaryColor::Off).unwrap();
        menu.update(buffer);
        menu.draw(buffer).unwrap();

        for ev in state.events() {
            if let Event::KeyDown {
                keycode: Some(code),
                ..
            } = ev
            {
                match code {
                    Keycode::Escape => {
                        return Some(TopLevelViewType::MainMenu);
                    }
                    Keycode::Return => {
                        return menu.interact(InteractionType::Select);
                    }
                    Keycode::Up => {
                        menu.interact(InteractionType::Previous);
                    }
                    Keycode::Down => {
                        menu.interact(InteractionType::Next);
                    }
                    _ => {}
                }
            }
        }

        app.settings().update(menu.data());

        None
    })
}

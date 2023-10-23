use crate::macguiver::application::Application;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::Drawable;

pub fn settings_panel(app: &mut impl Application<Color = BinaryColor>) {
    let mut menu = app.settings().menu();

    app.event_loop(move |app, state| {
        let buffer = app.main_buffer();
        buffer.clear(BinaryColor::Off).unwrap();
        menu.update(buffer);
        menu.draw(buffer).unwrap();

        for ev in state.events() {
            if let Some(_) = menu.interact(ev) {
                return Some(());
            }
        }

        app.settings().update(*menu.data());

        None
    })
}

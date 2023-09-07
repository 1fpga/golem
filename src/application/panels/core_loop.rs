use crate::macguiver::application::Application;
use crate::platform::{Core, CoreManager, MiSTerPlatform};
use embedded_graphics::pixelcolor::BinaryColor;
use sdl3::event::Event;

/// Run the core loop and send events to the core.
pub fn run_core_loop(app: &mut impl Application<Color = BinaryColor>, core: impl Core) {
    let settings = app.settings();
    let on_setting_update = settings.on_update();

    let mut menu_key_binding = settings.menu_key_binding();

    // Hide the OSD
    app.platform_mut().core_manager_mut().hide_menu();
    app.event_loop(|app, state| {
        for ev in state.events() {
            eprintln!("event: {ev:?}");

            match ev {
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if menu_key_binding == keycode {
                        app.platform_mut().core_manager_mut().show_menu();
                        return Some(());
                    }
                }
                _ => {}
            }
        }

        None
    })
}

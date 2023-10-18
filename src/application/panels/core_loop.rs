use crate::macguiver::application::Application;
use crate::platform::{Core, CoreManager, MiSTerPlatform};
use embedded_graphics::pixelcolor::BinaryColor;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use tracing::debug;

pub mod menu;

/// Run the core loop and send events to the core.
pub fn run_core_loop(app: &mut impl Application<Color = BinaryColor>, mut core: impl Core) {
    let settings = app.settings();
    let on_setting_update = settings.on_update();

    let mut menu_key_binding = settings.menu_key_binding();

    let mut i = 0;
    // Hide the OSD
    app.platform_mut().core_manager_mut().hide_menu();
    app.event_loop(move |app, state| {
        for ev in state.events() {
            match ev {
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if menu_key_binding == keycode {
                        debug!("Opening core menu");
                        if menu::core_menu(app, &mut core) {
                            return Some(());
                        } else {
                            continue;
                        }
                    }

                    core.send_key(keycode as u8);
                }
                Event::JoyButtonDown {
                    which, button_idx, ..
                } => {
                    core.sdl_joy_button_down((which - 1) as u8, button_idx);
                }
                Event::JoyButtonUp {
                    which, button_idx, ..
                } => {
                    core.sdl_joy_button_up((which - 1) as u8, button_idx);
                }
                _ => {}
            }
        }

        None
    })
}

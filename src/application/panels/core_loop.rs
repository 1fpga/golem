use crate::input::commands::CoreCommands;
use crate::input::InputState;
use crate::macguiver::application::Application;
use crate::platform::{Core, CoreManager, GoLEmPlatform};
use embedded_graphics::pixelcolor::BinaryColor;
use sdl3::event::Event;
use std::time::Instant;
use tracing::{debug, info, trace};

pub mod menu;

fn core_loop(app: &mut impl Application<Color = BinaryColor>, mut core: impl Core) {
    let mut inputs = InputState::default();
    let settings = app.settings();
    let mut on_setting_update = settings.on_update();

    let commands = [
        (
            settings.inner().mappings().show_menu.clone(),
            CoreCommands::ShowCoreMenu,
        ),
        (
            settings.inner().mappings().quit_core.clone(),
            CoreCommands::QuitCore,
        ),
    ];

    let mut i = 0;
    let mut prev = Instant::now();

    // This is a special loop that forwards everything to the core,
    // except for the menu button(s).
    app.event_loop(move |app, state| {
        i += 1;
        // Check for things that might be expensive once every some frames, to reduce lag.
        if i % 100 == 0 {
            let now = Instant::now();
            if on_setting_update.try_recv().is_ok() {
                debug!("Settings updated...");
            }

            // Every 500 frames, show FPS.
            if tracing::enabled!(tracing::Level::TRACE) && i % 500 == 0 {
                trace!("Settings update took {:?}", now.elapsed());
                trace!(
                    "FPS: {}",
                    500.0 / ((now - prev).as_millis() as f32 / 1000.0)
                );
            }

            prev = now;
        }

        for ev in state.events() {
            // debug!(?ev, "Core loop event");
            match ev {
                Event::KeyDown {
                    scancode: Some(scancode),
                    ..
                } => {
                    inputs.key_down(scancode);
                    core.send_key(scancode);
                }
                Event::JoyButtonDown {
                    which, button_idx, ..
                } => {
                    inputs.gamepad_button_down(which, button_idx);
                    core.sdl_joy_button_down((which - 1) as u8, button_idx);
                }
                Event::JoyButtonUp {
                    which, button_idx, ..
                } => {
                    inputs.gamepad_button_up(which, button_idx);
                    core.sdl_joy_button_up((which - 1) as u8, button_idx);
                }
                _ => {}
            }
        }

        // Check if any action needs to be taken.
        for (mapping, command) in &commands {
            if mapping.as_ref().map(|x| x.matches(&inputs)) == Some(true) {
                eprintln!("Mapping {:?} triggered", mapping);
                inputs.clear();

                match command {
                    CoreCommands::ShowCoreMenu => {
                        debug!("Opening core menu");
                        if menu::core_menu(app, &mut core) {
                            return Some(());
                        } else {
                            continue;
                        }
                    }
                    CoreCommands::QuitCore => {
                        debug!("Quitting core");
                        return Some(());
                    }
                }
            }
        }

        None
    });
}

/// Run the core loop and send events to the core.
pub fn run_core_loop(
    app: &mut impl Application<Color = BinaryColor>,
    mut core: impl Core,
    should_show_menu: bool,
) {
    let mut should_run_loop = true;
    debug!("Starting core loop...");

    // Hide the OSD
    app.hide_toolbar();
    if !should_show_menu {
        app.platform_mut().core_manager_mut().hide_menu();
    } else {
        should_run_loop = !menu::core_menu(app, &mut core);
    }

    if should_run_loop {
        core_loop(app, core);
    }

    debug!("Core loop ended");
    info!("Loading Main Menu");
    app.platform_mut().core_manager_mut().load_menu().unwrap();
    app.show_toolbar();
}

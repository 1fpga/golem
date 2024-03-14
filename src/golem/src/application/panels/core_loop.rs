use crate::application::GoLEmApp;
use crate::input::commands::{CommandResult, ShortcutCommand};
use crate::input::shortcut::Shortcut;
use crate::input::InputState;
use crate::platform::GoLEmPlatform;
use golem_core::GolemCore;
use sdl3::event::Event;
use std::time::Instant;
use tracing::{debug, error, info, trace};

pub mod menu;

fn commands_(app: &mut GoLEmApp, core: &GolemCore) -> Vec<(ShortcutCommand, Shortcut)> {
    let settings = app.settings();
    settings
        .inner()
        .mappings()
        .all_commands(core.name())
        .flat_map(|(cmd, shortcut)| shortcut.into_iter().map(move |x| (cmd.clone(), x.clone())))
        .collect::<Vec<_>>()
}

fn core_loop(app: &mut GoLEmApp, mut core: GolemCore) {
    let mut inputs = InputState::default();
    let settings = app.settings();
    let mut on_setting_update = settings.on_update();

    let mut commands = commands_(app, &core);

    let mut i = 0;
    let mut prev = Instant::now();

    let mut should_check_savestates = core.save_states().map(|x| !x.is_empty()).unwrap_or(false);

    // This is a special loop that forwards everything to the core,
    // except for the menu button(s).
    app.event_loop(move |app, state| {
        i += 1;
        // Check for things that might be expensive once every some frames, to reduce lag.
        if i % 100 == 0 {
            let now = Instant::now();
            if on_setting_update.try_recv().is_ok() {
                commands = commands_(app, &core);
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
            match ev {
                Event::KeyDown {
                    scancode: Some(scancode),
                    repeat,
                    ..
                } => {
                    if !repeat {
                        inputs.key_down(scancode);
                        core.key_down(scancode);
                    }
                }
                Event::KeyUp {
                    scancode: Some(scancode),
                    ..
                } => {
                    inputs.key_up(scancode);
                    core.key_up(scancode);
                }
                Event::ControllerButtonDown { which, button, .. } => {
                    inputs.controller_button_down(which, button);
                    core.sdl_button_down((which - 1) as u8, button);
                }
                Event::ControllerButtonUp { which, button, .. } => {
                    inputs.controller_button_up(which, button);
                    core.sdl_button_up((which - 1) as u8, button);
                }
                Event::ControllerAxisMotion {
                    which, axis, value, ..
                } => {
                    inputs.controller_axis_motion(which, axis, value);
                    core.sdl_axis_motion((which - 1) as u8, axis, value);
                }
                _ => {}
            }
        }

        // Check if any action needs to be taken.
        let mut update_commands = false;
        for (command, mapping) in &commands {
            if mapping.matches(&inputs) {
                info!(?mapping, ?inputs, "Command {:?} triggered", command);
                // TODO: do not clear the inputs, instead record inputs in the application.
                inputs.clear();
                update_commands = true;

                match command.execute(app, &mut core) {
                    CommandResult::Ok => {}
                    CommandResult::Err(err) => {
                        error!("Error executing command: {}", err);
                    }
                    CommandResult::QuitCore => return Some(()),
                };
            }
        }
        if update_commands {
            commands = commands_(app, &core);
        }

        // Check Savestates and SD Card every 5 loop. This should still be under every
        // frame, since we approximate 600fps.
        if should_check_savestates && i % 5 == 0 {
            let should_save_savestates = core
                .save_states()
                .map(|x| x.iter().any(|x| x.is_dirty()))
                .unwrap_or(false);

            if should_save_savestates {
                let start = Instant::now();
                let screenshot = core.take_screenshot().ok();

                if let Some(s) = core.save_states() {
                    let mut done_any = false;

                    s.iter_mut()
                        .enumerate()
                        .filter(|(_, ss)| ss.is_dirty())
                        .for_each(|(i, ss)| {
                            match app
                                .coordinator_mut()
                                .create_savestate(i, screenshot.as_ref())
                            {
                                Ok(f) => ss.write_to(f).unwrap(),
                                Err(err) => {
                                    error!(?err, "Error creating savestate. Will stop trying.");
                                    should_check_savestates = false;
                                }
                            }

                            done_any = true;
                        });

                    if done_any {
                        trace!("Saved save states in {}msec.", start.elapsed().as_millis());
                    }
                }
            }
        }

        None
    });
}

/// Run the core loop and send events to the core.
pub fn run_core_loop(app: &mut GoLEmApp, mut core: GolemCore, should_show_menu: bool) {
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

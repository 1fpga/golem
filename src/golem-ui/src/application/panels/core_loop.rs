use std::time::Instant;

use sdl3::event::Event;
use tracing::{debug, error, info, trace};

use one_fpga::{Core, GolemCore};

use crate::application::GoLEmApp;
use crate::input::commands::{CommandResult, ShortcutCommand};
use crate::input::shortcut::Shortcut;
use crate::input::InputState;

pub mod menu;

fn commands_(app: &mut GoLEmApp, core: &GolemCore) -> Vec<(ShortcutCommand, Shortcut)> {
    let settings = app.settings();
    if let Some(mappings) = settings.inner().mappings() {
        mappings
            .all_commands(core.name())
            .flat_map(|(cmd, shortcut)| shortcut.into_iter().map(move |x| (cmd.clone(), x.clone())))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    }
}

fn core_loop(app: &mut GoLEmApp, core: &mut GolemCore) {
    let mut inputs = InputState::default();
    let settings = app.settings();
    let mut on_setting_update = settings.on_update();

    let mut commands = commands_(app, &core);

    let mut i = 0;
    let mut prev = Instant::now();

    let mut should_check_savestates = matches!(core.save_state(0), Ok(Some(_)));

    let trace_enabled = tracing::enabled!(tracing::Level::TRACE);

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
            if trace_enabled && i % 500 == 0 {
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
                        let _ = core.key_down(scancode.into());
                    }
                }
                Event::KeyUp {
                    scancode: Some(scancode),
                    ..
                } => {
                    inputs.key_up(scancode);
                    let _ = core.key_up(scancode.into());
                }
                Event::ControllerButtonDown { which, button, .. } => {
                    inputs.controller_button_down(which, button);
                    let _ = core.gamepad_button_down((which - 1) as usize, button.into());
                }
                Event::ControllerButtonUp { which, button, .. } => {
                    inputs.controller_button_up(which, button);
                    let _ = core.gamepad_button_up((which - 1) as usize, button.into());
                }
                Event::ControllerAxisMotion {
                    which, axis, value, ..
                } => {
                    inputs.controller_axis_motion(which, axis, value);
                    // TODO
                    // let _ = core.sdl_axis_motion((which - 1) as u8, axis, value);
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

                match command.execute(app, core) {
                    CommandResult::Ok => {}
                    CommandResult::Err(err) => {
                        error!("Error executing command: {}", err);
                    }
                    CommandResult::QuitCore => core.quit(),
                };
            }
        }
        if update_commands {
            commands = commands_(app, &core);
        }

        // Check Savestates and SD Card every 5 loop. This should still be under every
        // frame, since we approximate 600fps.
        if should_check_savestates && i % 5 == 0 {
            let mut should_save_savestates = false;

            for i in 0.. {
                match core.save_state(i) {
                    Ok(Some(ss)) => {
                        if ss.is_dirty() {
                            should_save_savestates = true;
                            break;
                        }
                    }
                    Ok(None) | Err(_) => break,
                }
            }

            if should_save_savestates {
                let start = Instant::now();
                let screenshot = core.screenshot().ok();

                for i in 0.. {
                    match core.save_state_mut(i) {
                        Ok(Some(ss)) => {
                            if ss.is_dirty() {
                                match app
                                    .coordinator_mut()
                                    .create_savestate(i, screenshot.as_ref())
                                {
                                    Ok(mut f) => ss.save(&mut f).unwrap(),
                                    Err(err) => {
                                        error!(?err, "Error creating savestate. Will stop trying.");
                                        should_check_savestates = false;
                                    }
                                }
                            }
                        }
                        Ok(None) | Err(_) => break,
                    }
                }

                trace!("Saved save states in {}msec.", start.elapsed().as_millis());
            }
        }

        if core.should_quit() {
            return Some(());
        }

        None
    });
}

/// Run the core loop and send events to the core.
pub fn run_core_loop(app: &mut GoLEmApp, core: &mut GolemCore, should_show_menu: bool) {
    let mut should_run_loop = true;
    debug!("Starting core loop...");

    // Hide the OSD
    app.hide_toolbar();
    if !should_show_menu {
        app.platform_mut().core_manager_mut().hide_menu();
    } else {
        should_run_loop = !menu::core_menu(app, core);
    }

    if should_run_loop {
        core_loop(app, core);
    }

    debug!("Core loop ended");
    info!("Loading Main Menu");
    app.platform_mut().core_manager_mut().load_menu().unwrap();
    app.show_toolbar();
}

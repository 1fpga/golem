use crate::application::GoLEmApp;
use crate::input::commands::CommandId;
use crate::input::shortcut::Shortcut;
use crate::input::InputState;
use image::DynamicImage;
use one_fpga::{Core, GolemCore};
use sdl3::event::Event;
use std::fmt::Debug;
use std::path::Path;
use std::time::Instant;
use tracing::{debug, error, info, trace};

fn commands_(app: &mut GoLEmApp, _core: &GolemCore) -> Vec<(Shortcut, CommandId)> {
    app.commands()
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect::<Vec<_>>()
}

fn core_loop<C, E: Debug>(
    app: &mut GoLEmApp,
    core: &mut GolemCore,
    context: &mut C,
    mut shortcut_handler: impl FnMut(
        &mut GoLEmApp,
        &mut GolemCore,
        Shortcut,
        CommandId,
        &mut C,
    ) -> Result<(), E>,
    mut savestate_handler: impl FnMut(
        &mut GoLEmApp,
        &mut GolemCore,
        Option<&DynamicImage>,
        &[u8],
        &mut C,
    ) -> Result<(), E>,
) -> Result<(), E> {
    let mut inputs = InputState::default();
    let settings = app.settings();
    let mut on_setting_update = settings.on_update();

    let mut commands = commands_(app, core);
    let mut prev = Instant::now();

    let mut should_check_savestates = matches!(core.save_state(0), Ok(Some(_)));

    let trace_enabled = tracing::enabled!(tracing::Level::TRACE);
    let mut i = 0;

    // This is a special loop that forwards everything to the core,
    // except for the menu button(s).
    app.event_loop(move |app, state| {
        i += 1;

        // Check for things that might be expensive once every some second, to reduce lag.
        if prev.elapsed().as_secs() >= 1 {
            let now = Instant::now();
            if on_setting_update.try_recv().is_ok() {
                commands = commands_(app, core);
                debug!("Settings updated...");
            }

            if trace_enabled {
                trace!("Settings update took {:?}", now.elapsed());
                trace!("FPS: ~{}", i);
                i = 0;
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
        for (shortcut, id) in &commands {
            if shortcut.matches(&inputs) {
                info!(?shortcut, ?inputs, "Command {:?} triggered", *id);
                // TODO: do not clear the inputs, instead record inputs in the application.
                inputs.clear();
                update_commands = true;

                if let Err(e) = shortcut_handler(app, core, shortcut.clone(), *id, context) {
                    return Some(Err(e));
                }
            }
        }
        if update_commands {
            commands = commands_(app, core);
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
                                let mut buffer = vec![];
                                match ss.save(&mut buffer) {
                                    Ok(_) => ss,
                                    Err(err) => {
                                        error!(?err, "Error saving savestate. Will stop trying.");
                                        should_check_savestates = false;
                                        break;
                                    }
                                };

                                match savestate_handler(
                                    app,
                                    core,
                                    screenshot.as_ref(),
                                    &buffer,
                                    context,
                                ) {
                                    Ok(_) => {}
                                    Err(err) => {
                                        error!(?err, "Error saving savestate. Will stop trying.");
                                        should_check_savestates = false;
                                        break;
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

        if i % 10 == 0 {
            if core.should_quit() {
                return Some(Ok(()));
            }
        }

        None
    })
}

/// Run the core loop and send events to the core.
pub fn run_core_loop<C, E: Debug>(
    app: &mut GoLEmApp,
    core: &mut GolemCore,
    context: &mut C,
    shortcut_handler: impl FnMut(
        &mut GoLEmApp,
        &mut GolemCore,
        Shortcut,
        CommandId,
        &mut C,
    ) -> Result<(), E>,
    savestate_handler: impl FnMut(
        &mut GoLEmApp,
        &mut GolemCore,
        Option<&DynamicImage>,
        &[u8],
        &mut C,
    ) -> Result<(), E>,
) -> Result<(), E> {
    debug!("Starting core loop...");

    // Hide the OSD
    app.hide_toolbar();
    app.platform_mut().core_manager_mut().hide_osd();

    let mut result = Ok(());
    result = core_loop(app, core, context, shortcut_handler, savestate_handler);

    debug!("Core loop ended");
    info!("Loading Main Menu");
    app.platform_mut().core_manager_mut().load_menu().unwrap();
    app.show_toolbar();

    result
}

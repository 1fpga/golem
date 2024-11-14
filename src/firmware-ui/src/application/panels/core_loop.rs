use crate::application::OneFpgaApp;
use crate::input::commands::CommandId;
use image::DynamicImage;
use one_fpga::{Core, OneFpgaCore};
use sdl3::event::Event;
use std::fmt::Debug;
use std::time::Instant;
use tracing::{debug, error, info, trace};

fn core_loop<C, E: Debug>(
    app: &mut OneFpgaApp,
    core: &mut OneFpgaCore,
    context: &mut C,
    mut shortcut_handler: impl FnMut(
        &mut OneFpgaApp,
        &mut OneFpgaCore,
        CommandId,
        &mut C,
    ) -> Result<(), E>,
    mut savestate_handler: impl FnMut(
        &mut OneFpgaApp,
        &mut OneFpgaCore,
        Option<&DynamicImage>,
        usize,
        &[u8],
        &mut C,
    ) -> Result<(), E>,
) -> Result<(), E> {
    let mut should_check_savestates = matches!(core.save_state(0), Ok(Some(_)));
    let mut i = 0;

    // This is a special loop that forwards everything to the core,
    // except for the menu button(s).
    app.event_loop(move |app, state| {
        i += 1;

        for ev in state.events() {
            match ev {
                Event::KeyDown {
                    scancode: Some(scancode),
                    repeat,
                    ..
                } => {
                    if !repeat {
                        let _ = core.key_down((*scancode).into());
                    }
                }
                Event::KeyUp {
                    scancode: Some(scancode),
                    ..
                } => {
                    let _ = core.key_up((*scancode).into());
                }
                Event::ControllerButtonDown { which, button, .. } => {
                    let _ = core.gamepad_button_down((which - 1) as usize, (*button).into());
                }
                Event::ControllerButtonUp { which, button, .. } => {
                    let _ = core.gamepad_button_up((which - 1) as usize, (*button).into());
                }
                // TODO: this.
                // Event::ControllerAxisMotion {
                //     which, axis, value, ..
                // } => {
                //     let _ = core.axis_motion((which - 1) as u8, *axis, *value);
                // }
                _ => {}
            }
        }

        // Check if any action needs to be taken.
        for id in state.shortcuts() {
            if let Err(e) = shortcut_handler(app, core, id, context) {
                return Some(Err(e));
            }
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
                                if let Err(err) = ss.save(&mut buffer) {
                                    error!(?err, "Error saving savestate. Will stop trying.");
                                    should_check_savestates = false;
                                    break;
                                }

                                if let Err(err) = savestate_handler(
                                    app,
                                    core,
                                    screenshot.as_ref(),
                                    i,
                                    &buffer,
                                    context,
                                ) {
                                    error!(?err, "Error saving savestate. Will stop trying.");
                                    should_check_savestates = false;
                                    break;
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
    app: &mut OneFpgaApp,
    core: &mut OneFpgaCore,
    context: &mut C,
    shortcut_handler: impl FnMut(&mut OneFpgaApp, &mut OneFpgaCore, CommandId, &mut C) -> Result<(), E>,
    savestate_handler: impl FnMut(
        &mut OneFpgaApp,
        &mut OneFpgaCore,
        Option<&DynamicImage>,
        usize,
        &[u8],
        &mut C,
    ) -> Result<(), E>,
) -> Result<(), E> {
    debug!("Starting core loop...");

    // Hide the OSD
    app.hide_toolbar();
    app.platform_mut().core_manager_mut().hide_osd();

    let result = core_loop(app, core, context, shortcut_handler, savestate_handler);

    debug!("Core loop ended");
    info!("Loading Main Menu");
    app.platform_mut().core_manager_mut().load_menu().unwrap();
    app.show_toolbar();

    result
}

use crate::application::GoLEmApp;
use crate::input::commands::CoreCommands;
use crate::input::{BasicInputShortcut, InputState};
use crate::platform::{Core, CoreManager, GoLEmPlatform};
use sdl3::event::Event;
use std::time::Instant;
use tracing::{debug, error, info, trace};

pub mod menu;

fn commands_(app: &mut GoLEmApp, core: &impl Core) -> Vec<(CoreCommands, BasicInputShortcut)> {
    let settings = app.settings();
    settings
        .inner()
        .mappings()
        .global_commands()
        .chain(settings.inner().mappings().core_commands(core.name()))
        .map(|(cmd, shortcut)| (cmd, shortcut.clone()))
        .collect::<Vec<_>>()
}

fn core_loop(app: &mut GoLEmApp, mut core: impl Core) {
    let mut inputs = InputState::default();
    let settings = app.settings();
    let mut on_setting_update = settings.on_update();

    let mut commands = commands_(app, &core);

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
            debug!(?ev, "Core loop event");
            match ev {
                Event::KeyDown {
                    scancode: Some(scancode),
                    repeat,
                    ..
                } => {
                    if !repeat {
                        inputs.key_down(scancode);
                    }
                    core.send_key(scancode);
                }
                Event::KeyUp {
                    scancode: Some(scancode),
                    ..
                } => {
                    inputs.key_up(scancode);
                }
                Event::ControllerButtonDown { which, button, .. } => {
                    inputs.controller_button_down(which, button);
                    core.sdl_button_down((which - 1) as u8, button);
                }
                Event::ControllerButtonUp { which, button, .. } => {
                    inputs.controller_button_up(which, button);
                    core.sdl_button_up((which - 1) as u8, button);
                }
                _ => {}
            }
        }

        // Check if any action needs to be taken.
        let mut update_commands = false;
        for (command, mapping) in &commands {
            if mapping.matches(&inputs) {
                info!(?mapping, ?inputs, "Command {:?} triggered", command);
                inputs.clear();

                match command {
                    CoreCommands::ShowCoreMenu => {
                        debug!("Opening core menu");
                        if menu::core_menu(app, &mut core) {
                            return Some(());
                        }
                        // For some reason this doesn't update properly if we only
                        // use the bus. So regenerate the commands.
                        update_commands = true;
                    }
                    CoreCommands::ResetCore => {
                        debug!("Resetting core");
                        core.status_pulse(0);
                    }
                    CoreCommands::QuitCore => {
                        debug!("Quitting core");
                        return Some(());
                    }
                    CoreCommands::CoreSpecificCommand(id) => {
                        let menu = core.menu_options().iter().find(|m| {
                            eprintln!("{:?} == {} ({:?})", m.id(), id, m.label());
                            m.id() == Some(*id)
                        });
                        let menu = menu.cloned();
                        debug!(?menu, "Sending core-specific command {}", id);
                        if let Some(menu) = menu {
                            match core.trigger_menu(&menu) {
                                Ok(true) => {
                                    debug!("Core-specific command {} triggered", id);
                                }
                                Ok(false) => {
                                    debug!("Core-specific command {} not triggered", id);
                                }
                                Err(e) => {
                                    error!("Error triggering menu: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        }
        if update_commands {
            commands = commands_(app, &core);
        }

        None
    });
}

/// Run the core loop and send events to the core.
pub fn run_core_loop(app: &mut GoLEmApp, mut core: impl Core, should_show_menu: bool) {
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

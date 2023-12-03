use crate::application::panels::alert::alert;
use crate::application::GoLEmApp;
use crate::input::commands::CoreCommands;
use crate::input::{BasicInputShortcut, InputState};
use crate::platform::Core;
use embedded_graphics::mono_font::{ascii, MonoTextStyle};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Line, PrimitiveStyle};
use embedded_graphics::text::Text;
use embedded_layout::layout::linear::{spacing, LinearLayout};
use embedded_layout::prelude::*;
use embedded_text::style::{HeightMode, TextBoxStyleBuilder};
use embedded_text::TextBox;
use sdl3::event::Event;
use tracing::info;

pub fn remap(app: &mut GoLEmApp, core: Option<&(impl Core + ?Sized)>, command: CoreCommands) {
    let mapping = app
        .settings()
        .inner()
        .mappings()
        .for_command(core.map(|x| x.name()), command)
        .cloned();
    let mapping_str = mapping.map(|m| m.to_string());

    // First, ask if the user wants to remap the command, delete it or cancel.
    let choice = alert(
        app,
        &format!("Remapping {}", command),
        if let Some(m) = mapping_str.as_ref() {
            m.as_str()
        } else {
            "Currently unmapped."
        },
        &["Remap", "Delete", "Back"],
    );

    match choice {
        None | Some(2) => {
            return;
        }
        Some(1) => {
            app.settings()
                .inner_mut()
                .mappings_mut()
                .delete(core.map(|x| x.name()), command);
            app.settings().update_done();
            return;
        }
        _ => {}
    }

    let display_area = app.main_buffer().bounding_box();

    let bounds = app.main_buffer().bounding_box();

    let mut input = BasicInputShortcut::default();
    let mut current = InputState::default();
    let mut has_been_set = false;

    app.event_loop(move |app, state| {
        let character_style = u8g2_fonts::U8g2TextStyle::new(
            u8g2_fonts::fonts::u8g2_font_haxrcorp4089_t_cyrillic,
            BinaryColor::On,
        );
        let textbox_style = TextBoxStyleBuilder::new()
            .height_mode(HeightMode::FitToText)
            .alignment(embedded_text::alignment::HorizontalAlignment::Justified)
            .paragraph_spacing(1)
            .build();

        let input_str = input.to_string();
        let text_box =
            TextBox::with_textbox_style(&input_str, bounds, character_style, textbox_style);

        let layout = LinearLayout::vertical(
            Chain::new(Text::new(
                "Press the button you want",
                Point::zero(),
                MonoTextStyle::new(&ascii::FONT_8X13_BOLD, BinaryColor::On),
            ))
            .append(
                Line::new(
                    Point::zero(),
                    Point::new(display_area.bounding_box().size.width as i32, 0),
                )
                .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1)),
            )
            .append(Text::new(
                "Press ESCAPE to cancel.",
                Point::zero(),
                MonoTextStyle::new(&ascii::FONT_8X13_BOLD, BinaryColor::On),
            ))
            .append(
                Line::new(
                    Point::zero(),
                    Point::new(display_area.bounding_box().size.width as i32, 0),
                )
                .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1)),
            )
            .append(text_box),
        )
        .with_alignment(horizontal::Center)
        .with_spacing(spacing::FixedMargin(2))
        .arrange()
        .align_to(&display_area, horizontal::Center, vertical::Top);

        let buffer = app.main_buffer();
        buffer.clear(BinaryColor::Off).unwrap();
        layout.draw(buffer).unwrap();

        for e in state.events() {
            match e {
                Event::KeyDown {
                    scancode: Some(scancode),
                    repeat: false,
                    ..
                } => {
                    if scancode == sdl3::keyboard::Scancode::Escape {
                        return Some(());
                    }
                    input.add_key(scancode);
                    current.key_down(scancode);
                    has_been_set = true;
                }
                Event::KeyUp {
                    scancode: Some(scancode),
                    ..
                } => {
                    current.key_up(scancode);
                }
                Event::ControllerButtonDown { which, button, .. } => {
                    input.add_gamepad_button(button);
                    current.controller_button_down(which, button);
                    has_been_set = true;
                }
                Event::ControllerButtonUp { which, button, .. } => {
                    current.controller_button_up(which, button);
                }
                _ => {}
            }
        }

        if has_been_set && current.is_empty() {
            info!(
                core = core.map(|x| x.name()),
                ?command,
                ?input,
                "Updating mapping."
            );

            if let CoreCommands::CoreSpecificCommand(id) = command {
                if let Some(c) = core {
                    if let Some(label) = c
                        .menu_options()
                        .iter()
                        .find(|o| o.id() == Some(id))
                        .and_then(|o| o.label())
                    {
                        app.settings().inner_mut().mappings_mut().set_core_specific(
                            c.name(),
                            label,
                            input.clone(),
                        );
                    }
                }
            } else {
                app.settings()
                    .inner_mut()
                    .mappings_mut()
                    .set(command, input.clone());
            }
            app.settings().update_done();
            return Some(());
        }

        None
    });
}

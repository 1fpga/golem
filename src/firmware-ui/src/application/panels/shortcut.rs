use crate::application::OneFpgaApp;
use crate::input::shortcut::{Modifiers, Shortcut};
use embedded_graphics::mono_font::{ascii, MonoTextStyle};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Line, PrimitiveStyle};
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use embedded_layout::layout::linear::{spacing, LinearLayout};
use embedded_layout::prelude::*;
use embedded_text::style::{HeightMode, TextBoxStyleBuilder};
use embedded_text::TextBox;
use sdl3::event::Event;
use sdl3::keyboard::Scancode;

pub fn prompt_shortcut(
    app: &mut OneFpgaApp,
    title: &str,
    message: Option<&str>,
) -> Option<Shortcut> {
    let display_area = app.main_buffer().bounding_box();

    let bounds = app.main_buffer().bounding_box();

    let mut shortcut = Shortcut::default();

    app.draw_loop(move |app, state| {
        let character_style = u8g2_fonts::U8g2TextStyle::new(
            u8g2_fonts::fonts::u8g2_font_haxrcorp4089_t_cyrillic,
            BinaryColor::On,
        );
        let textbox_style = TextBoxStyleBuilder::new()
            .height_mode(HeightMode::FitToText)
            .alignment(embedded_text::alignment::HorizontalAlignment::Justified)
            .paragraph_spacing(1)
            .build();

        let input_str = shortcut.to_string();
        let text_box =
            TextBox::with_textbox_style(&input_str, bounds, character_style, textbox_style);

        let layout = LinearLayout::vertical(
            Chain::new(Text::new(
                title,
                Point::zero(),
                MonoTextStyle::new(&ascii::FONT_6X10, BinaryColor::On),
            ))
            .append(
                Line::new(
                    Point::zero(),
                    Point::new(display_area.bounding_box().size.width as i32, 0),
                )
                .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1)),
            )
            .append(Text::new(
                message.unwrap_or("Press ESCAPE to cancel."),
                Point::zero(),
                MonoTextStyle::new(&ascii::FONT_6X10, BinaryColor::On),
            ))
            .append(text_box),
        )
        .with_alignment(horizontal::Center)
        .with_spacing(spacing::FixedMargin(2))
        .arrange()
        .align_to(&display_area, horizontal::Center, vertical::Top);

        let buffer = app.osd_buffer();
        let _ = buffer.clear(BinaryColor::Off);
        let _ = layout.draw(buffer);

        for e in state.events() {
            match e {
                Event::KeyDown {
                    scancode: Some(scancode),
                    repeat: false,
                    ..
                } => match scancode {
                    Scancode::Escape => {
                        return Some(None);
                    }
                    Scancode::LCtrl | Scancode::RCtrl => {
                        shortcut.add_modifier(Modifiers::Ctrl);
                    }
                    Scancode::LShift | Scancode::RShift => {
                        shortcut.add_modifier(Modifiers::Shift);
                    }
                    Scancode::LAlt | Scancode::RAlt => {
                        shortcut.add_modifier(Modifiers::Alt);
                    }
                    Scancode::LGui | Scancode::RGui => {
                        shortcut.add_modifier(Modifiers::Gui);
                    }
                    _ => {
                        shortcut.add_key(*scancode);
                    }
                },
                Event::KeyUp {
                    scancode: Some(_), ..
                } if !shortcut.is_empty() => return Some(Some(shortcut.clone())),
                Event::ControllerButtonDown { button, .. } => {
                    shortcut.add_gamepad_button(*button);
                }
                Event::ControllerButtonUp { .. } if !shortcut.is_empty() => {
                    return Some(Some(shortcut.clone()))
                }
                Event::ControllerAxisMotion { axis, value, .. } => {
                    shortcut.add_axis(*axis, *value);
                }

                _ => {}
            }
        }

        None
    })
}

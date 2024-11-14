use crate::application::OneFpgaApp;
use crate::input::password::InputPassword;
use crate::input::shortcut::AxisValue;
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

pub fn enter_password(
    app: &mut OneFpgaApp,
    title: &str,
    message: &str,
    length: u8,
) -> Option<InputPassword> {
    let display_area = app.osd_buffer().bounding_box();

    let mut password = InputPassword::new();
    let text_style = MonoTextStyle::new(&ascii::FONT_10X20, BinaryColor::On);

    app.draw_loop(move |app, state| {
        let mut password_str = String::new();
        for _ in 0..password.len() {
            password_str.push_str("* ");
        }
        for _ in password.len()..(length as usize) {
            password_str.push_str("? ");
        }

        let character_style = u8g2_fonts::U8g2TextStyle::new(
            u8g2_fonts::fonts::u8g2_font_haxrcorp4089_t_cyrillic,
            BinaryColor::On,
        );
        let textbox_style = TextBoxStyleBuilder::new()
            .height_mode(HeightMode::FitToText)
            .alignment(embedded_text::alignment::HorizontalAlignment::Justified)
            .paragraph_spacing(1)
            .build();

        let message_box =
            TextBox::with_textbox_style(message, display_area, character_style, textbox_style);
        let text_box = Text::new(&password_str, Point::new(12, 12), text_style);

        let layout = LinearLayout::vertical(
            Chain::new(Text::new(
                title,
                Point::zero(),
                MonoTextStyle::new(&ascii::FONT_8X13_BOLD, BinaryColor::On),
            ))
            .append(Text::new(
                "Press any button or key.",
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
            .append(message_box)
            .append(text_box),
        )
        .with_alignment(horizontal::Center)
        .with_spacing(spacing::FixedMargin(2))
        .arrange()
        .align_to(&display_area, horizontal::Center, vertical::Top);

        let buffer = app.osd_buffer();
        buffer.clear(BinaryColor::Off).unwrap();
        layout.draw(buffer).unwrap();

        for e in state.events() {
            match e {
                Event::KeyDown {
                    scancode: Some(scancode),
                    repeat: false,
                    ..
                } => {
                    if *scancode == sdl3::keyboard::Scancode::Escape {
                        return Some(None);
                    }
                    password.add_key(*scancode);
                }
                Event::ControllerButtonDown { button, .. } => {
                    password.add_controller_button(*button);
                }
                Event::ControllerAxisMotion { axis, value, .. } => {
                    let x = AxisValue::from(*value);
                    if !x.is_idle() {
                        password.add_controller_axis(*axis, *value);
                    }
                }
                _ => {}
            }
        }

        if password.len() == length as usize {
            return Some(Some(password.clone()));
        }

        None
    })
}

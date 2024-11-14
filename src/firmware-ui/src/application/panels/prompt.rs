use crate::application::menu::bottom_bar;
use crate::application::OneFpgaApp;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, Point};
use embedded_graphics::mono_font::{ascii, MonoTextStyle};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::primitives::{Line, Primitive, PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use embedded_layout::align::{horizontal, vertical, Align};
use embedded_layout::layout::linear::{spacing, LinearLayout};
use embedded_layout::object_chain::Chain;
use embedded_layout::View;
use embedded_text::style::{HeightMode, TextBoxStyleBuilder};
use embedded_text::TextBox;
use sdl3::event::Event;
use sdl3::gamepad::Button;
use sdl3::keyboard::Keycode;
use std::time::Instant;

pub fn prompt(
    title: &str,
    message: &str,
    text: String,
    max_length: u16,
    app: &mut OneFpgaApp,
) -> Option<String> {
    let display_area = app.main_buffer().bounding_box();

    let character_style = u8g2_fonts::U8g2TextStyle::new(
        u8g2_fonts::fonts::u8g2_font_haxrcorp4089_t_cyrillic,
        BinaryColor::On,
    );
    let messagebox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(embedded_text::alignment::HorizontalAlignment::Left)
        .paragraph_spacing(1)
        .build();

    let text_style = MonoTextStyle::new(&ascii::FONT_8X13, BinaryColor::On);

    let bottom_bar = bottom_bar(Some("Enter"), Some("Back"), None, None, None, None);
    let bottom_row = bottom_bar.size();

    let bottom_area = Rectangle::new(
        display_area.top_left
            + Point::new(
                0,
                display_area.size.height as i32 - bottom_row.height as i32 - 1,
            ),
        bottom_row,
    );

    let message_box = TextBox::with_textbox_style(
        message,
        display_area,
        character_style.clone(),
        messagebox_style,
    );

    let layout = LinearLayout::vertical(
        Chain::new(Text::new(
            title,
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
        .append(
            Line::new(
                Point::zero(),
                Point::new(display_area.bounding_box().size.width as i32, 0),
            )
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1)),
        ),
    )
    .with_alignment(horizontal::Center)
    .with_spacing(spacing::FixedMargin(2))
    .arrange()
    .align_to(&display_area, horizontal::Center, vertical::Top)
    .into_inner();

    let mut result = text;
    let start = Instant::now();

    app.draw_loop(move |app, state| {
        let mut text_box = Text::new(&result, Point::zero(), text_style);
        let text_box_size = text_box.bounding_box().size;
        let layout_bounds = layout.bounds();

        let text_box_y = layout_bounds
            .bottom_right()
            .unwrap_or(layout_bounds.top_left)
            .y
            + 20;

        text_box.position = if text_box_size.width < display_area.size.width - 4 {
            Point::new(2, text_box_y)
        } else {
            Point::new(
                display_area.size.width as i32 - 2 - text_box_size.width as i32,
                text_box_y,
            )
        };

        let buffer = app.osd_buffer();
        let _ = buffer.clear(BinaryColor::Off);
        let _ = layout.draw(buffer);
        let _ = bottom_bar.draw(&mut buffer.sub_buffer(bottom_area));
        let _ = text_box.draw(buffer);

        let delta = start.elapsed().as_millis() as u32;
        if delta % 1000 < 500 {
            let cursor = Line::new(
                Point::new(
                    text_box.position.x + text_box_size.width as i32,
                    text_box_y + 3,
                ),
                Point::new(
                    text_box.position.x + text_box_size.width as i32,
                    text_box_y - 11,
                ),
            )
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1));
            let _ = cursor.draw(buffer);
        }

        for ev in state.events() {
            match ev {
                Event::KeyDown { keycode, .. } => match keycode {
                    Some(Keycode::Backspace) | Some(Keycode::KpBackspace) => {
                        result.pop();
                    }
                    Some(Keycode::Return) | Some(Keycode::KpEnter) | Some(Keycode::Return2) => {
                        return Some(Some(result.clone()));
                    }
                    Some(Keycode::Escape) => {
                        return Some(None);
                    }
                    _ => {}
                },
                Event::TextInput { text, .. } => {
                    if result.len() < max_length as usize {
                        result.push_str(&text);
                    }
                }
                Event::ControllerButtonDown { button, .. } => match button {
                    Button::A => return Some(Some(result.clone())),
                    Button::B => return Some(None),
                    _ => {}
                },
                _ => {}
            }
        }

        None
    })
}

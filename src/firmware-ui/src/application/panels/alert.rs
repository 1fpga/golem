use crate::application::menu::style;
use crate::application::menu::style::MenuReturn;
use crate::application::panels::qrcode::qrcode_alert;
use crate::application::widgets::menu::SizedMenu;
use crate::application::OneFpgaApp;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::mono_font::ascii;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Line, PrimitiveStyle};
use embedded_graphics::text::Text;
use embedded_layout::layout::linear::{spacing, LinearLayout};
use embedded_layout::prelude::*;
use embedded_menu::items::menu_item::SelectValue;
use embedded_menu::items::MenuItem;
use embedded_menu::Menu;
use embedded_text::style::{HeightMode, TextBoxStyleBuilder};
use embedded_text::TextBox;
use std::convert::identity;
use tracing::error;
use url::Url;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum MenuAction {
    #[default]
    Back,
    Select(usize),
}

impl MenuReturn for MenuAction {
    fn back() -> Option<Self> {
        Some(MenuAction::Back)
    }
}

impl SelectValue for MenuAction {
    fn marker(&self) -> &str {
        ""
    }
}

pub fn report_issue(app: &mut OneFpgaApp, error: &impl std::error::Error) {
    let mut url = Url::parse("https://github.com/1fpga/firmware/issues/new").unwrap();
    url.query_pairs_mut()
        .append_pair("title", "Reporting Error")
        .append_pair(
            "body",
            &format!("**Describe what you were doing**\n\n\n{}", error),
        )
        .append_pair("labels", "qr-code");

    qrcode_alert(
        app,
        "Report issue",
        "\
            Using this QR Code will lead you to GitHub to report the issue, \
            including your error message. No other data is shared.\
        ",
        url.as_str(),
    );
}

pub fn show_error(app: &mut OneFpgaApp, error: impl std::error::Error, recoverable: bool) {
    let error_str = error.to_string();
    error!(?error);
    let reboot = if recoverable {
        loop {
            match alert(
                app,
                "An error occurred",
                &error_str,
                &["Report", "Reboot", "Back"],
            ) {
                None | Some(2) => break false,
                Some(1) => break true,
                Some(0) => report_issue(app, &error),
                _ => {}
            }
        }
    } else {
        let _ = alert(app, "Error", &error_str, &["Report"]);
        true
    };

    #[cfg(target_arch = "arm")]
    if reboot {
        unsafe {
            libc::reboot(libc::RB_AUTOBOOT);
        }
    }

    if reboot {
        error!("Rebooting... Just kidding you're on a desktop.");
    }
}

pub fn show(app: &mut OneFpgaApp, title: &str, message: &str) {
    let display_area = app.osd_buffer().bounding_box();

    let character_style = u8g2_fonts::U8g2TextStyle::new(
        u8g2_fonts::fonts::u8g2_font_haxrcorp4089_t_cyrillic,
        BinaryColor::On,
    );
    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(embedded_text::alignment::HorizontalAlignment::Justified)
        .paragraph_spacing(1)
        .build();

    let mut bounds = app.osd_buffer().bounding_box();
    bounds.size.width -= 2;
    let text_box = TextBox::with_textbox_style(message, bounds, character_style, textbox_style);

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
        .append(text_box),
    )
    .with_alignment(horizontal::Center)
    .with_spacing(spacing::FixedMargin(2))
    .arrange()
    .align_to(&display_area, horizontal::Center, vertical::Top)
    .into_inner();

    // Only show once, return immediately.
    app.draw(move |app| {
        let buffer = app.osd_buffer();
        let _ = buffer.clear(BinaryColor::Off);
        let _ = layout.draw(buffer);
    });
}

pub fn alert(app: &mut OneFpgaApp, title: &str, message: &str, choices: &[&str]) -> Option<usize> {
    let display_area = app.main_buffer().bounding_box();

    let mut choices = choices
        .iter()
        .enumerate()
        .map(|(i, ch)| MenuItem::new(ch, MenuAction::Select(i)).with_value_converter(identity))
        .collect::<Vec<_>>();

    let menu = SizedMenu::new(
        Size::new(display_area.size.width - 12, 32),
        Menu::with_style(
            "",
            style::menu_style_simple(app.ui_settings().menu_style_options()),
        )
        .add_menu_items(&mut choices)
        .build(),
    );

    let character_style = u8g2_fonts::U8g2TextStyle::new(
        u8g2_fonts::fonts::u8g2_font_haxrcorp4089_t_cyrillic,
        BinaryColor::On,
    );
    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(embedded_text::alignment::HorizontalAlignment::Justified)
        .paragraph_spacing(1)
        .build();

    let mut bounds = app.main_buffer().bounding_box();
    bounds.size.width -= 2;
    let text_box = TextBox::with_textbox_style(message, bounds, character_style, textbox_style);

    let mut layout = LinearLayout::vertical(
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
        .append(text_box)
        .append(menu),
    )
    .with_alignment(horizontal::Center)
    .with_spacing(spacing::FixedMargin(2))
    .arrange()
    .align_to(&display_area, horizontal::Center, vertical::Top)
    .into_inner();

    app.draw_loop(move |app, state| {
        let buffer = app.osd_buffer();
        let _ = buffer.clear(BinaryColor::Off);
        let _ = layout.draw(buffer);

        let menu = &mut layout.object;
        for ev in state.events() {
            match menu.interact(ev.clone()) {
                None => {}
                Some(MenuAction::Back) => return Some(None),
                Some(MenuAction::Select(idx)) => return Some(Some(idx)),
            }
        }
        menu.update();

        None
    })
}

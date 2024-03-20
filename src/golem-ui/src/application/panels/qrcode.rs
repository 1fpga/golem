use crate::application::menu::style;
use crate::application::menu::style::MenuReturn;
use crate::application::widgets::menu::SizedMenu;
use crate::application::GoLEmApp;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, Point};
use embedded_graphics::image::Image;
use embedded_graphics::mono_font::{ascii, MonoTextStyle};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Line, PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;
use embedded_layout::layout::linear::{spacing, LinearLayout};
use embedded_layout::object_chain::Chain;
use embedded_layout::prelude::*;
use embedded_menu::items::NavigationItem;
use embedded_menu::Menu;
use embedded_text::style::{HeightMode, TextBoxStyleBuilder};
use embedded_text::TextBox;

#[derive(Default, Debug, Clone, Copy)]
pub enum MenuAction {
    #[default]
    Back,
}

impl MenuReturn for MenuAction {
    fn back() -> Option<Self> {
        Some(MenuAction::Back)
    }
}

pub fn qrcode_alert(app: &mut GoLEmApp, title: &str, message: &str, url: &str) {
    let display_area = app.main_buffer().bounding_box();
    let qrcode = qrcode::QrCode::new(url).unwrap();
    let pixmap = qrcode
        .render::<image::Luma<u8>>()
        .dark_color(image::Luma([0u8; 1]))
        .light_color(image::Luma([255; 1]))
        .quiet_zone(false) // disable quiet zone (white border)
        .max_dimensions(128, 128) // adjust colors
        .build();

    // Do some reason rendering the QR code to a pixmap and then reading its
    // pixels into a buffer does not work. Creating a file and reading it does.
    let dir = tempdir::TempDir::new("qrcode").unwrap();
    let path = dir.path().join("qrcode.bmp");
    pixmap.save(&path).unwrap();
    let bmp_content = std::fs::read(&path).unwrap();
    let bmp = tinybmp::Bmp::from_slice(&bmp_content).unwrap();

    let image = Image::new(&bmp, Point::zero());

    let character_style = u8g2_fonts::U8g2TextStyle::new(
        u8g2_fonts::fonts::u8g2_font_haxrcorp4089_t_cyrillic,
        BinaryColor::On,
    );
    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(embedded_text::alignment::HorizontalAlignment::Justified)
        .paragraph_spacing(1)
        .build();

    let bounds = Rectangle::new(Point::zero(), Size::new(128, 100));
    let text_box = TextBox::with_textbox_style(message, bounds, character_style, textbox_style);

    let mut items = [NavigationItem::new("Back", MenuAction::Back)];
    let menu = SizedMenu::new(
        Size::new(64, 32),
        Menu::with_style(" ", style::menu_style_simple())
            .add_items(&mut items)
            .build(),
    );

    let mut layout = LinearLayout::horizontal(
        Chain::new(image).append(
            LinearLayout::vertical(
                Chain::new(Text::new(
                    title,
                    Point::zero(),
                    MonoTextStyle::new(&ascii::FONT_8X13_BOLD, BinaryColor::On),
                ))
                .append(
                    Line::new(
                        Point::zero(),
                        Point::new(display_area.bounding_box().size.width as i32 / 2, 0),
                    )
                    .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1)),
                )
                .append(text_box)
                .append(menu),
            )
            .with_alignment(horizontal::Center)
            .with_spacing(spacing::FixedMargin(2))
            .arrange(),
        ),
    )
    .with_alignment(vertical::Center)
    .arrange()
    .align_to(&display_area, horizontal::Center, vertical::Center);

    app.event_loop(move |app, state| {
        let buffer = app.main_buffer();
        buffer.clear(BinaryColor::Off).unwrap();
        layout.draw(buffer).unwrap();

        let menu = &mut layout.inner_mut().object.inner_mut().object;
        for ev in state.events() {
            match menu.interact(ev) {
                None => {}
                Some(MenuAction::Back) => return Some(()),
            }
        }
        menu.update(buffer);

        None
    });
}

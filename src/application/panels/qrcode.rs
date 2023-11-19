use crate::application::menu::style;
use crate::application::menu::style::MenuReturn;
use crate::application::widgets::menu::SizedMenu;
use crate::macguiver::application::Application;
use bitvec::order::Lsb0;
use bitvec::prelude::Msb0;
use bitvec::vec::BitVec;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, Point};
use embedded_graphics::image;
use embedded_graphics::image::ImageRaw;
use embedded_graphics::mono_font::{ascii, MonoTextStyle};
use embedded_graphics::pixelcolor::raw::{ByteOrder, LittleEndian};
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
use fast_qr::convert::image::ImageBuilder;
use fast_qr::convert::{Builder, Shape};
use fast_qr::QRBuilder;

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

pub fn qrcode_alert(
    app: &mut impl Application<Color = BinaryColor>,
    title: &str,
    message: &str,
    url: &str,
) {
    let display_area = app.main_buffer().bounding_box();
    let qrcode = QRBuilder::new(url).build().unwrap();

    let pixmap = ImageBuilder::default()
        .shape(Shape::Square)
        .margin(2)
        .background_color([0, 0, 0, 0])
        .fit_height(app.main_buffer().size().height)
        .fit_width(app.main_buffer().size().width / 2)
        .to_pixmap(&qrcode);
    let width = pixmap.width();
    let height = pixmap.height();

    let mut pixels: BitVec<_, Msb0> = BitVec::with_capacity((width * height) as usize);
    for p in pixmap.pixels() {
        pixels.push(p.alpha() != 0);
    }
    let image: ImageRaw<BinaryColor, LittleEndian> =
        image::ImageRaw::new(pixels.as_raw_slice(), width);

    let image = image::Image::new(&image, Point::zero());

    let character_style = u8g2_fonts::U8g2TextStyle::new(
        u8g2_fonts::fonts::u8g2_font_haxrcorp4089_t_cyrillic,
        BinaryColor::On,
    );
    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(embedded_text::alignment::HorizontalAlignment::Justified)
        .paragraph_spacing(1)
        .build();

    let bounds = Rectangle::new(Point::zero(), Size::new(128, 64));
    let text_box = TextBox::with_textbox_style(message, bounds, character_style, textbox_style);

    let mut items = [NavigationItem::new("Back", MenuAction::Back)];
    let menu = SizedMenu::new(
        Size::new(48, 24),
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
    .with_alignment(vertical::Top)
    .with_spacing(spacing::FixedMargin(2))
    .arrange()
    .align_to(&display_area, horizontal::Center, vertical::Top);

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

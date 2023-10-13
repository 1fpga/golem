use crate::application::menu::style;
use crate::application::menu::style::MenuReturn;
use crate::application::widgets::menu::SizedMenu;
use crate::application::widgets::text::wrap_text;
use crate::macguiver::application::Application;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::mono_font::ascii;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Line, PrimitiveStyle};
use embedded_graphics::text::Text;
use embedded_layout::layout::linear::{spacing, LinearLayout};
use embedded_layout::prelude::*;
use embedded_menu::items::NavigationItem;
use embedded_menu::Menu;
use tracing::error;

#[derive(Default, Debug, Clone, Copy)]
pub enum MenuAction {
    #[default]
    Back,
    Select(usize),
}

impl MenuReturn for MenuAction {
    fn back() -> Self {
        MenuAction::Back
    }
}

pub fn show_error(app: &mut impl Application<Color = BinaryColor>, error: impl AsRef<str>) {
    let error = error.as_ref();
    error!(?error);
    let _ = alert(app, "Error", error, &["Okay"]);
}

pub fn alert(
    app: &mut impl Application<Color = BinaryColor>,
    title: &str,
    message: &str,
    choices: &[&str],
) -> Option<usize> {
    let display_area = app.main_buffer().bounding_box();

    let mut choices = choices
        .into_iter()
        .enumerate()
        .map(|(i, ch)| NavigationItem::new(ch, MenuAction::Select(i)))
        .collect::<Vec<_>>();

    let menu = SizedMenu::new(
        Size::new(100, 100),
        Menu::with_style(" ", style::menu_style())
            .add_items(&mut choices)
            .build(),
    );

    let text_style = MonoTextStyle::new(&ascii::FONT_8X13, BinaryColor::On);
    let message = wrap_text(
        message,
        display_area.bounding_box().size.width,
        &text_style.font,
    )
    .join("\n");

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
        .append(Text::new(&message, Point::zero(), text_style))
        .append(menu),
    )
    .with_alignment(horizontal::Center)
    .with_spacing(spacing::FixedMargin(2))
    .arrange()
    .align_to(&display_area, horizontal::Center, vertical::Top)
    .into_inner();

    app.event_loop(move |app, state| {
        let buffer = app.main_buffer();
        buffer.clear(BinaryColor::Off).unwrap();
        layout.draw(buffer).unwrap();

        let menu = &mut layout.object;
        for ev in state.events() {
            match menu.interact(ev) {
                None => {}
                Some(MenuAction::Back) => return Some(None),
                Some(MenuAction::Select(idx)) => return Some(Some(idx)),
            }
        }
        menu.update(buffer);

        None
    })
}

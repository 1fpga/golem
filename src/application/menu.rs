use crate::application::menu::style::OptionalMenuItem;
use crate::application::menu::style::{MenuReturn, SdlMenuAction, SectionSeparator};
use crate::application::widgets::menu::SizedMenu;
use crate::macguiver::application::Application;
use embedded_graphics::mono_font::{ascii, MonoTextStyle};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Line, PrimitiveStyle, Rectangle};
use embedded_layout::align::{horizontal, vertical, Align};
use embedded_layout::layout::linear::{spacing, LinearLayout};
use embedded_layout::object_chain::Chain;
use embedded_menu::items::NavigationItem;
use embedded_menu::selection_indicator::style::Invert;
use embedded_menu::selection_indicator::AnimatedPosition;
use embedded_menu::{Menu, MenuState};
use u8g2_fonts::types::{HorizontalAlignment, VerticalPosition};

pub mod cores;
pub mod filesystem;
pub mod games;
pub mod main;
pub mod style;

use crate::application::widgets::controller::ControllerButton;
use crate::application::widgets::opt::OptionalView;
use crate::application::widgets::text::FontRendererView;
use crate::application::widgets::EmptyView;
pub use cores::cores_menu_panel;
pub use main::main_menu;

pub type GolemMenuState<R> = MenuState<style::SdlMenuInputAdapter<R>, AnimatedPosition, Invert>;

#[derive(Clone)]
pub struct TextMenuOptions<'a, R: MenuReturn + Copy> {
    pub sort_by: Option<&'a str>,
    pub detail_label: Option<&'a str>,
    pub state: Option<GolemMenuState<R>>,

    /// Prefix items added to the menu before the categorized and sorted section of items.
    pub prefix: &'a [(&'a str, &'a str, R)],
    /// Suffix items added to the menu after the categorized and sorted section of items.
    pub suffix: &'a [(&'a str, &'a str, R)],
}

impl<'a, R: MenuReturn + Copy> Default for TextMenuOptions<'a, R> {
    fn default() -> Self {
        Self {
            prefix: &[],
            suffix: &[],
            sort_by: None,
            detail_label: None,
            state: None,
        }
    }
}

impl<'a, R: MenuReturn + Copy> TextMenuOptions<'a, R> {
    pub fn with_details(self, label: &'a str) -> Self {
        Self {
            detail_label: Some(label),
            ..self
        }
    }

    pub fn with_sort(self, field: &'a str) -> Self {
        Self {
            sort_by: Some(field),
            ..self
        }
    }

    pub fn with_state(self, state: Option<GolemMenuState<R>>) -> Self {
        Self { state, ..self }
    }

    pub fn with_prefix(self, prefix: &'a [(&'a str, &'a str, R)]) -> Self {
        Self { prefix, ..self }
    }

    pub fn with_suffix(self, suffix: &'a [(&'a str, &'a str, R)]) -> Self {
        Self { suffix, ..self }
    }
}

pub fn text_menu<R: MenuReturn + Copy>(
    app: &mut impl Application<Color = BinaryColor>,
    title: &str,
    items: &[(&str, &str, R)],
    options: TextMenuOptions<R>,
) -> (R, GolemMenuState<R>) {
    let TextMenuOptions {
        sort_by,
        state,
        detail_label,
        prefix,
        suffix,
    } = options;
    let show_back = R::back().is_some();
    let show_details = detail_label.is_some();
    let show_sort = R::sort().is_some();

    fn to_navigation<'a, R: MenuReturn + Copy>(
        (label, marker, result): &(&'a str, &'a str, R),
    ) -> NavigationItem<&'a str, &'a str, &'a str, SdlMenuAction<R>> {
        NavigationItem::new(*label, SdlMenuAction::Select(*result)).with_marker(*marker)
    }

    let mut prefix_items = prefix.iter().map(to_navigation).collect::<Vec<_>>();
    let mut items_items = items.iter().map(to_navigation).collect::<Vec<_>>();
    let mut suffix_items = suffix.iter().map(to_navigation).collect::<Vec<_>>();
    let mut back_items = if show_back {
        vec![NavigationItem::new("Back", SdlMenuAction::Back).with_marker("<-")]
    } else {
        vec![]
    };

    let separator1 = OptionalMenuItem::new(!prefix_items.is_empty(), SectionSeparator::new());
    let separator2 = OptionalMenuItem::new(!suffix_items.is_empty(), SectionSeparator::new());
    let separator3 = OptionalMenuItem::new(show_back, SectionSeparator::new());

    let display_area = app.main_buffer().bounding_box();
    let text_style = MonoTextStyle::new(&ascii::FONT_6X10, BinaryColor::On);
    let bottom_row = Size::new(
        display_area.size.width,
        text_style.font.character_size.height + 4,
    );

    let menu_size = app
        .main_buffer()
        .bounding_box()
        .size
        .saturating_sub(Size::new(0, bottom_row.height));
    let menu = SizedMenu::new(
        menu_size,
        Menu::with_style(title, style::menu_style())
            .add_items(&mut prefix_items)
            .add_item(separator1)
            .add_items(&mut items_items)
            .add_item(separator2)
            .add_items(&mut suffix_items)
            .add_item(separator3)
            .add_items(&mut back_items)
            .build_with_state(state.unwrap_or_default()),
    );

    type Font = u8g2_fonts::fonts::u8g2_font_haxrcorp4089_t_cyrillic;
    let sort_field = format!(
        "Sort{}",
        sort_by.map(|f| format!(" ({f})")).unwrap_or("".to_string())
    );
    let bottom_bar = Chain::new(EmptyView::default())
        .append(ControllerButton::new("a", &ascii::FONT_6X10))
        .append(FontRendererView::new::<Font>(
            VerticalPosition::Baseline,
            HorizontalAlignment::Left,
            "Select",
        ))
        .append(OptionalView::new(
            show_back,
            ControllerButton::new("b", &ascii::FONT_6X10),
        ))
        .append(OptionalView::new(
            show_back,
            FontRendererView::new::<Font>(
                VerticalPosition::Baseline,
                HorizontalAlignment::Left,
                "Back",
            ),
        ))
        .append(OptionalView::new(
            show_details,
            ControllerButton::new("x", &ascii::FONT_6X10),
        ))
        .append(OptionalView::new(
            show_details,
            FontRendererView::new::<Font>(
                VerticalPosition::Baseline,
                HorizontalAlignment::Left,
                detail_label.unwrap_or("Details"),
            ),
        ))
        .append(OptionalView::new(
            show_sort,
            ControllerButton::new("y", &ascii::FONT_6X10),
        ))
        .append(OptionalView::new(
            show_sort,
            FontRendererView::new::<Font>(
                VerticalPosition::Baseline,
                HorizontalAlignment::Left,
                sort_field.as_str(),
            ),
        ));

    let mut layout = LinearLayout::vertical(
        Chain::new(menu)
            .append(
                Line::new(
                    Point::new(0, 0),
                    Point::new(display_area.size.width as i32, 0),
                )
                .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1)),
            )
            .append(
                LinearLayout::horizontal(bottom_bar)
                    .with_spacing(spacing::FixedMargin(2))
                    .arrange(),
            ),
    )
    .with_alignment(horizontal::Left)
    .arrange()
    .align_to(&display_area, horizontal::Center, vertical::Top)
    .into_inner();

    let menu_bounding_box = Rectangle::new(Point::zero(), menu_size);
    let mut selected: Option<R> = None;

    app.event_loop(|app, state| {
        let buffer = app.main_buffer();
        buffer.clear(BinaryColor::Off).unwrap();

        {
            let menu = &mut layout.parent.parent.object;
            menu.update(&menu_bounding_box);
        }

        layout.draw(buffer).unwrap();

        let menu = &mut layout.parent.parent.object;

        for ev in state.events() {
            if let Some(action) = menu.interact(ev) {
                match action {
                    SdlMenuAction::Back => return R::back().map(|b| (b, menu.state())),
                    SdlMenuAction::Select(result) => return Some((result, menu.state())),
                    SdlMenuAction::ChangeSort => return R::sort().map(|r| (r, menu.state())),
                    SdlMenuAction::ShowOptions(result) => {
                        return result.into_details().map(|r| (r, menu.state()))
                    }
                    _ => {}
                }
            }
        }

        None
    })
}

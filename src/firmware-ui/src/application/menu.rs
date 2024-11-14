use embedded_graphics::mono_font::{ascii, MonoTextStyle};
use embedded_graphics::pixelcolor::{BinaryColor, Rgb888};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Line, PrimitiveStyle, Rectangle};
use embedded_layout::align::horizontal;
use embedded_layout::layout::linear::{spacing, LinearLayout};
use embedded_layout::object_chain::Chain;
use embedded_layout::View;
use embedded_menu::selection_indicator::AnimatedPosition;
use embedded_menu::{Menu, MenuState};
use sdl3::keyboard::Keycode;
use std::fmt::Debug;
use tracing::info;
use u8g2_fonts::types::{HorizontalAlignment, VerticalPosition};

// pub use cores::cores_menu_panel;
pub use item::*;
pub use options::*;

use crate::application::menu::style::{MenuReturn, SdlMenuAction, SectionSeparator};
use crate::application::menu::style::{OptionalMenuItem, RectangleIndicator};
use crate::application::widgets::controller::ControllerButton;
use crate::application::widgets::menu::SizedMenu;
use crate::application::widgets::opt::OptionalView;
use crate::application::widgets::text::FontRendererView;
use crate::application::widgets::EmptyView;
use crate::application::OneFpgaApp;
use crate::input::commands::CommandId;

pub mod filesystem;
pub mod item;
pub mod options;
pub mod style;

pub type OneFpgaMenuState<R> =
    MenuState<style::SdlMenuInputAdapter<R>, AnimatedPosition, RectangleIndicator>;

fn bottom_bar_button<'a>(
    name: &'a str,
    label: Option<&'a str>,
) -> impl embedded_layout::view_group::ViewGroup + 'a + Drawable<Color = BinaryColor> {
    type Font = u8g2_fonts::fonts::u8g2_font_haxrcorp4089_t_cyrillic;

    LinearLayout::horizontal(
        Chain::new(OptionalView::new(
            label.is_some(),
            ControllerButton::new(name, &ascii::FONT_6X10),
        ))
        .append(OptionalView::new(
            label.is_some(),
            FontRendererView::new::<Font>(
                VerticalPosition::Baseline,
                HorizontalAlignment::Left,
                label.unwrap_or(name),
            ),
        )),
    )
    .with_spacing(spacing::FixedMargin(2))
    .arrange()
}

pub fn bottom_bar<'a>(
    a_button: Option<&'a str>,
    b_button: Option<&'a str>,
    x_button: Option<&'a str>,
    y_button: Option<&'a str>,
    l_button: Option<&'a str>,
    r_button: Option<&'a str>,
) -> impl embedded_layout::view_group::ViewGroup + 'a + Drawable<Color = BinaryColor> {
    LinearLayout::horizontal(
        Chain::<EmptyView>::new(EmptyView::default())
            .append(bottom_bar_button("a", a_button))
            .append(bottom_bar_button("b", b_button))
            .append(bottom_bar_button("x", x_button))
            .append(bottom_bar_button("y", y_button))
            .append(bottom_bar_button("l", l_button))
            .append(bottom_bar_button("r", r_button)),
    )
    .with_spacing(spacing::FixedMargin(2))
    .arrange()
}

pub fn text_menu<'a, C, R: MenuReturn + Copy, E: Debug>(
    app: &mut OneFpgaApp,
    title: &str,
    items: &'a [impl IntoTextMenuItem<'a, R>],
    options: TextMenuOptions<'a, R>,
    context: &mut C,
    mut shortcut_handler: impl FnMut(&mut OneFpgaApp, CommandId, &mut C) -> Result<(), E>,
) -> Result<(R, OneFpgaMenuState<R>), E> {
    let TextMenuOptions {
        show_back_menu,
        back_label,
        show_sort,
        sort_by,
        state,
        detail_label,
        title_font,
        prefix,
        suffix,
        selected,
    } = options;
    let mut menu_state = state.unwrap_or_default();

    let show_back_button = R::back().is_some() && show_back_menu;
    let show_back = show_back_button && show_back_menu;
    let show_details = detail_label.is_some();
    let show_sort = show_sort.unwrap_or(true) && R::sort().is_some();

    let mut buffer = app.osd_buffer().clone();
    let display_area = buffer.bounding_box();

    let mut prefix_items = prefix
        .iter()
        .map(|item| item.to_menu_item())
        .collect::<Vec<TextMenuItem<_>>>();
    let mut items_items = items
        .iter()
        .map(|item| OptionalMenuItem::new(true, item.to_menu_item()))
        .collect::<Vec<OptionalMenuItem<_, _>>>();
    let mut suffix_items = suffix
        .iter()
        .map(|item| item.to_menu_item())
        .collect::<Vec<TextMenuItem<_>>>();

    let text_style = MonoTextStyle::new(&ascii::FONT_6X10, BinaryColor::On);
    let bottom_row = Size::new(
        display_area.size.width,
        text_style.font.character_size.height + 4,
    );
    let bottom_area = Rectangle::new(
        display_area.top_left
            + Point::new(
                0,
                display_area.size.height as i32 - bottom_row.height as i32 + 1,
            ),
        bottom_row,
    );

    let menu_size = buffer
        .bounding_box()
        .size
        .saturating_sub(Size::new(0, bottom_row.height));

    let show1 = !prefix_items.is_empty();
    let show2 = !items_items.is_empty() && !suffix_items.is_empty();
    let show3 = show_back;

    let mut menu_style = style::menu_style(app.ui_settings().menu_style_options());
    if let Some(font) = title_font {
        menu_style = menu_style.with_title_font(font);
    }

    let mut filter = "".to_string();
    loop {
        let separator1 = OptionalMenuItem::new(show1, SectionSeparator::new());
        let separator2 = OptionalMenuItem::new(show2, SectionSeparator::new());
        let separator3 = OptionalMenuItem::new(show3, SectionSeparator::new());

        let sort_field = format!(
            "Sort{}",
            sort_by.map(|f| format!(" - {f}")).unwrap_or("".to_string())
        );

        let bottom_bar = bottom_bar(
            Some("Select"),
            show_back_button.then_some("Back"),
            show_details.then_some(()).and(detail_label),
            show_sort.then_some(sort_field.as_str()),
            None,
            None,
        );

        for f in items_items.iter_mut() {
            let label = f.inner().title();
            let is_visible = if filter.is_empty() {
                true
            } else if label.is_empty() {
                true
            } else {
                label.to_lowercase().contains(&filter.to_lowercase())
            };
            f.set_visible(is_visible);
        }

        let curr_filter_label = filter.clone();
        type Font = u8g2_fonts::fonts::u8g2_font_haxrcorp4089_t_cyrillic;
        let mut filter_bar = FontRendererView::new::<Font>(
            VerticalPosition::Top,
            HorizontalAlignment::Left,
            curr_filter_label.as_str(),
        );

        // Not sure why, it's translated too high.
        let height = filter_bar.size().height as i32;
        View::translate_mut(&mut filter_bar, Point::new(1, height + 2));

        let back_item = OptionalMenuItem::new(
            show_back,
            SimpleMenuItem::new(back_label.unwrap_or("Back"), SdlMenuAction::Back)
                .with_marker("<-"),
        );

        let prefix_len = prefix_items.len();
        let mut menu = SizedMenu::new(
            menu_size,
            Menu::with_style(title, menu_style)
                .add_menu_items(&mut prefix_items)
                .add_menu_item(separator1)
                .add_menu_items(&mut items_items)
                .add_menu_item(separator2)
                .add_menu_items(&mut suffix_items)
                .add_menu_item(separator3)
                .add_menu_item(back_item)
                .build_with_state(menu_state),
        );

        if let Some(selected) = selected {
            // Need to add prefix items (and separator1 which is always present) to the index.
            let selected = selected + prefix_len as u32 + 1;
            menu.interact(sdl3::event::Event::User {
                timestamp: 0,
                window_id: 0,
                type_: 0,
                code: selected as i32,
                data1: std::ptr::null_mut(),
                data2: std::ptr::null_mut(),
            });
        }

        let mut layout = LinearLayout::vertical(
            Chain::new(menu).append(
                Line::new(
                    Point::new(0, 0),
                    Point::new(display_area.size.width as i32, 0),
                )
                .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1)),
            ),
        )
        .with_alignment(horizontal::Left)
        .arrange();

        let (result, new_state) = app.draw_loop(|app, state| {
            let menu_bounding_box = Rectangle::new(Point::zero(), menu_size);
            let _ = buffer.clear(Rgb888::BLACK.into());

            {
                let menu = &mut layout.inner_mut().parent.object;
                menu.update(&menu_bounding_box);
            }
            let _ = layout.draw(&mut buffer.color_converted());

            if filter.is_empty() {
                bottom_bar
                    .draw(&mut buffer.sub_buffer(bottom_area).color_converted())
                    .unwrap();
            } else {
                filter_bar
                    .draw(&mut buffer.sub_buffer(bottom_area).color_converted())
                    .unwrap();
            }

            let menu = &mut layout.inner_mut().parent.object;

            for ev in state.events() {
                // TODO: remove clone.
                if let Some(action) = menu.interact(ev.clone()) {
                    match action {
                        SdlMenuAction::Back => {
                            return R::back().map(|b| Ok((Some(*&b), menu.state().clone())))
                        }
                        SdlMenuAction::Select(result) => {
                            return Some(Ok((Some(result), menu.state().clone())));
                        }
                        SdlMenuAction::ChangeSort => {
                            return R::sort().map(|r| Ok((Some(r), menu.state().clone())));
                        }
                        SdlMenuAction::ShowOptions => {
                            if let SdlMenuAction::Select(r) = menu.selected_value() {
                                return r
                                    .into_details()
                                    .map(|r| Ok((Some(r), menu.state().clone())));
                            }
                        }
                        SdlMenuAction::KeyPress(Keycode::Backspace)
                        | SdlMenuAction::KeyPress(Keycode::KpBackspace) => {
                            filter.pop();

                            info!("filter: {}", filter);
                            return Some(Ok((None, menu.state().clone())));
                        }
                        SdlMenuAction::TextInput(text) => {
                            for c in text.iter() {
                                if *c == 0 as char {
                                    break;
                                }
                                filter.push(*c);
                            }
                            return Some(Ok((None, menu.state().clone())));
                        }
                        _ => {}
                    }
                }
            }

            if let Some(c) = state.shortcut {
                if let Err(e) = shortcut_handler(app, c, context) {
                    return Some(Err(e));
                }
            }

            None
        })?;

        if let Some(r) = result {
            return Ok((r, new_state));
        }
        menu_state = new_state;
    }
}

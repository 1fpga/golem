pub mod cores;
pub mod filesystem;
pub mod main;
pub mod style;

pub use cores::cores_menu_panel;
pub use main::main_menu;

use crate::application::menu::style::MenuReturn;
use crate::macguiver::application::Application;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_menu::items::NavigationItem;
use embedded_menu::selection_indicator::style::Invert;
use embedded_menu::selection_indicator::AnimatedPosition;
use embedded_menu::{Menu, MenuState};

pub type GolemMenuState<R> = MenuState<style::SdlMenuInputAdapter<R>, AnimatedPosition, Invert>;

pub fn text_menu<R: MenuReturn + Copy>(
    app: &mut impl Application<Color = BinaryColor>,
    title: &str,
    items: &[(&str, &str, R)],
    state: Option<GolemMenuState<R>>,
) -> (R, GolemMenuState<R>) {
    let mut items: Vec<_> = items
        .iter()
        .map(|(label, marker, result)| NavigationItem::new(*label, *result).with_marker(*marker))
        .collect();

    let mut menu = Menu::with_style(title, style::menu_style())
        .add_items(&mut items)
        .build_with_state(state.unwrap_or_default());

    app.event_loop(|app, state| {
        let buffer = app.main_buffer();
        buffer.clear(BinaryColor::Off).unwrap();
        menu.update(buffer);
        menu.draw(buffer).unwrap();

        for ev in state.events() {
            if let Some(action) = menu.interact(ev) {
                return Some((action, menu.state()));
            }
        }

        None
    })
}

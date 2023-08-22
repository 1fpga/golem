use crate::application::menu::style;
use crate::application::TopLevelViewType;
use crate::macguiver::application::Application;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::Drawable;
use embedded_menu::interaction::InteractionType;
use embedded_menu::items::NavigationItem;
use embedded_menu::Menu;
use mister_db::diesel::{QueryDsl, RunQueryDsl};
use mister_db::schema::cores::dsl::cores;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use std::ops::DerefMut;

pub fn main_menu(app: &mut impl Application<Color = BinaryColor>) -> TopLevelViewType {
    let ncores: i64 = cores
        .count()
        .get_result(app.database().write().unwrap().deref_mut())
        .unwrap();
    let core_title = format!("Cores ({ncores})");

    let mut menu = Menu::with_style("Main Menu", style::menu_style())
        .add_item(NavigationItem::new(&core_title, TopLevelViewType::MainMenu).with_marker(">"))
        .add_item(NavigationItem::new("ROMs", TopLevelViewType::MainMenu).with_marker(">"))
        .add_item(NavigationItem::new(
            "Settings...",
            TopLevelViewType::Settings,
        ))
        .add_item(NavigationItem::new(
            "Input Tester",
            TopLevelViewType::InputTester,
        ))
        .add_item(NavigationItem::new("About", TopLevelViewType::About))
        .build();

    app.event_loop(|app, state| {
        let buffer = app.main_buffer();
        buffer.clear(BinaryColor::Off).unwrap();
        menu.update(buffer);
        menu.draw(buffer).unwrap();

        for ev in state.events() {
            if let Event::KeyDown {
                keycode: Some(code),
                ..
            } = ev
            {
                match code {
                    Keycode::Escape => {
                        return Some(TopLevelViewType::InputTester);
                    }
                    Keycode::Return => {
                        return menu.interact(InteractionType::Select);
                    }
                    Keycode::Up => {
                        menu.interact(InteractionType::Previous);
                    }
                    Keycode::Down => {
                        menu.interact(InteractionType::Next);
                    }
                    _ => {}
                }
            }
        }

        None
    })
}

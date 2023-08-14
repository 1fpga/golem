use crate::application::menu::style;
use crate::application::{Panel, TopLevelView, TopLevelViewType};
use crate::macguiver::application::Application;
use embedded_graphics::Drawable;
use embedded_menu::interaction::InteractionType;
use embedded_menu::items::NavigationItem;
use embedded_menu::Menu;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;

#[derive(Debug, Clone, Copy)]
pub enum MenuAction {
    Back,
    Settings,
    StartRom,
}

pub fn main_menu(app: &mut impl Application) -> TopLevelViewType {
    let buffer = app.main_buffer();

    let mut menu = Menu::with_style("Main Menu", style::menu_style())
        .add_item(
            NavigationItem::new("Settings...", MenuAction::Settings)
                .with_marker(">")
                .with_detail_text("Lorem ipsum dolor sit amet, in per ."),
        )
        .add_item(
            NavigationItem::new("ROMs", MenuAction::StartRom)
                .with_marker(">")
                .with_detail_text("Lorem ipsum dolor sit amet, in per .asd.asd. as.d "),
        )
        .add_item(NavigationItem::new("Keyboard Tester", MenuAction::StartRom))
        .build();

    app.event_loop(|state| {
        menu.update(buffer);
        menu.draw(buffer).unwrap();

        for ev in app.events() {
            if let Event::KeyDown {
                keycode: Some(code),
                ..
            } = ev
            {
                // match code {
                //     Keycode::Escape => {
                //         return Some(TopLevelViewType::KeyboardTester);
                //     }
                //     Keycode::Return => {
                //         return Some(TopLevelViewType::MainMenu);
                //     }
                //     Keycode::Up => {
                //         menu.interact(InteractionType::Up);
                //     }
                //     Keycode::Down => {
                //         menu.interact(InteractionType::Down);
                //     }
                //     _ => {}
                // }
            }
        }

        None
    })
}

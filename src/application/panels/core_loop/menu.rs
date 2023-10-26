use crate::application::menu::style;
use crate::application::menu::style::MenuReturn;
use crate::macguiver::application::Application;
use crate::platform::{Core, CoreManager, MiSTerPlatform};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::Drawable;
use embedded_menu::items::NavigationItem;
use embedded_menu::Menu;
use std::convert::TryFrom;

mod items;

#[derive(Default, Debug, Clone, Copy)]
pub enum CoreMenuAction {
    #[default]
    Nothing,
    ToggleOption(u8, u8, usize),
    Back,
    Quit,
}

impl MenuReturn for CoreMenuAction {
    fn back() -> Option<Self> {
        Some(Self::Back)
    }
}

pub fn core_menu(app: &mut impl Application<Color = BinaryColor>, core: &mut impl Core) -> bool {
    app.platform_mut().core_manager_mut().show_menu();

    let status = core.status_bits();
    let mut items = core
        .menu_options()
        .iter()
        .filter_map(|o| items::ConfigMenuSelect::try_from((*o).clone()).ok())
        .map(|mut o| {
            if let Some(bits) = o.bits() {
                let value = status.get_range(bits.clone()) as usize;
                o.select(value);
            }
            o
        })
        .collect::<Vec<_>>();

    let mut menu = Menu::with_style("Core Menu", style::menu_style_simple())
        .add_items(&mut items)
        .add_item(NavigationItem::new("Back", CoreMenuAction::Back))
        .add_item(NavigationItem::new("Quit Core", CoreMenuAction::Quit))
        .build();

    let result = app.event_loop(|app, state| {
        let buffer = app.main_buffer();
        buffer.clear(BinaryColor::Off).unwrap();
        menu.update(buffer);
        menu.draw(buffer).unwrap();

        for ev in state.events() {
            if let Some(action) = menu.interact(ev) {
                match action {
                    CoreMenuAction::Back => {
                        return Some(false);
                    }
                    CoreMenuAction::Quit => {
                        return Some(true);
                    }
                    CoreMenuAction::ToggleOption(from, to, value) => {
                        let mut bits = core.status_bits();
                        bits.set_range(from..to, value as u32);
                        core.set_status_bits(bits);
                    }
                    CoreMenuAction::Nothing => {}
                }
            }
        }

        None
    });
    app.platform_mut().core_manager_mut().hide_menu();
    result
}

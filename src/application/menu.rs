use crate::application::{Panel, TopLevelViewType};
use crate::data::settings::Settings;
use crate::macguiver::buffer::DrawBuffer;
use crate::platform::PlatformState;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::Drawable;
use embedded_menu::interaction::InteractionType;
use embedded_menu::items::NavigationItem;
use embedded_menu::Menu;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use std::cell::RefCell;
use std::rc::Rc;

pub mod style;

#[derive(Debug, Clone, Copy)]
pub enum MenuAction {
    Back,
    Settings,
    StartRom,
}

type BoxedUpdateFn = Box<dyn FnMut(&PlatformState) -> Result<Option<MenuAction>, String>>;
type BoxedDrawFn = Box<dyn Fn(&mut DrawBuffer<BinaryColor>)>;

pub struct MainMenu {
    // Function to update the menu.
    update: BoxedUpdateFn,

    // Function to draw the menu.
    draw: BoxedDrawFn,
}

impl Panel for MainMenu {
    fn new(_settings: &Settings) -> Self {
        let menu = Menu::with_style("Main Menu", style::menu_style())
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

        let menu = Rc::new(RefCell::new(menu));
        let (update, draw) = {
            let menu_update = menu.clone();
            let menu_draw = menu.clone();

            let update = move |state: &PlatformState| {
                let mut menu = menu_update.borrow_mut();
                menu.update(state);

                for ev in state.events() {
                    if let Event::KeyDown {
                        keycode: Some(code),
                        ..
                    } = ev
                    {
                        match code {
                            Keycode::Escape => {
                                return Ok(Some(MenuAction::Back));
                            }
                            Keycode::Return => {
                                return Ok(menu.interact(InteractionType::Select));
                            }
                            Keycode::Up => {
                                return Ok(menu.interact(InteractionType::Previous));
                            }
                            Keycode::Down => {
                                return Ok(menu.interact(InteractionType::Next));
                            }
                            Keycode::Right => {
                                for _ in 0..9 {
                                    menu.interact(InteractionType::Next);
                                }
                                return Ok(menu.interact(InteractionType::Next));
                            }
                            _ => {}
                        }
                    }
                }

                return Ok(None);
            };

            let draw = move |target: &mut DrawBuffer<BinaryColor>| {
                let menu = menu_draw.borrow();
                menu.draw(target).unwrap();
            };

            (update, draw)
        };

        Self {
            update: Box::new(update),
            draw: Box::new(draw),
        }
    }

    fn update(&mut self, state: &PlatformState) -> Result<Option<TopLevelViewType>, String> {
        let action = (self.update)(state)?;
        if let Some(action) = action {
            match action {
                MenuAction::Back => Ok(Some(TopLevelViewType::MainMenu)),
                MenuAction::Settings => Ok(Some(TopLevelViewType::Settings)),
                MenuAction::StartRom => Ok(Some(TopLevelViewType::KeyboardTester)),
            }
        } else {
            Ok(None)
        }
    }

    fn draw(&self, target: &mut DrawBuffer<BinaryColor>) {
        (self.draw)(target)
    }
}

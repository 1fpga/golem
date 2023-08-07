use crate::application::{Panel, TopLevelView};
use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::events::keyboard::Keycode;
use crate::platform::PlatformState;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_menu::interaction::InteractionType;
use embedded_menu::items::NavigationItem;
use embedded_menu::selection_indicator::style::triangle::Triangle;
use embedded_menu::{Menu, MenuStyle};
use std::cell::RefCell;
use std::rc::Rc;

type BoxedUpdateFn = Box<dyn FnMut(&PlatformState) -> Result<Option<TopLevelView>, String>>;
type BoxedDrawFn = Box<dyn Fn(&mut DrawBuffer<BinaryColor>)>;

pub struct MainMenu {
    // Function to update the menu.
    update: BoxedUpdateFn,

    // Function to draw the menu.
    draw: BoxedDrawFn,
}

impl Panel for MainMenu {
    type NextView = TopLevelView;

    fn new() -> Self {
        let menu = Menu::with_style(
            "Menu",
            MenuStyle::new(BinaryColor::On)
                .with_animated_selection_indicator(5)
                .with_selection_indicator(Triangle)
                .with_font(&FONT_6X10),
        )
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

                if state.pressed().contains(Keycode::Escape) {
                    return Ok(Some(MenuAction::Back));
                }
                if state.pressed().contains(Keycode::Return) {
                    return Ok(menu.interact(InteractionType::Select));
                }
                if state.pressed().contains(Keycode::Up) {
                    return Ok(menu.interact(InteractionType::Previous));
                }
                if state.pressed().contains(Keycode::Down) {
                    return Ok(menu.interact(InteractionType::Next));
                }

                Ok(None)
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

    fn update(&mut self, state: &PlatformState) -> Result<Option<Self::NextView>, String> {
        let action = (self.update)(state)?;
        if let Some(action) = action {
            match action {
                MenuAction::Back => Ok(Some(TopLevelView::menu())),
                MenuAction::Settings => Ok(Some(TopLevelView::icon())),
                MenuAction::StartRom => Ok(Some(TopLevelView::keyboard_tester())),
            }
        } else {
            Ok(None)
        }
    }

    fn draw(&self, target: &mut DrawBuffer<BinaryColor>) {
        (self.draw)(target)
    }
}

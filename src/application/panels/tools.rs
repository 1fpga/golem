use crate::application::menu::style::MenuReturn;
use crate::application::menu::text_menu;
use crate::application::menu::TextMenuOptions;
use crate::application::panels::input_tester::input_tester;
use crate::macguiver::application::Application;
use embedded_graphics::pixelcolor::BinaryColor;

mod menu_tester;

#[derive(Copy, Clone, Debug, PartialEq)]
enum Menu {
    InputTester,
    MenuTester,
    Back,
}

impl MenuReturn for Menu {
    fn back() -> Option<Self> {
        Some(Menu::Back)
    }
}

pub fn tools_menu(app: &mut impl Application<Color = BinaryColor>) {
    let mut state = None;

    loop {
        let (result, new_state) = text_menu(
            app,
            "Tools",
            &[
                ("Input Tester", "", Menu::InputTester),
                ("Menu Tester", "", Menu::MenuTester),
            ],
            TextMenuOptions::default().with_state(state),
        );
        state = Some(new_state);

        match result {
            Menu::InputTester => input_tester(app),
            Menu::MenuTester => menu_tester::menu_tester(app),
            Menu::Back => break,
        }
    }
}

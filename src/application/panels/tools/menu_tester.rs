use crate::application::menu::style::MenuReturn;
use crate::application::menu::{text_menu, TextMenuOptions};
use crate::application::panels::alert::alert;
use crate::application::GoLEmApp;

#[derive(Copy, Clone, Debug, PartialEq)]
enum Menu {
    Select(usize),
    Details(usize),
    Sort,
    Idle,
    Back,
}

impl MenuReturn for Menu {
    fn back() -> Option<Self> {
        Some(Menu::Back)
    }

    fn into_details(self) -> Option<Self> {
        match self {
            Menu::Select(i) => Some(Menu::Details(i)),
            _ => None,
        }
    }

    fn sort() -> Option<Self> {
        Some(Menu::Sort)
    }
}

pub fn menu_tester(app: &mut GoLEmApp) {
    let mut state = None;

    let prefixes = ["A", "B", "C", "D"]
        .iter()
        .copied()
        .map(|s| (s, "1234", Menu::Idle))
        .collect::<Vec<_>>();

    let suffixes = ["z", "y", "x", "w"]
        .iter()
        .copied()
        .map(|s| (s, "5678", Menu::Idle))
        .collect::<Vec<_>>();

    let mut curr = 0;
    let items_owned = std::iter::repeat_with(|| {
        curr += 1;
        format!("Option {curr}. Char: {c}", c = (curr as u8) as char)
    })
    .take(128)
    .collect::<Vec<_>>();
    let items = items_owned
        .iter()
        .enumerate()
        .map(|(i, s)| (s.as_str(), "1234", Menu::Select(i + 1)))
        .collect::<Vec<_>>();

    loop {
        let (result, new_state) = text_menu(
            app,
            "<Menu Tester>",
            &items,
            TextMenuOptions::default()
                .with_state(state)
                .with_sort("Name")
                .with_details("Details")
                .with_prefix(&prefixes)
                .with_suffix(&suffixes),
        );
        state = Some(new_state);

        match result {
            Menu::Back => break,
            Menu::Select(i) => {
                let _ = alert(
                    app,
                    "Menu Tester",
                    format!("You selected item {i}").as_str(),
                    &["Okay", "Not Okay"],
                );
            }
            Menu::Details(i) => {
                let _ = alert(
                    app,
                    "Details",
                    format!("Here are the details of {i}").as_str(),
                    &["Okay"],
                );
            }
            _ => {}
        }
    }
}

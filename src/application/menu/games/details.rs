use crate::application::menu::style::MenuReturn;
use crate::application::menu::{text_menu, TextMenuOptions};
use crate::macguiver::application::Application;
use embedded_graphics::pixelcolor::BinaryColor;

#[derive(Default, Debug, Clone, Copy)]
enum MenuAction {
    #[default]
    Back,
    Favorite,
    Delete,
}

impl MenuReturn for MenuAction {
    fn back() -> Option<Self> {
        Some(Self::Back)
    }
}

pub fn games_details(
    app: &mut impl Application<Color = BinaryColor>,
    game: &mut golem_db::models::Game,
) -> Result<(), anyhow::Error> {
    let mut state = None;

    loop {
        let (action, new_state) = text_menu(
            app,
            &format!("{}", game.name),
            &[
                (
                    "Favorite",
                    if game.favorite { "[X]" } else { "[ ]" },
                    MenuAction::Favorite,
                ),
                ("Delete", "", MenuAction::Delete),
            ],
            TextMenuOptions::default().with_state(state),
        );
        state = Some(new_state);

        match action {
            MenuAction::Back => break,
            MenuAction::Favorite => {
                game.favorite(&mut app.database().lock().unwrap())?;
            }
            MenuAction::Delete => {
                game.delete(&mut app.database().lock().unwrap())?;
            }
        }
    }

    Ok(())
}

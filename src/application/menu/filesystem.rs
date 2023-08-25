use crate::application::menu::style::{menu_style, MenuReturn};
use crate::macguiver::application::Application;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::mono_font::ascii;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::Drawable;
use embedded_menu::items::NavigationItem;
use embedded_menu::Menu;
use regex::Regex;
use std::path::{Path, PathBuf};

const MAXIMUM_TITLE_PATH_LENGTH: usize = 38;

pub struct FilesystemMenuOptions {
    /// Allow the user to go to the parent of the initial directory.
    allow_back: bool,

    /// Show directories first.
    dir_first: bool,

    /// Show hidden files and directories.
    show_hidden: bool,

    /// Show extensions on files.
    show_extensions: bool,

    /// Sort files by name.
    sort: bool,

    /// File pattern to show.
    pattern: Option<Regex>,
}

impl Default for FilesystemMenuOptions {
    fn default() -> Self {
        Self {
            allow_back: false,
            dir_first: true,
            show_hidden: false,
            show_extensions: true,
            sort: true,
            pattern: None,
        }
    }
}

impl FilesystemMenuOptions {
    pub fn with_allow_back(self, allow_back: bool) -> Self {
        Self { allow_back, ..self }
    }
    pub fn pattern(self, pattern: Regex) -> Self {
        Self {
            pattern: Some(pattern),
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum MenuAction {
    Back,
    UpDir,
    Select(usize),
}

impl MenuReturn for MenuAction {
    fn back() -> Self {
        MenuAction::Back
    }
}

#[derive(Default)]
enum EventLoopResult {
    #[default]
    Back,
    Continue(PathBuf),
    Select(PathBuf),
}

pub fn select_file_path_menu(
    app: &mut impl Application<Color = BinaryColor>,
    title: impl AsRef<str>,
    initial: impl AsRef<Path>,
    options: FilesystemMenuOptions,
) -> Result<Option<PathBuf>, std::io::Error> {
    let mut path = initial.as_ref().to_path_buf();
    loop {
        let mut entries = std::fs::read_dir(&path)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                let name = path.file_name()?.to_string_lossy().to_string();

                if !options.show_hidden && name.starts_with('.') {
                    return None;
                }

                if path.is_dir() {
                    if let Some(pattern) = options.pattern.as_ref() {
                        if !pattern.is_match(&name) {
                            return None;
                        }
                    }
                }

                let mut name = if options.show_extensions {
                    name
                } else {
                    path.file_stem()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default()
                };

                if path.is_dir() {
                    name.push('/');
                }

                Some((path, name))
            })
            .collect::<Vec<_>>();

        if options.sort {
            if options.dir_first {
                entries.sort_by(|(a, _), (b, _)| match (a.is_dir(), b.is_dir()) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.cmp(&b),
                });
            } else {
                entries.sort_by(|(a, _), (b, _)| a.cmp(&b));
            }
        }

        let mut entries_items = entries
            .iter()
            .enumerate()
            .map(|(idx, (path, name))| {
                if path.is_dir() {
                    NavigationItem::new(name, MenuAction::Select(idx)).with_marker(">")
                } else {
                    NavigationItem::new(name, MenuAction::Select(idx))
                }
            })
            .collect::<Vec<_>>();

        let path_string = path.to_string_lossy().to_string();
        let split_idx = path_string.len().saturating_sub(MAXIMUM_TITLE_PATH_LENGTH);
        // Format the title. If path is too long, replace the beginning with "...".
        let title = format!(
            "{}\n{}",
            title.as_ref(),
            if split_idx == 0 {
                path_string
            } else {
                format!("...{}", &path_string[split_idx..])
            }
        );

        let mut maybe_dotdot = if path.parent().is_some() {
            if options.allow_back || path != initial.as_ref() {
                Some(NavigationItem::new("..", MenuAction::UpDir).with_marker(">"))
            } else {
                None
            }
        } else {
            None
        }
        .into_iter()
        .collect::<Vec<_>>();

        let mut menu = Menu::with_style(&title, menu_style().with_title_font(&ascii::FONT_6X9))
            .add_items(maybe_dotdot.as_mut_slice())
            .add_items(entries_items.as_mut_slice())
            .add_item(NavigationItem::new("< Cancel", MenuAction::Back).with_marker("<<"))
            .build();

        let selection: EventLoopResult = app.event_loop(|app, state| {
            let buffer = app.main_buffer();
            buffer.clear(BinaryColor::Off).unwrap();
            menu.update(buffer);
            menu.draw(buffer).unwrap();

            for ev in state.events() {
                match menu.interact(ev) {
                    None => {}
                    Some(MenuAction::Back) => return Some(EventLoopResult::Back),
                    Some(MenuAction::Select(idx)) => {
                        let path = entries[idx].0.to_path_buf();
                        return if path.is_dir() {
                            Some(EventLoopResult::Continue(path))
                        } else {
                            Some(EventLoopResult::Select(path))
                        };
                    }
                    Some(MenuAction::UpDir) => {
                        let path = path.parent().unwrap().to_path_buf();
                        return Some(EventLoopResult::Continue(path));
                    }
                }
            }
            None
        });

        match selection {
            EventLoopResult::Back => return Ok(None),
            EventLoopResult::Continue(p) => path = p,
            EventLoopResult::Select(s) => return Ok(Some(s)),
        }
    }
}

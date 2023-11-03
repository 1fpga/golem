#![allow(dead_code)]
use crate::application::menu::style::MenuReturn;
use crate::application::menu::{text_menu, TextMenuOptions};
use crate::macguiver::application::Application;
use embedded_graphics::mono_font::ascii;
use embedded_graphics::pixelcolor::BinaryColor;
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

    /// Select directory only (not files).
    directory: bool,

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
            directory: false,
            pattern: None,
        }
    }
}

impl FilesystemMenuOptions {
    pub fn with_allow_back(self, allow_back: bool) -> Self {
        Self { allow_back, ..self }
    }

    pub fn with_pattern(self, pattern: Regex) -> Self {
        Self {
            pattern: Some(pattern),
            ..self
        }
    }

    pub fn with_select_dir(self) -> Self {
        Self {
            directory: true,
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum MenuAction {
    Back,
    UpDir,
    Select(usize),
    SelectCurrentDirectory,
    ChangeSort,
}

impl MenuReturn for MenuAction {
    fn back() -> Option<Self> {
        Some(MenuAction::Back)
    }

    fn sort() -> Option<Self> {
        Some(Self::ChangeSort)
    }
}

enum SortOption {
    NameAsc,
    NameDesc,
    SizeAsc,
    SizeDesc,
}

impl SortOption {
    pub fn next(self) -> Self {
        match self {
            Self::NameAsc => Self::NameDesc,
            Self::NameDesc => Self::SizeAsc,
            Self::SizeAsc => Self::SizeDesc,
            Self::SizeDesc => Self::NameAsc,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NameAsc => "Name A-Z",
            Self::NameDesc => "Name Z-A",
            Self::SizeAsc => "Size >",
            Self::SizeDesc => "Size <",
        }
    }
}

pub fn select_file_path_menu(
    app: &mut impl Application<Color = BinaryColor>,
    title: impl AsRef<str>,
    initial: impl AsRef<Path>,
    options: FilesystemMenuOptions,
) -> Result<Option<PathBuf>, std::io::Error> {
    let FilesystemMenuOptions {
        allow_back,
        dir_first,
        show_hidden,
        show_extensions,
        directory,
        pattern,
    } = options;

    let mut path = initial.as_ref().to_path_buf();

    let mut sort = SortOption::NameAsc;

    loop {
        let mut entries = std::fs::read_dir(&path)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                let name = path.file_name()?.to_string_lossy().to_string();

                if !show_hidden && name.starts_with('.') {
                    return None;
                }

                if let Some(pattern) = pattern.as_ref() {
                    if !pattern.is_match(&name) {
                        return None;
                    }
                }

                if path.is_dir() {
                    if let Some(pattern) = pattern.as_ref() {
                        if !pattern.is_match(&name) {
                            return None;
                        }
                    }
                } else if directory {
                    return None;
                }

                let mut name = if show_extensions {
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

        entries.sort_by(|(a, _), (b, _)| {
            if dir_first {
                match (a.is_dir(), b.is_dir()) {
                    (true, false) => return std::cmp::Ordering::Less,
                    (false, true) => return std::cmp::Ordering::Greater,
                    _ => {}
                }
            }

            match sort {
                SortOption::NameAsc => a.file_name().cmp(&b.file_name()),
                SortOption::NameDesc => b.file_name().cmp(&a.file_name()),
                SortOption::SizeAsc => a
                    .metadata()
                    .unwrap()
                    .len()
                    .cmp(&b.metadata().unwrap().len()),
                SortOption::SizeDesc => b
                    .metadata()
                    .unwrap()
                    .len()
                    .cmp(&a.metadata().unwrap().len()),
            }
        });

        let entries_items = entries
            .iter()
            .enumerate()
            .map(|(idx, (path, name))| {
                if path.is_dir() {
                    (name, "DIR".to_string(), MenuAction::Select(idx))
                } else {
                    (
                        name,
                        humansize::format_size(
                            std::fs::metadata(path).unwrap().len(),
                            humansize::DECIMAL,
                        ),
                        MenuAction::Select(idx),
                    )
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

        let mut menu_options = TextMenuOptions::default()
            .with_sort(sort.as_str())
            .with_title_font(&ascii::FONT_6X9)
            .with_back("Cancel");

        if path.parent().is_some() && (allow_back || path != initial.as_ref()) {
            menu_options = menu_options.with_prefix(&[("..", "", MenuAction::UpDir)]);
        }
        if directory {
            menu_options = menu_options.with_suffix(&[(
                "Select Current Directory",
                "",
                MenuAction::SelectCurrentDirectory,
            )]);
        }

        let (selection, _new_state) =
            text_menu(app, &title, entries_items.as_slice(), menu_options);

        match selection {
            MenuAction::Select(idx) => {
                let p = entries[idx].0.to_path_buf();
                if p.is_dir() {
                    path = p.to_path_buf();
                } else {
                    return Ok(Some(p));
                }
            }
            MenuAction::SelectCurrentDirectory => {
                return Ok(Some(path.to_path_buf()));
            }
            MenuAction::Back => return Ok(None),
            MenuAction::UpDir => {
                path = path.parent().unwrap().to_path_buf();
            }
            MenuAction::ChangeSort => {
                sort = sort.next();
            }
        }
    }
}

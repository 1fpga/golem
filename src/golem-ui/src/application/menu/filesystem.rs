#![allow(dead_code)]

use std::path::{Path, PathBuf};

use embedded_graphics::mono_font::ascii;
use regex::Regex;

use crate::application::menu::style::MenuReturn;
use crate::application::menu::{text_menu, TextMenuOptions};
use crate::application::GoLEmApp;

const MAXIMUM_TITLE_PATH_LENGTH: usize = 38;

#[derive(Debug, Default, Clone)]
pub struct FilesystemMenuOptions {
    /// Allow the user to go to the parent of the initial directory.
    pub allow_back: Option<bool>,

    /// Show directories first.
    pub dir_first: Option<bool>,

    /// Show hidden files and directories.
    pub show_hidden: Option<bool>,

    /// Show extensions on files.
    pub show_extensions: Option<bool>,

    /// Select directory only (not files).
    pub directory: Option<bool>,

    /// File pattern to show.
    pub pattern: Option<Regex>,

    /// Extensions to show.
    pub extensions: Option<Vec<String>>,
}

impl FilesystemMenuOptions {
    pub fn with_allow_back(self, allow_back: bool) -> Self {
        Self {
            allow_back: Some(allow_back),
            ..self
        }
    }

    pub fn with_pattern(self, pattern: Regex) -> Self {
        Self {
            pattern: Some(pattern),
            ..self
        }
    }

    pub fn with_select_dir(self) -> Self {
        Self {
            directory: Some(true),
            ..self
        }
    }

    pub fn with_extensions(self, extensions: Vec<String>) -> Self {
        Self {
            extensions: Some(extensions),
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
    app: &mut GoLEmApp,
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
        extensions,
    } = options;

    let show_hidden = show_hidden.unwrap_or(false);
    let allow_back = allow_back.unwrap_or(true);
    let dir_first = dir_first.unwrap_or(true);
    let directory = directory.unwrap_or(false);
    let show_extensions = show_extensions.unwrap_or(true);

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

                if let Some(ext) = extensions.as_ref() {
                    if path.is_file() {
                        let extension = path.extension().unwrap_or_default().to_string_lossy();
                        if !ext
                            .iter()
                            .any(|e| e.as_str().eq_ignore_ascii_case(&extension))
                        {
                            return None;
                        }
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

        let items = entries
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
        let entries_items = items
            .iter()
            .map(|(a, b, c)| (a.as_str(), b.as_str(), *c))
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

        let up = [("..", "", MenuAction::UpDir)];
        if path.parent().is_some() && (allow_back || path != initial.as_ref()) {
            menu_options = menu_options.with_prefix(&up);
        }

        let select_curr_dir = [(
            "Select Current Directory",
            "",
            MenuAction::SelectCurrentDirectory,
        )];
        if directory {
            menu_options = menu_options.with_suffix(&select_curr_dir);
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

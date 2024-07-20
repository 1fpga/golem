use image::GenericImageView;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::time::Instant;

use sdl3::keyboard::Scancode;
use tracing::{debug, error, info, warn};

use mister_fpga::core::MisterFpgaCore;
use one_fpga::Core;

use crate::application::panels::core_loop::menu::core_menu;
use crate::application::GoLEmApp;
use crate::data::paths;
use crate::input::shortcut::Shortcut;

/// A command ID that can be associated with a shortcut.
/// Passing strings around is hard across all languages and subsystems.
/// Commands should be [Copy] to make them easier to work with. Since
/// [String] is not [Copy], we use a [u32] hash of the command label.
///
/// This hash is fast, deterministic, and has a low risk of collision.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct CommandId(u32);

impl CommandId {
    pub fn new(str: &str) -> Self {
        CommandId(CommandId::id_from_str(str))
    }

    pub fn id_from_str(str: &str) -> u32 {
        let mut s: u32 = 0;
        for c in str.as_bytes() {
            s = s.wrapping_mul(223).wrapping_add(*c as u32);
        }
        s
    }
}

impl FromStr for CommandId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(CommandId(CommandId::id_from_str(s)))
    }
}

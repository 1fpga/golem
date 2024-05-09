use std::fmt;

/// A scancode maps a virtual key on the keyboard to a physical key on the keyboard.
/// We reuse the SDL3 type for simplicity, as it is well maintained and complete.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Scancode(sdl3::keyboard::Scancode);

impl Scancode {
    /// Create a new scancode from an SDL3 scancode.
    pub fn new(scancode: sdl3::keyboard::Scancode) -> Self {
        Self(scancode)
    }

    /// Get the SDL3 scancode.
    pub fn sdl_scancode(&self) -> sdl3::keyboard::Scancode {
        self.0
    }

    /// Get an u8 representation of the key (the key scan code).
    #[inline]
    pub fn as_repr(&self) -> i32 {
        self.0 as i32
    }
}

impl std::str::FromStr for Scancode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        sdl3::keyboard::Scancode::from_name(s)
            .map(Scancode)
            .ok_or_else(|| format!("Could not parse scancode: {}", s))
    }
}

impl fmt::Display for Scancode {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0.name())
    }
}

impl From<sdl3::keyboard::Scancode> for Scancode {
    fn from(scancode: sdl3::keyboard::Scancode) -> Self {
        Self(scancode)
    }
}

impl From<Scancode> for sdl3::keyboard::Scancode {
    fn from(scancode: Scancode) -> Self {
        scancode.0
    }
}

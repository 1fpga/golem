use crate::config::video::aspect::AspectRatio;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// A Resolution.
#[derive(Default, Debug, Clone, Copy, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Resolution {
    pub width: u16,
    pub height: u16,
}

impl Resolution {
    pub fn new(width: u16, height: u16) -> Self {
        Resolution { width, height }
    }

    pub fn aspect_ratio(&self) -> AspectRatio {
        AspectRatio::new(self.width, self.height)
    }
}

impl Display for Resolution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}x{}", self.width, self.height,))?;
        Ok(())
    }
}

impl FromStr for Resolution {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((w, h)) = s.split_once('x') {
            let w = u16::from_str(w).map_err(|_| "Invalid width")?;
            let h = u16::from_str(h).map_err(|_| "Invalid height")?;
            Ok(Resolution {
                width: w,
                height: h,
            })
        } else {
            Err("Invalid resolution: expected WIDTHxHEIGHT[@VREFRESH]")
        }
    }
}

#[test]
fn can_deserialize_and_serialize() {
    let res = Resolution::from_str("1920x1080").unwrap();
    assert_eq!(res.to_string(), "1920x1080");
    assert_eq!(res, Resolution::new(1920, 1080));
}

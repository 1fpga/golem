use crate::config::video::resolution::Resolution;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

fn gcd_(mut u: u16, mut v: u16) -> u16 {
    if u == v {
        return u;
    }
    if u == 0 {
        return v;
    }
    if v == 0 {
        return u;
    }

    let shift = (u | v).trailing_zeros();
    u >>= shift;
    v >>= shift;
    u >>= u.trailing_zeros();

    loop {
        v >>= v.trailing_zeros();

        if u > v {
            std::mem::swap(&mut u, &mut v);
        }

        v -= u; // here v >= u

        if v == 0 {
            break;
        }
    }

    u << shift
}

/// An aspect ratio is similar to a resolution, but will always find the smallest ratio between
/// the width and the height. For example, 1920x1080 and 3840x2160 will both have an aspect ratio
/// of 16:9.
#[derive(
    SerializeDisplay,
    DeserializeFromStr,
    Default,
    Debug,
    Clone,
    Copy,
    Hash,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
)]
pub struct AspectRatio {
    pub vertical: u16,
    pub horizontal: u16,
}

impl AspectRatio {
    pub fn new(horizontal: u16, vertical: u16) -> Self {
        let gcd = gcd_(vertical, horizontal).max(1);
        AspectRatio {
            vertical: vertical / gcd,
            horizontal: horizontal / gcd,
        }
    }

    pub fn square() -> Self {
        Self::new(1, 1)
    }

    pub fn zero() -> Self {
        Self::new(0, 0)
    }
}

impl Display for AspectRatio {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}:{}", self.horizontal, self.vertical))
    }
}

impl FromStr for AspectRatio {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((v, h)) = s.split_once(':') {
            let horizontal = h.parse::<u16>().unwrap();
            let vertical = v.parse::<u16>().unwrap();
            Ok(AspectRatio::new(horizontal, vertical))
        } else {
            Err("Invalid aspect ratio: expected 'horizontal:vertical'")
        }
    }
}

impl From<(u16, u16)> for AspectRatio {
    fn from((horizontal, vertical): (u16, u16)) -> Self {
        Self::new(horizontal, vertical)
    }
}

impl From<AspectRatio> for (u16, u16) {
    fn from(ratio: AspectRatio) -> Self {
        (ratio.horizontal, ratio.vertical)
    }
}

impl From<Resolution> for AspectRatio {
    fn from(res: Resolution) -> Self {
        Self::new(res.width, res.height)
    }
}

#[test]
fn works() {
    let ratio = AspectRatio::new(3840, 2160);
    assert_eq!(ratio.to_string(), "16:9");
}

use crate::config::video::resolution::Resolution;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

fn gcd_(mut u: u32, mut v: u32) -> u32 {
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

        #[allow(clippy::manual_swap)]
        if u > v {
            // mem::swap(&mut u, &mut v);
            let temp = u;
            u = v;
            v = temp;
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
    pub vertical: u32,
    pub horizontal: u32,
}

impl AspectRatio {
    pub fn new(horizontal: u32, vertical: u32) -> Self {
        let gcd = gcd_(vertical, horizontal);
        AspectRatio {
            vertical: vertical / gcd,
            horizontal: horizontal / gcd,
        }
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
            let horizontal = h.parse::<u32>().unwrap();
            let vertical = v.parse::<u32>().unwrap();
            Ok(AspectRatio::new(horizontal, vertical))
        } else {
            Err("Invalid aspect ratio: expected 'horizontal:vertical'")
        }
    }
}

impl From<(u32, u32)> for AspectRatio {
    fn from((horizontal, vertical): (u32, u32)) -> Self {
        Self::new(horizontal, vertical)
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

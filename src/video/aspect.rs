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
    pub fn new(vertical: u16, horizontal: u16) -> Self {
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
            let vertical = v.parse::<u16>().unwrap();
            let horizontal = h.parse::<u16>().unwrap();
            Ok(AspectRatio::new(vertical, horizontal))
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

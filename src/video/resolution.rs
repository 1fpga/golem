use crate::video::aspect::AspectRatio;
use std::fmt::{Display, Formatter};
use std::num::NonZeroU32;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Resolution {
    pub width: u16,
    pub height: u16,
    /// A fixed point non-zero value. This is the framerate in milliseconds, once deserialized.
    pub framerate: Option<NonZeroU32>,
}

impl Resolution {
    pub fn new(width: u16, height: u16, framerate: u32) -> Self {
        Resolution {
            width,
            height,
            framerate: NonZeroU32::new(framerate),
        }
    }

    pub fn new_no_framerate(width: u16, height: u16) -> Self {
        Resolution {
            width,
            height,
            framerate: None,
        }
    }

    pub fn new_with_framerate(width: u16, height: u16, framerate: u32) -> Self {
        Resolution {
            width,
            height,
            framerate: NonZeroU32::new(framerate),
        }
    }

    pub fn aspect_ratio(&self) -> AspectRatio {
        AspectRatio::new(self.width, self.height)
    }

    pub fn framerate_ms(&self) -> Option<u32> {
        self.framerate.map(|f| f.get())
    }

    pub fn framerate_secs(&self) -> Option<f32> {
        self.framerate.map(|f| (f.get() as f32) / 1000.0)
    }
}

impl Display for Resolution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}x{}", self.width, self.height,))?;
        if let Some(framerate) = self.framerate {
            f.write_fmt(format_args!("@{}", (framerate.get() as f32) / 1000.0))?;
        }
        Ok(())
    }
}

impl FromStr for Resolution {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((w, rest)) = s.split_once('x') {
            let w = u16::from_str(w).map_err(|_| "Invalid width")?;
            let (h, framerate) = match rest.split_once('@') {
                Some((h, framerate)) => (
                    h.parse::<u16>().map_err(|_| "Invalid height")?,
                    NonZeroU32::new(
                        (framerate.parse::<f32>().map_err(|_| "Invalid framerate")? * 1000.0)
                            as u32,
                    ),
                ),
                None => (rest.parse::<u16>().map_err(|_| "Invalid height")?, None),
            };
            Ok(Resolution {
                width: w,
                height: h,
                framerate,
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
    assert_eq!(res, Resolution::new(1920, 1080, 0));

    // Test with no framerate.
    let res = Resolution::from_str("1920x1080@0").unwrap();
    assert_eq!(res.to_string(), "1920x1080");
    assert_eq!(res, Resolution::new(1920, 1080, 0));

    // Test with simple resolution.
    let res = Resolution::from_str("1920x1080@1").unwrap();
    assert_eq!(res.to_string(), "1920x1080@1");
    assert_eq!(res, Resolution::new(1920, 1080, 1000));

    // Test with resolution.
    let res = Resolution::from_str("1920x1080@1.234").unwrap();
    assert_eq!(res.to_string(), "1920x1080@1.234");
    assert_eq!(res, Resolution::new(1920, 1080, 1234));

    // Test with resolution that has a really detailed framerate.
    let res = Resolution::from_str("1920x1080@1.23456789").unwrap();
    assert_eq!(res.to_string(), "1920x1080@1.234");
    assert_eq!(res, Resolution::new(1920, 1080, 1234));
}

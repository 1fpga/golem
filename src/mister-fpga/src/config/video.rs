use serde::de::{Error, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

pub mod aspect;
pub mod edid;
pub mod resolution;

#[derive(Clone, Copy, PartialEq)]
pub struct VideoGainOffsets {
    pub gain_red: f32,
    pub offset_red: f32,
    pub gain_green: f32,
    pub offset_green: f32,
    pub gain_blue: f32,
    pub offset_blue: f32,
}

impl Serialize for VideoGainOffsets {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            // Serialize into a string separated by spaces.
            serializer.serialize_str(&self.to_string())
        } else {
            let mut seq = serializer.serialize_seq(Some(6))?;
            seq.serialize_element(&self.gain_red)?;
            seq.serialize_element(&self.offset_red)?;
            seq.serialize_element(&self.gain_green)?;
            seq.serialize_element(&self.offset_green)?;
            seq.serialize_element(&self.gain_blue)?;
            seq.serialize_element(&self.offset_blue)?;
            seq.end()
        }
    }
}

struct VideoGainOffsetsVisitor;

impl<'de> Visitor<'de> for VideoGainOffsetsVisitor {
    type Value = VideoGainOffsets;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string or a sequence of 6 elements")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Self::Value::from_str(v).map_err(E::custom)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_str(&v)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let gain_red = seq
            .next_element::<f32>()?
            .ok_or_else(|| A::Error::custom("Not enough values"))?;
        let offset_red = seq
            .next_element::<f32>()?
            .ok_or_else(|| A::Error::custom("Not enough values"))?;
        let gain_green = seq
            .next_element::<f32>()?
            .ok_or_else(|| A::Error::custom("Not enough values"))?;
        let offset_green = seq
            .next_element::<f32>()?
            .ok_or_else(|| A::Error::custom("Not enough values"))?;
        let gain_blue = seq
            .next_element::<f32>()?
            .ok_or_else(|| A::Error::custom("Not enough values"))?;
        let offset_blue = seq
            .next_element::<f32>()?
            .ok_or_else(|| A::Error::custom("Not enough values"))?;
        Ok(VideoGainOffsets {
            gain_red,
            offset_red,
            gain_green,
            offset_green,
            gain_blue,
            offset_blue,
        })
    }
}

impl<'de> Deserialize<'de> for VideoGainOffsets {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(VideoGainOffsetsVisitor)
    }
}

impl FromStr for VideoGainOffsets {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values: Vec<f32> = s
            .splitn(6, ',')
            .map(|s| s.trim().parse::<f32>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "Could not parse values as floats.")?;
        if values.len() != 6 {
            return Err("Expected 6 elements");
        }

        Ok(VideoGainOffsets {
            gain_red: values[0],
            offset_red: values[1],
            gain_green: values[2],
            offset_green: values[3],
            gain_blue: values[4],
            offset_blue: values[5],
        })
    }
}

impl Default for VideoGainOffsets {
    fn default() -> Self {
        VideoGainOffsets {
            gain_red: 1.0,
            offset_red: 0.0,
            gain_green: 1.0,
            offset_green: 0.0,
            gain_blue: 1.0,
            offset_blue: 0.0,
        }
    }
}

impl fmt::Display for VideoGainOffsets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {}",
            self.gain_red,
            self.offset_red,
            self.gain_green,
            self.offset_green,
            self.gain_blue,
            self.offset_blue
        )
    }
}

impl fmt::Debug for VideoGainOffsets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VideoGainOffsets")
            .field(&self.gain_red)
            .field(&self.offset_red)
            .field(&self.gain_green)
            .field(&self.offset_green)
            .field(&self.gain_blue)
            .field(&self.offset_blue)
            .finish()
    }
}

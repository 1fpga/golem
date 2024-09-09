use bitvec::prelude::*;
use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};
use std::fmt::{Debug, Display, Formatter, Write};

pub mod units;

/// A 128-bit status bit map, used by MiSTer cores to communicate options and triggers.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StatusBitMap(BitArray<[u16; 8], Lsb0>);

impl Serialize for StatusBitMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // If the format is human-readable, use bits as a string instead.
        if serializer.is_human_readable() {
            return serializer.serialize_str(&self.to_string());
        }

        // Either serialize the first 4 words or the whole array.
        let r = self.as_raw_slice();
        let short = r.iter().skip(4).all(|x| *x == 0);

        let mut seq = serializer.serialize_seq(Some(if short { 4 } else { 8 }))?;
        seq.serialize_element(&r[0])?;
        seq.serialize_element(&r[1])?;
        seq.serialize_element(&r[2])?;
        seq.serialize_element(&r[3])?;

        if !short {
            seq.serialize_element(&r[4])?;
            seq.serialize_element(&r[5])?;
            seq.serialize_element(&r[6])?;
            seq.serialize_element(&r[7])?;
        }
        seq.end()
    }
}

impl Debug for StatusBitMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut str = String::with_capacity(128 + 4);
        let arr = self.0.as_raw_slice();

        for byte in 0..arr.len() {
            str += &format!("{:032b} ", arr[byte]);
            if arr[byte..].iter().all(|x| *x == 0) {
                break;
            }
        }

        f.debug_tuple("StatusBitMap")
            .field(&str.trim_end())
            .finish()
    }
}

impl Default for StatusBitMap {
    fn default() -> Self {
        Self(BitArray::new([0; 8]))
    }
}

impl StatusBitMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn as_raw_slice(&self) -> &[u16] {
        self.0.as_raw_slice()
    }
    pub fn as_mut_raw_slice(&mut self) -> &mut [u16] {
        self.0.as_raw_mut_slice()
    }

    pub fn has_extra(&self) -> bool {
        self.0.as_raw_slice()[4..].iter().any(|x| *x != 0)
    }

    pub fn set(&mut self, idx: usize, value: bool) {
        self.0.set(idx, value);
    }

    pub fn get(&self, idx: usize) -> bool {
        self.0[idx]
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get_range(&self, range: impl IntoIterator<Item = u8>) -> u32 {
        let mut result = 0;
        let mut iter = range.into_iter().peekable();
        let start = *iter.peek().unwrap_or(&0);
        for i in iter {
            result |= (self.get(i as usize) as u32) << (i - start) as usize;
        }
        result
    }

    /// Set a range of bits to a value. This cannot do more than 32 bits at a time.
    /// On error, this may panic.
    pub fn set_range(&mut self, range: impl IntoIterator<Item = u8>, mut value: u32) {
        for i in range.into_iter() {
            self.set(i as usize, value & 1 != 0);
            value >>= 1;
        }
    }

    pub fn debug_header() -> String {
        "              Upper                          Lower\n\
        0         1         2         3          4         5         6\n\
        01234567890123456789012345678901 23456789012345678901234567890123\n\
        0123456789ABCDEFGHIJKLMNOPQRSTUV 0123456789ABCDEFGHIJKLMNOPQRSTUV\n\
        "
        .to_string()
    }

    pub fn debug_string(&self, header: bool) -> String {
        let mut result = String::new();
        if header {
            result += &Self::debug_header();
        }

        fn output_u64(mut word64: u64) -> String {
            let mut word_str = String::new();
            while word64 != 0 {
                word_str.push(if word64 & 1 != 0 { 'X' } else { ' ' });
                word64 >>= 1;
            }
            if word_str.len() > 32 {
                word_str.insert(32, ' ');
            }
            word_str
        }

        let raw = self.0.as_raw_slice();
        result.push_str(&output_u64(
            (raw[0] as u64)
                | ((raw[1] as u64) << 16)
                | ((raw[2] as u64) << 32)
                | ((raw[3] as u64) << 48),
        ));

        if raw[4] != 0 || raw[5] != 0 || raw[6] != 0 || raw[7] != 0 {
            if header {
                result += "\n\
                    0     0         0         0          1         1         1       \n\
                    6     7         8         9          0         1         2       \n\
                    45678901234567890123456789012345 67890123456789012345678901234567\n\
                    ";
            }
            result.push_str(&output_u64(
                ((raw[0] as u64) << 48)
                    | ((raw[1] as u64) << 32)
                    | ((raw[2] as u64) << 16)
                    | (raw[3] as u64),
            ));
        }

        result.push('\n');
        result
    }

    pub fn iter(&self) -> impl Iterator<Item = bool> + '_ {
        self.0.iter().by_vals()
    }
}

impl Display for StatusBitMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..(if self.has_extra() { 128 } else { 64 }) {
            f.write_char(if self.get(i) { '1' } else { '0' })?;
        }
        Ok(())
    }
}

#[test]
fn status_bits() {
    let mut status_bits = StatusBitMap::new();
    status_bits.set(0, true);
    status_bits.set(2, true);
    status_bits.set_range(4..8, 0b0101);

    status_bits.set_range(32..34, 3);
    status_bits.set_range(64..67, 3);
    status_bits.set_range(96..120, 8);

    assert_eq!(
        status_bits.to_string(),
        concat!(
            "10101010000000000000000000000000",
            "11000000000000000000000000000000",
            "11000000000000000000000000000000",
            "00010000000000000000000000000000"
        )
    );

    assert_eq!(status_bits.get_range(32..34), 3);
    assert_eq!(status_bits.get_range(64..67), 3);
}

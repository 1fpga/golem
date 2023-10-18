use super::midi;
use super::uart;
use crate::types::units::UnitConversion;
use std::ops::Range;
use std::str::FromStr;

#[derive(Default, Debug, Clone)]
pub struct Settings {
    /// UART mode
    pub uart_mode: Vec<uart::UartSpeed>,

    /// MIDI mode
    pub midi_mode: Vec<midi::MidiSpeed>,

    /// The save state memory range of the core.
    pub save_state: Option<Range<usize>>,
}

impl Settings {
    fn parse_save_state(s: &str) -> Result<Range<usize>, &'static str> {
        if let Some((base, size)) = s.split_once(':') {
            let base = base.parse::<usize>().map_err(|_| "Invalid base")?;
            let size = size.parse::<usize>().map_err(|_| "Invalid size")?;
            let end = base.checked_add(size).ok_or("Save state range overflow")?;

            if base < 0x2000_0000 || !(0x2000_0000..0x4000_0000).contains(&end) {
                return Err("Save state range out of bounds");
            }
            if size > 128.mebibytes() {
                return Err("Save state size too large");
            }
            if size == 0 {
                return Err("Save state size cannot be zero");
            }

            Ok(base..end)
        } else {
            Err("Could not parse save state range")
        }
    }
}

impl FromStr for Settings {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut save_state = None;
        let mut uart_mode = Vec::new();
        let mut midi_mode = Vec::new();

        for setting in s.split(',') {
            let setting = setting.trim();
            if setting.is_empty() {
                continue;
            }

            if let Some(s) = s.strip_prefix("SS") {
                save_state = Some(Self::parse_save_state(s)?);
            } else if let Some(s) = s.strip_prefix("UART") {
                // Parse strings of format "12345(label):56789(label 2)".
                for speed in s.split(':') {
                    uart_mode.push(speed.parse::<uart::UartSpeed>()?);
                }
            } else if let Some(s) = s.strip_prefix("MIDI") {
                // Parse strings of format "12345(label):56789(label 2)".
                for speed in s.split(':') {
                    midi_mode.push(speed.parse::<midi::MidiSpeed>()?);
                }
            } else {
                return Err("Unknown config setting");
            }
        }

        Ok(Self {
            save_state,
            uart_mode,
            midi_mode,
        })
    }
}
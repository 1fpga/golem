use super::LABELED_SPEED_RE;
use std::str::FromStr;

const DEFAULT_MIDI_SPEED: u32 = 31250;

#[derive(Debug, Clone)]
pub struct MidiSpeed {
    pub speed: u32,
    pub label: String,
}

impl FromStr for MidiSpeed {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let captures = LABELED_SPEED_RE.captures(s).ok_or("Invalid MIDI mode")?;

        let speed = match captures.get(1) {
            None => DEFAULT_MIDI_SPEED,
            Some(s) => s.as_str().parse::<u32>().map_err(|_| "Invalid MIDI mode")?,
        };

        let label = captures
            .get(2)
            .map(|s| s.as_str().to_string())
            .unwrap_or_else(|| format!("{speed}"));
        Ok(Self { speed, label })
    }
}

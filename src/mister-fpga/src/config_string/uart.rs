use super::LABELED_SPEED_RE;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct UartSpeed {
    pub speed: u32,
    pub label: String,
}

impl FromStr for UartSpeed {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let captures = LABELED_SPEED_RE.captures(s).ok_or("Invalid UART mode")?;
        let speed = captures
            .get(1)
            .ok_or("Invalid UART mode")
            .and_then(|s| s.as_str().parse::<u32>().map_err(|_| "Invalid UART mode"))?;
        let label = captures
            .get(2)
            .map(|s| s.as_str().to_string())
            .unwrap_or_else(|| format!("{speed}"));
        Ok(Self { speed, label })
    }
}

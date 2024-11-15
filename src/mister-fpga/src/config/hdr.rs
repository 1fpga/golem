use merg::Merge;
use serde::Deserialize;

/// 1 - enable HDR using HLG (recommended for most users)
/// 2 - enable HDR using the DCI P3 color space (use color controls to tweak, suggestion: set saturation to 80).
#[derive(Default, Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HdrConfig {
    #[default]
    #[serde(alias = "0")]
    None = 0,

    #[serde(alias = "1")]
    Hlg = 1,

    #[serde(alias = "dcip3", alias = "2")]
    DciP3 = 2,
}

impl Merge for HdrConfig {
    fn merge(&mut self, other: Self) {
        if other != HdrConfig::default() {
            *self = other;
        }
    }
}

impl HdrConfig {
    pub fn is_enabled(&self) -> bool {
        *self != HdrConfig::None
    }
}

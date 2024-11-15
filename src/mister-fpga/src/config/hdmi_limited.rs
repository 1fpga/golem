use merg::Merge;
use serde::Deserialize;

#[derive(Default, Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HdmiLimitedConfig {
    #[default]
    #[serde(alias = "0")]
    FullColorRange = 0,
    #[serde(alias = "1")]
    Limited = 1,
    #[serde(alias = "2")]
    LimitedForVgaConverters = 2,
}

impl Merge for HdmiLimitedConfig {
    fn merge(&mut self, other: Self) {
        if other != HdmiLimitedConfig::default() {
            *self = other;
        }
    }
}

impl HdmiLimitedConfig {
    pub fn is_limited(&self) -> bool {
        *self != HdmiLimitedConfig::FullColorRange
    }
}

use merge::Merge;
use serde::{Deserialize, Serialize};

#[derive(strum::Display, Default, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VgaModeConfig {
    #[default]
    #[serde(alias = "0")]
    Rgb = 0,
    #[serde(alias = "1")]
    Ypbpr = 1,
    #[serde(alias = "2")]
    Svideo = 2,
    #[serde(alias = "3")]
    Cvbs = 3,
}

impl Merge for VgaModeConfig {
    fn merge(&mut self, other: Self) {
        if other != VgaModeConfig::default() {
            *self = other;
        }
    }
}

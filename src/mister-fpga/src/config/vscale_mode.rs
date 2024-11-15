use merg::Merge;
use serde::Deserialize;

#[derive(Default, Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub enum VideoScaleModeConfig {
    #[default]
    #[serde(alias = "0")]
    Fit = 0,
    #[serde(alias = "1")]
    IntegerFit = 1,
    #[serde(alias = "2")]
    HalfStepFit = 2,
    #[serde(alias = "3")]
    QuarterStepFit = 3,
    #[serde(alias = "4")]
    IntegerFitCoreAspectRatio = 4,
    #[serde(alias = "5")]
    IntegerFitDisplayAspectRatio = 5,
}

impl Merge for VideoScaleModeConfig {
    fn merge(&mut self, other: Self) {
        if other != VideoScaleModeConfig::default() {
            *self = other;
        }
    }
}

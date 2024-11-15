use merg::Merge;
use serde::Deserialize;

/// Display OSD menu rotated,  0 - no rotation, 1 - rotate right (+90°), 2 - rotate left (-90°)
#[derive(Default, Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OsdRotateConfig {
    #[default]
    #[serde(alias = "0")]
    NoRotation = 0,
    #[serde(alias = "1")]
    RotateRight = 1,
    #[serde(alias = "2")]
    RotateLeft = 2,
}

impl Merge for OsdRotateConfig {
    fn merge(&mut self, other: Self) {
        if other != OsdRotateConfig::default() {
            *self = other;
        }
    }
}

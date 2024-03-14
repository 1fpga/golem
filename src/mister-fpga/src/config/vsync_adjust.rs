use serde::Deserialize;

#[derive(Default, Debug, Copy, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VsyncAdjustConfig {
    #[default]
    #[serde(alias = "0")]
    Disabled = 0,

    #[serde(alias = "1")]
    Automatic = 1,

    #[serde(alias = "2")]
    LowLatency = 2,
}

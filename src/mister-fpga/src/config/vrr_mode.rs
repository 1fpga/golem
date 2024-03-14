use serde::Deserialize;

/// Variable Refresh Rate control
/// 0 - Do not enable VRR (send no VRR control frames)
/// 1 - Auto Detect VRR from display EDID.
/// 2 - Force Enable Freesync
/// 3 - Force Enable Vesa HDMI Forum VRR
#[derive(Default, Debug, Clone, Copy, PartialEq, Deserialize)]
pub enum VrrModeConfig {
    #[default]
    #[serde(alias = "0")]
    Disabled = 0,

    #[serde(alias = "1")]
    Auto = 1,

    #[serde(alias = "2")]
    Freesync = 2,

    #[serde(alias = "3")]
    HdmiVrr = 3,
}

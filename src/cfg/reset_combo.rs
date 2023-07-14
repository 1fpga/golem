use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResetComboConfig {
    #[default]
    #[serde(alias = "lctrl+lalt+ralt", alias = "0", alias = "3")]
    LCtrlLAltRAlt,
    #[serde(alias = "lctrl+lgui+rgui", alias = "1")]
    LCtrlLGuiRGui,
    #[serde(alias = "lctrl+lalt+del", alias = "2")]
    LCtrlLAltDel,
}

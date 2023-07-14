use serde::Deserialize;
use strum::EnumString;

/// A structure representing the ntsc_mode coniguration option.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Deserialize, EnumString)]
#[serde(rename_all = "snake_case")]
pub enum NtscModeConfig {
    #[default]
    #[serde(alias = "normal", alias = "0")]
    Ntsc = 0,
    #[serde(alias = "pal60", alias = "1")]
    Pal60 = 1,
    #[serde(alias = "palm", alias = "2")]
    PalM = 2,
}

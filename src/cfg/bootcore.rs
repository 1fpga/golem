use serde_with::DeserializeFromStr;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(DeserializeFromStr, Debug, Default, Clone, PartialEq, Eq)]
pub enum BootCoreConfig {
    #[default]
    None,
    LastCore,
    ExactLastCore,
    CoreName(String),
}

impl BootCoreConfig {
    pub fn is_none(&self) -> bool {
        self == &BootCoreConfig::None
    }
    pub fn is_last_core(&self) -> bool {
        matches!(
            self,
            BootCoreConfig::LastCore | BootCoreConfig::ExactLastCore
        )
    }
}

impl Display for BootCoreConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BootCoreConfig::None => write!(f, ""),
            BootCoreConfig::LastCore => write!(f, "lastcore"),
            BootCoreConfig::ExactLastCore => write!(f, "exactlastcore"),
            BootCoreConfig::CoreName(name) => write!(f, "{}", name),
        }
    }
}

impl FromStr for BootCoreConfig {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "" => Ok(BootCoreConfig::None),
            "lastcore" => Ok(BootCoreConfig::LastCore),
            "exactlastcore" => Ok(BootCoreConfig::ExactLastCore),
            _ => Ok(BootCoreConfig::CoreName(s.into())),
        }
    }
}

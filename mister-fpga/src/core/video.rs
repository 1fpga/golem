use crate::config;
use tracing::{error, warn};

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
use linux as private;

#[cfg(not(target_os = "linux"))]
mod private {
    use crate::config;
    use tracing::debug;

    pub fn hdmi_config_init(config: &config::MisterConfig) -> Result<(), String> {
        debug!(?config, "HDMI configuration not supported on this platform");
        Ok(())
    }

    pub fn init_mode(
        options: &config::MisterConfig,
        _core: &mut crate::core::MisterFpgaCore,
        _is_menu: bool,
    ) -> Result<(), String> {
        debug!(
            ?options,
            "Video mode configuration not supported on this platform"
        );
        Ok(())
    }
}

/// Initialize the video Hardware configuration.
// TODO: this should not take the whole config but a subset of it related only to video.
pub fn init(options: &config::MisterConfig) {
    if let Err(error) = private::hdmi_config_init(options) {
        error!("Failed to initialize HDMI configuration: {}", error);
        warn!("This is not a fatal error, the application will continue to run.");
    }
}

pub fn init_mode(
    options: &config::MisterConfig,
    core: &mut crate::core::MisterFpgaCore,
    is_menu: bool,
) {
    if !is_menu {
        return;
    }
    if let Err(error) = private::init_mode(options, core, is_menu) {
        error!("Failed to initialize video mode: {}", error);
        warn!("This is not a fatal error, the application will continue to run.");
    }
}

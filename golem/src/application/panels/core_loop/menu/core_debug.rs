use crate::application::panels::alert::alert;
use crate::application::GoLEmApp;
use crate::platform::Core;

pub fn debug_menu(app: &mut GoLEmApp, core: &mut impl Core) {
    let mask = core.status_mask().debug_string(true);
    let value = core.status_bits().debug_string(false);
    let message = format!(
        "\
    Status bits:\n\
    Mask:
    {}
    {}
    ",
        mask, value
    );

    let _ = alert(app, "Debug Info", &message, &["Back"]);
}

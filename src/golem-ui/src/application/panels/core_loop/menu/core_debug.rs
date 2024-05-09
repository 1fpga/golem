use mister_fpga::core::MisterFpgaCore;

use crate::application::panels::alert::alert;
use crate::application::GoLEmApp;

pub fn debug_menu(app: &mut GoLEmApp, core: &mut MisterFpgaCore) {
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

use crate::application::panels::alert::alert;
use crate::macguiver::application::Application;
use crate::platform::Core;
use embedded_graphics::pixelcolor::BinaryColor;

pub fn debug_info(app: &mut impl Application<Color = BinaryColor>, core: &mut impl Core) {
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

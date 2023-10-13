use crate::macguiver::application::Application;
use embedded_graphics::pixelcolor::BinaryColor;

pub fn progress_spinner(
    app: &mut impl Application<Color = BinaryColor>,
    message: &str,
    update_callback: impl FnMut() -> bool,
    cancel_callback: impl FnOnce(),
) {
    todo!()
}

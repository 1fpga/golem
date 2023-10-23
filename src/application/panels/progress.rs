use crate::macguiver::application::Application;
use embedded_graphics::pixelcolor::BinaryColor;

pub fn progress_spinner(
    _app: &mut impl Application<Color = BinaryColor>,
    _message: &str,
    _update_callback: impl FnMut() -> bool,
    _cancel_callback: impl FnOnce(),
) {
    todo!()
}

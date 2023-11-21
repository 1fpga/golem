use sdl3::keyboard::Scancode;

pub trait InputUpdate {
    fn key_down(&mut self, code: Scancode);
    fn key_up(&mut self, code: Scancode);
    fn game_button_down(&mut self, button: u8);
    fn game_button_up(&mut self, button: u8);
    fn joy_axis(&mut self, axis: u8, value: i32);
    fn mouse(&mut self, delta_x: i32, delta_y: i32);

    fn matches(&self) -> bool;
}

#[derive(Default, Debug)]
pub struct BasicInputShortcut {
    keyboard: [Option<Scancode>; 8],
    joystick: [Option<u8>; 8],
    axis: [(Option<u8>, Option<i32>); 4],
}

impl BasicInputShortcut {
    pub fn add_key(&mut self, code: Scancode) {
        for x in 0..8 {}
    }
}

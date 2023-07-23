use crate::macgyver::buffer::DrawBuffer;
use std::convert::Infallible;
use std::time::Duration;

pub trait Widget {
    // fn draw(self, display: &mut DrawBuffer<BinaryColor>) -> Result<(), Infallible>;
    fn update(&mut self, delta: Duration);
}

pub struct WidgetGroup {
    widgets: Vec<Box<dyn Widget>>,
}

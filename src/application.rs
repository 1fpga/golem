use crate::application::widgets::keyboard::KeyboardTesterWidget;
use crate::macguiver::application::Application;
use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::widgets::boxed::{BoxedWidget, HorizontalAlignment, VerticalAlignment};
use crate::macguiver::widgets::text::fps::FpsCounter;

use crate::macguiver::widgets::Widget;
use crate::platform::{PlatformState, WindowManager};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;

mod toolbar;
mod widgets;

#[derive(Debug)]
pub struct MiSTer {
    toolbar: toolbar::Toolbar,
    keyboard_tester: KeyboardTesterWidget,
}

impl MiSTer {
    pub fn run(&mut self) -> Result<(), String> {
        let mut window_manager = WindowManager::default();
        window_manager.run(self)
    }
}

impl Application for MiSTer {
    type Color = BinaryColor;

    fn new() -> Self
    where
        Self: Sized,
    {
        let mut toolbar = toolbar::Toolbar::default();
        toolbar.append(
            BoxedWidget::new(FpsCounter::<200>::new(MonoTextStyle::new(
                &embedded_graphics::mono_font::ascii::FONT_6X9,
                BinaryColor::On,
            )))
            .aligned(VerticalAlignment::Middle, HorizontalAlignment::Left),
        );
        toolbar.append(widgets::network::NetworkWidget::new());

        Self {
            toolbar,
            keyboard_tester: KeyboardTesterWidget::new(),
        }
    }

    fn update(&mut self, state: &PlatformState) {
        self.keyboard_tester.set_state(*state.keys());
        self.toolbar.update();
    }

    fn draw_title(&self, target: &mut DrawBuffer<BinaryColor>) {
        self.toolbar.draw(target);
    }

    fn draw(&self, target: &mut DrawBuffer<BinaryColor>) {
        self.keyboard_tester.draw(target);
    }
}

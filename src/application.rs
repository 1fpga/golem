use crate::application::widgets::keyboard::KeyboardTesterWidget;
use crate::macguiver::application::Application;
use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::widgets::boxed::{BoxedWidget, HorizontalAlignment, VerticalAlignment};
use crate::macguiver::widgets::text::fps::FpsCounter;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::Drawable;

use crate::macguiver::widgets::image::ImageWidget;
use crate::macguiver::widgets::Widget;
use crate::main_inner::Flags;
use crate::platform::{PlatformState, WindowManager};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::Text;

mod toolbar;
mod widgets;

#[derive(Debug)]
pub struct MiSTer {
    toolbar: toolbar::Toolbar,
    keyboard_tester: KeyboardTesterWidget,
}

impl MiSTer {
    pub fn run(&mut self, flags: Flags) -> Result<(), String> {
        let mut window_manager = WindowManager::default();
        window_manager.run(self, flags)
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

        let font_images = [
            include_bytes!("../assets/font/arrow_down.bin"),
            include_bytes!("../assets/font/arrow_left.bin"),
            include_bytes!("../assets/font/arrow_right.bin"),
            include_bytes!("../assets/font/arrow_right_mini.bin"),
            include_bytes!("../assets/font/arrow_up.bin"),
            include_bytes!("../assets/font/atari_left.bin"),
            include_bytes!("../assets/font/atari_right.bin"),
            include_bytes!("../assets/font/battery_charging.bin"),
            include_bytes!("../assets/font/battery_empty.bin"),
            include_bytes!("../assets/font/battery_full.bin"),
            include_bytes!("../assets/font/battery_half.bin"),
            include_bytes!("../assets/font/bluetooth.bin"),
            include_bytes!("../assets/font/box_empty.bin"),
            include_bytes!("../assets/font/box_fill1.bin"),
            include_bytes!("../assets/font/box_fill2.bin"),
            include_bytes!("../assets/font/box_fill3.bin"),
            include_bytes!("../assets/font/box_fill4.bin"),
            include_bytes!("../assets/font/box_top.bin"),
            include_bytes!("../assets/font/burger_menu_2.bin"),
            include_bytes!("../assets/font/burger_menu_3.bin"),
            include_bytes!("../assets/font/burger_menu_4.bin"),
            include_bytes!("../assets/font/checkbox_empty.bin"),
            include_bytes!("../assets/font/checkbox_full.bin"),
            include_bytes!("../assets/font/checkbox_mark.bin"),
            include_bytes!("../assets/font/dot_middle.bin"),
            include_bytes!("../assets/font/lock_locked.bin"),
            include_bytes!("../assets/font/lock_unlocked.bin"),
            include_bytes!("../assets/font/mem_32.bin"),
            include_bytes!("../assets/font/mem_64.bin"),
            include_bytes!("../assets/font/mem_128.bin"),
            include_bytes!("../assets/font/mem_none.bin"),
            include_bytes!("../assets/font/network_eth.bin"),
            include_bytes!("../assets/font/network_globe.bin"),
            include_bytes!("../assets/font/network_wifi.bin"),
            include_bytes!("../assets/font/null.bin"),
            include_bytes!("../assets/font/speaker_empty.bin"),
            include_bytes!("../assets/font/speaker_full.bin"),
        ];

        for (i, bin) in font_images.iter().enumerate() {
            let i = i as i32;
            let image = ImageWidget::from_bin(bin, 8).unwrap();

            image.draw(&mut target.sub_buffer(Rectangle::new(
                Point::new(12 + (i % 16) * 10, 12 + (i / 16) * 10),
                Size::new(8, 8),
            )));
        }

        Text::new(
            "0\n1\n2\n3\n4\n5\n6\n7\n8\n9\nA\nB\nC\nD\nE\nF",
            Point::new(2, 19),
            MonoTextStyle::new(
                &embedded_graphics::mono_font::ascii::FONT_6X10,
                BinaryColor::On,
            ),
        )
        .draw(target)
        .unwrap();

        ImageWidget::from_bin(include_bytes!("../assets/font/speaker_empty.bin"), 8)
            .unwrap()
            .draw(&mut target.sub_buffer(Rectangle::new(Point::new(200, 16), Size::new(8, 8))));
        ImageWidget::from_bin(include_bytes!("../assets/font/speaker_full.bin"), 8)
            .unwrap()
            .draw(&mut target.sub_buffer(Rectangle::new(Point::new(200, 30), Size::new(8, 8))));
    }
}

use crate::application::{Panel, TopLevelView, TopLevelViewType};
use crate::data::settings::Settings;
use crate::macguiver::buffer::DrawBuffer;
use crate::platform::PlatformState;
use embedded_graphics::geometry::Point;
use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use sdl3::event::Event;

pub struct IconView;

impl Panel for IconView {
    fn new(_settings: &Settings) -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn update(&mut self, state: &PlatformState) -> Result<Option<TopLevelViewType>, String> {
        if state.events().any(|event| {
            matches!(
                event,
                Event::KeyDown {
                    keycode: Some(sdl3::keyboard::Keycode::Tab),
                    ..
                }
            )
        }) {
            Ok(Some(TopLevelViewType::KeyboardTester))
        } else {
            Ok(None)
        }
    }

    fn draw(&self, target: &mut DrawBuffer<BinaryColor>) {
        let font_images = [
            include_bytes!("../../assets/icons/arrow_down.raw"),
            include_bytes!("../../assets/icons/arrow_left.raw"),
            include_bytes!("../../assets/icons/arrow_right.raw"),
            include_bytes!("../../assets/icons/arrow_right_mini.raw"),
            include_bytes!("../../assets/icons/arrow_up.raw"),
            include_bytes!("../../assets/icons/atari_left.raw"),
            include_bytes!("../../assets/icons/atari_right.raw"),
            include_bytes!("../../assets/icons/battery_charging.raw"),
            include_bytes!("../../assets/icons/battery_empty.raw"),
            include_bytes!("../../assets/icons/battery_full.raw"),
            include_bytes!("../../assets/icons/battery_half.raw"),
            include_bytes!("../../assets/icons/bluetooth.raw"),
            include_bytes!("../../assets/icons/box_empty.raw"),
            include_bytes!("../../assets/icons/box_fill1.raw"),
            include_bytes!("../../assets/icons/box_fill2.raw"),
            include_bytes!("../../assets/icons/box_fill3.raw"),
            include_bytes!("../../assets/icons/box_fill4.raw"),
            include_bytes!("../../assets/icons/box_top.raw"),
            include_bytes!("../../assets/icons/burger_menu_2.raw"),
            include_bytes!("../../assets/icons/burger_menu_3.raw"),
            include_bytes!("../../assets/icons/burger_menu_4.raw"),
            include_bytes!("../../assets/icons/checkbox_empty.raw"),
            include_bytes!("../../assets/icons/checkbox_full.raw"),
            include_bytes!("../../assets/icons/checkbox_mark.raw"),
            include_bytes!("../../assets/icons/dot_middle.raw"),
            include_bytes!("../../assets/icons/lock_locked.raw"),
            include_bytes!("../../assets/icons/lock_unlocked.raw"),
            include_bytes!("../../assets/icons/mem_32.raw"),
            include_bytes!("../../assets/icons/mem_64.raw"),
            include_bytes!("../../assets/icons/mem_128.raw"),
            include_bytes!("../../assets/icons/mem_none.raw"),
            include_bytes!("../../assets/icons/network_eth.raw"),
            include_bytes!("../../assets/icons/network_globe.raw"),
            include_bytes!("../../assets/icons/network_wifi.raw"),
            include_bytes!("../../assets/icons/null.raw"),
            include_bytes!("../../assets/icons/speaker_empty.raw"),
            include_bytes!("../../assets/icons/speaker_full.raw"),
        ];

        for (i, bin) in font_images.iter().enumerate() {
            let i = i as i32;
            Image::new(
                &ImageRaw::<BinaryColor>::new(*bin, 8),
                Point::new(12 + (i % 16) * 10, 12 + (i / 16) * 10),
            )
            .draw(target)
            .unwrap();
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
    }
}

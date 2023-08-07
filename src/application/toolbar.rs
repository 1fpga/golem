use crate::application::widgets::network::{NetworkWidget, NetworkWidgetView};
use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::views::clock::DateTimeWidget;
use crate::macguiver::views::fps::FpsCounter;
use crate::macguiver::views::Widget;
use embedded_graphics::geometry::{Dimensions, Point};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::Drawable;
use embedded_layout::align::{horizontal, vertical, Align};
use embedded_layout::layout::linear::{spacing, LinearLayout};
use embedded_layout::object_chain::Chain;
use embedded_layout::View;

/// The toolbar for Mister, showing up in the title OSD bar.
///
/// This includes on the left a series of icons that can be updated individually, and
/// on the right a clock.
pub struct Toolbar {
    clock: DateTimeWidget,
    fps: FpsCounter,
    network: NetworkWidget,
    show_fps: bool,
}

impl Default for Toolbar {
    fn default() -> Self {
        Self {
            clock: DateTimeWidget::default(),
            fps: FpsCounter::new(MonoTextStyle::new(
                &embedded_graphics::mono_font::ascii::FONT_6X9,
                BinaryColor::On,
            )),
            network: NetworkWidget::new(),
            show_fps: true,
        }
    }
}

impl View for Toolbar {
    fn translate_impl(&mut self, _by: Point) {
        todo!()
    }

    fn bounds(&self) -> Rectangle {
        todo!()
    }
}

impl Widget for Toolbar {
    type Color = BinaryColor;

    fn update(&mut self) -> bool {
        [
            self.clock.update(),
            self.fps.update(),
            self.network.update(),
        ]
        .iter()
        .any(|x| *x)
    }

    fn draw(&self, target: &mut DrawBuffer<Self::Color>) {
        let mut bound = target.bounding_box();
        // Move things off border.
        bound.top_left += Point::new(2, 0);
        bound.size.width -= 4;

        LinearLayout::horizontal(
            Chain::new(self.fps.clone()).append(NetworkWidgetView::from_network(&self.network)),
        )
        .with_spacing(spacing::FixedMargin(2))
        .arrange()
        .align_to(&bound, horizontal::Left, vertical::Center)
        .draw(target)
        .unwrap();

        self.clock
            .clone()
            .align_to(&bound, horizontal::Right, vertical::Center)
            .draw(target)
            .unwrap();
    }
}

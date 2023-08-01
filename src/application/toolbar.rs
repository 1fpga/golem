use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::widgets::boxed::{BoxedWidget, HorizontalAlignment, VerticalAlignment};
use crate::macguiver::widgets::clock::DateTimeWidget;
use crate::macguiver::widgets::group::horizontal::HorizontalWidgetGroup;
use crate::macguiver::widgets::Widget;
use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::BinaryColor;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

/// The toolbar for Mister, showing up in the title OSD bar.
///
/// This includes on the left a series of icons that can be updated individually, and
/// on the right a clock.
#[derive(Debug)]
pub struct Toolbar {
    icons: Rc<RefCell<HorizontalWidgetGroup<BinaryColor>>>,
    left_group: BoxedWidget<BinaryColor>,
    clock: BoxedWidget<BinaryColor>,
}

impl Default for Toolbar {
    fn default() -> Self {
        let icons = Rc::new(RefCell::new(
            HorizontalWidgetGroup::default().with_spacing(1),
        ));
        Self {
            clock: BoxedWidget::new(DateTimeWidget::default())
                .aligned(VerticalAlignment::Middle, HorizontalAlignment::Right)
                .with_margin_tuple((1, 3, 1, 3)),
            left_group: BoxedWidget::new(Rc::clone(&icons))
                .aligned(VerticalAlignment::Middle, HorizontalAlignment::Left)
                .with_margin_tuple((0, 3, 0, 3)),
            icons,
        }
    }
}

impl Toolbar {
    pub fn append(&mut self, widget: impl Widget<Color = BinaryColor> + 'static) {
        self.icons.borrow_mut().append(widget);
    }
}

impl Widget for Toolbar {
    type Color = BinaryColor;

    fn size_hint(&self, parent_size: Size) -> Size {
        parent_size
    }

    fn update(&mut self) {
        self.left_group.update();
        self.clock.update();
    }

    fn draw(&self, target: &mut DrawBuffer<Self::Color>) {
        self.left_group.draw(target);
        self.clock.draw(target);
    }
}

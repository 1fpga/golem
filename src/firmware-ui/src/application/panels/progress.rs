use crate::application::OneFpgaApp;
use embedded_graphics::draw_target::{DrawTarget, DrawTargetExt};
use embedded_graphics::geometry::{Dimensions, Point, Size};
use embedded_graphics::mono_font::{ascii, MonoTextStyle};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Transform;
use embedded_graphics::primitives::{
    CornerRadii, Line, Primitive, PrimitiveStyle, Rectangle, RoundedRectangle, Styled,
};
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use embedded_layout::align::horizontal;
use embedded_layout::layout::linear::{spacing, LinearLayout};
use embedded_layout::object_chain::Chain;

struct ProgressBar {
    width: u32,
    total: u32,
    current: u32,
    rectangle: Styled<RoundedRectangle, PrimitiveStyle<BinaryColor>>,
    bar: Styled<RoundedRectangle, PrimitiveStyle<BinaryColor>>,
}

impl Dimensions for ProgressBar {
    fn bounding_box(&self) -> Rectangle {
        self.rectangle.bounding_box()
    }
}

impl Transform for ProgressBar {
    fn translate(&self, by: Point) -> Self {
        Self {
            rectangle: self.rectangle.translate(by),
            bar: self.bar.translate(by),
            ..*self
        }
    }

    fn translate_mut(&mut self, by: Point) -> &mut Self {
        self.rectangle.translate_mut(by);
        self.bar.translate_mut(by);
        self
    }
}

impl Drawable for ProgressBar {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, display: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.rectangle.draw(display)?;
        self.bar.draw(display)?;
        Ok(())
    }
}

impl ProgressBar {
    pub fn new(width: u32, total: u32, current: u32) -> Self {
        let mut rectangle_style = PrimitiveStyle::with_fill(BinaryColor::Off);
        rectangle_style.stroke_width = 2;
        rectangle_style.stroke_color = Some(BinaryColor::On);
        let rectangle = RoundedRectangle::new(
            Rectangle::new(Point::zero(), Size::new(width, 32)),
            CornerRadii::new(Size::new(8, 8)),
        )
        .into_styled(rectangle_style);

        let bar = RoundedRectangle::new(
            Rectangle::new(Point::zero(), Size::new(0, 32)),
            CornerRadii::new(Size::new(8, 8)),
        )
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On));

        let mut this = Self {
            width,
            total,
            current,
            rectangle,
            bar,
        };
        this.set_progress(current, None);
        this
    }

    pub fn set_progress(&mut self, current: u32, total: Option<u32>) {
        self.current = current;
        if let Some(total) = total {
            self.total = total;
        }
        self.bar.primitive.rectangle.size.width = if self.total == 0 {
            0
        } else {
            current * self.width / self.total
        };
    }
}

#[allow(unused)]
pub enum ProgressBarUpdate {
    UpdateBar(u32),
    UpdateBarTotal(u32, u32),
    UpdateMessage(String),
    Done,
    Cancel,
    Idle,
}

pub fn progress_bar(
    app: &mut OneFpgaApp,
    message: &str,
    total: u32,
    mut update_callback: impl FnMut() -> ProgressBarUpdate,
) -> bool {
    let display_area = app.main_buffer().bounding_box();

    let bar = ProgressBar::new(display_area.size.width * 3 / 4, total, 0);
    let message = message.to_string();

    let mut layout = LinearLayout::vertical(
        Chain::new(
            LinearLayout::vertical(
                Chain::new(Text::new(
                    &message,
                    Point::zero(),
                    MonoTextStyle::new(&ascii::FONT_8X13_BOLD, BinaryColor::On),
                ))
                .append(
                    Line::new(
                        Point::zero(),
                        Point::new(display_area.bounding_box().size.width as i32, 0),
                    )
                    .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1)),
                ),
            )
            .arrange(),
        )
        .append(bar),
    )
    .with_alignment(horizontal::Center)
    .with_spacing(spacing::DistributeFill(display_area.size.height - 32))
    .arrange();

    let mut last_update = std::time::Instant::now();

    app.draw_loop(|app, _state| {
        let mut buffer = app.main_buffer().color_converted();
        let _ = buffer.clear(BinaryColor::Off);
        let _ = layout.draw(&mut buffer);

        let now = std::time::Instant::now();
        let elapsed = now - last_update;
        if elapsed.as_millis() > 100 {
            last_update = now;

            match update_callback() {
                ProgressBarUpdate::UpdateBar(current) => {
                    let bar = &mut layout.inner_mut().object;
                    bar.set_progress(current, None);
                }
                ProgressBarUpdate::UpdateBarTotal(current, total) => {
                    let bar = &mut layout.inner_mut().object;
                    bar.set_progress(current, Some(total));
                }
                ProgressBarUpdate::UpdateMessage(_new_message) => {
                    // message = new_message;
                    // layout = layout.arrange();
                }
                ProgressBarUpdate::Done => {
                    return Some(true);
                }
                ProgressBarUpdate::Cancel => {
                    return Some(false);
                }
                ProgressBarUpdate::Idle => {}
            }
        }

        None
    })
}

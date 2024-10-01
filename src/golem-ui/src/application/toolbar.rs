use crate::application::widgets::network::{NetworkWidget, NetworkWidgetView};
use crate::data::settings::Settings;
use crate::macguiver::views::clock::DateTimeWidget;
use crate::macguiver::views::fps::{FpsCounter, FpsCounterView};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, Point};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::transform::Transform;
use embedded_graphics::Drawable;
use embedded_layout::align::{horizontal, vertical, Align};
use embedded_layout::layout::linear::{spacing, LinearLayout};
use embedded_layout::prelude::Views;
use std::sync::Arc;

enum ToolbarItem {
    Fps(FpsCounterView),
    Network(NetworkWidgetView),
}

impl Transform for ToolbarItem {
    fn translate(&self, by: Point) -> Self {
        match self {
            ToolbarItem::Fps(fps) => ToolbarItem::Fps(fps.translate(by)),
            ToolbarItem::Network(network) => ToolbarItem::Network(network.translate(by)),
        }
    }

    fn translate_mut(&mut self, by: Point) -> &mut Self {
        match self {
            ToolbarItem::Fps(fps) => {
                fps.translate_mut(by);
                self
            }
            ToolbarItem::Network(network) => {
                network.translate_mut(by);
                self
            }
        }
    }
}

impl Dimensions for ToolbarItem {
    fn bounding_box(&self) -> Rectangle {
        match self {
            ToolbarItem::Fps(fps) => fps.bounding_box(),
            ToolbarItem::Network(network) => network.bounding_box(),
        }
    }
}

impl Drawable for ToolbarItem {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        match self {
            ToolbarItem::Fps(fps) => fps.draw(target),
            ToolbarItem::Network(network) => network.draw(target),
        }
    }
}

/// The toolbar for Mister, showing up in the title OSD bar.
///
/// This includes on the left a series of icons that can be updated individually, and
/// on the right a clock.
pub struct Toolbar {
    fps: FpsCounter,
    network: NetworkWidget,
    clock: DateTimeWidget,

    settings: Arc<Settings>,
    on_settings_update: bus::BusReader<()>,
}

impl Toolbar {
    pub fn new(settings: Arc<Settings>) -> Self {
        let on_settings_update = settings.on_update();
        let clock = DateTimeWidget::new(settings.toolbar_datetime_format().time_format());

        Self {
            clock,
            fps: FpsCounter::new(MonoTextStyle::new(
                &embedded_graphics::mono_font::ascii::FONT_6X9,
                BinaryColor::On,
            )),
            network: NetworkWidget::new(),
            settings,
            on_settings_update,
        }
    }

    pub fn update(&mut self) -> bool {
        let mut should_redraw = self.clock.update()
            || self.network.update()
            || self
                .clock
                .set_time_format(self.settings.toolbar_datetime_format().time_format());

        // Check for settings changes.
        if self.on_settings_update.try_recv().is_ok() {
            should_redraw = true;
        }

        if self.settings.show_fps() {
            should_redraw = should_redraw || self.fps.update();
        }

        should_redraw
    }
}

impl Transform for Toolbar {
    fn translate(&self, _by: Point) -> Self {
        unimplemented!()
    }

    fn translate_mut(&mut self, _by: Point) -> &mut Self {
        // Do nothing, toolbar always takes up the whole area.
        self
    }
}

impl Drawable for Toolbar {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let mut bound = target.bounding_box();
        // Move things off border.
        bound.top_left += Point::new(2, 0);
        bound.size.width -= 4;

        let mut items: Vec<ToolbarItem> = Vec::new();
        if self.settings.show_fps() {
            items.push(ToolbarItem::Fps(FpsCounterView::from_fps_counter(
                &self.fps,
            )));
        }

        items.push(ToolbarItem::Network(NetworkWidgetView::from_network(
            &self.network,
        )));

        LinearLayout::horizontal(Views::new(items.as_mut_slice()))
            .with_spacing(spacing::FixedMargin(2))
            .arrange()
            .align_to(&bound, horizontal::Left, vertical::Center)
            .draw(target)?;

        self.clock
            .clone()
            .align_to(&bound, horizontal::Right, vertical::Center)
            .draw(target)?;

        Ok(())
    }
}

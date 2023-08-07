use crate::application::toolbar::Toolbar;
use crate::application::widgets::keyboard::KeyboardTesterWidget;
use crate::macguiver::application::{Application, UpdateResult};
use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::events::keyboard::Keycode;
use crate::macguiver::views::Widget;
use crate::main_inner::Flags;
use crate::platform::{PlatformState, WindowManager};
use embedded_graphics::geometry::Point;
use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;

mod menu;
mod toolbar;
mod widgets;

pub trait Panel {
    type NextView: Panel;

    fn new() -> Self;
    fn update(&mut self, state: &PlatformState) -> Result<Option<Self::NextView>, String>;
    fn draw(&self, target: &mut DrawBuffer<BinaryColor>);
}

pub struct KeyboardTesterView {
    widget: KeyboardTesterWidget,
}

impl Panel for KeyboardTesterView {
    type NextView = TopLevelView;

    fn new() -> Self {
        Self {
            widget: KeyboardTesterWidget::new(),
        }
    }

    fn update(&mut self, state: &PlatformState) -> Result<Option<TopLevelView>, String> {
        self.widget.set_state(*state.keys());
        if state.pressed().contains(Keycode::Tab) {
            Ok(Some(TopLevelView::menu()))
        } else {
            Ok(None)
        }
    }

    fn draw(&self, target: &mut DrawBuffer<BinaryColor>) {
        self.widget.draw(target).unwrap();
    }
}

pub struct IconView;

impl Panel for IconView {
    type NextView = TopLevelView;

    fn new() -> Self {
        Self {}
    }

    fn update(&mut self, state: &PlatformState) -> Result<Option<Self::NextView>, String> {
        if state.pressed().contains(Keycode::Tab) {
            Ok(Some(TopLevelView::keyboard_tester()))
        } else {
            Ok(None)
        }
    }

    fn draw(&self, target: &mut DrawBuffer<BinaryColor>) {
        let font_images = [
            include_bytes!("../assets/icons/arrow_down.raw"),
            include_bytes!("../assets/icons/arrow_left.raw"),
            include_bytes!("../assets/icons/arrow_right.raw"),
            include_bytes!("../assets/icons/arrow_right_mini.raw"),
            include_bytes!("../assets/icons/arrow_up.raw"),
            include_bytes!("../assets/icons/atari_left.raw"),
            include_bytes!("../assets/icons/atari_right.raw"),
            include_bytes!("../assets/icons/battery_charging.raw"),
            include_bytes!("../assets/icons/battery_empty.raw"),
            include_bytes!("../assets/icons/battery_full.raw"),
            include_bytes!("../assets/icons/battery_half.raw"),
            include_bytes!("../assets/icons/bluetooth.raw"),
            include_bytes!("../assets/icons/box_empty.raw"),
            include_bytes!("../assets/icons/box_fill1.raw"),
            include_bytes!("../assets/icons/box_fill2.raw"),
            include_bytes!("../assets/icons/box_fill3.raw"),
            include_bytes!("../assets/icons/box_fill4.raw"),
            include_bytes!("../assets/icons/box_top.raw"),
            include_bytes!("../assets/icons/burger_menu_2.raw"),
            include_bytes!("../assets/icons/burger_menu_3.raw"),
            include_bytes!("../assets/icons/burger_menu_4.raw"),
            include_bytes!("../assets/icons/checkbox_empty.raw"),
            include_bytes!("../assets/icons/checkbox_full.raw"),
            include_bytes!("../assets/icons/checkbox_mark.raw"),
            include_bytes!("../assets/icons/dot_middle.raw"),
            include_bytes!("../assets/icons/lock_locked.raw"),
            include_bytes!("../assets/icons/lock_unlocked.raw"),
            include_bytes!("../assets/icons/mem_32.raw"),
            include_bytes!("../assets/icons/mem_64.raw"),
            include_bytes!("../assets/icons/mem_128.raw"),
            include_bytes!("../assets/icons/mem_none.raw"),
            include_bytes!("../assets/icons/network_eth.raw"),
            include_bytes!("../assets/icons/network_globe.raw"),
            include_bytes!("../assets/icons/network_wifi.raw"),
            include_bytes!("../assets/icons/null.raw"),
            include_bytes!("../assets/icons/speaker_empty.raw"),
            include_bytes!("../assets/icons/speaker_full.raw"),
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

/// Top-level Views for the MiSTer application.
pub enum TopLevelView {
    KeyboardTester(KeyboardTesterView),
    IconView(IconView),
    MainMenu(menu::MainMenu),
}

impl TopLevelView {
    pub fn keyboard_tester() -> Self {
        Self::KeyboardTester(KeyboardTesterView::new())
    }

    pub fn icon() -> Self {
        Self::IconView(IconView::new())
    }

    pub fn menu() -> Self {
        Self::MainMenu(menu::MainMenu::new())
    }
}

impl Panel for TopLevelView {
    type NextView = TopLevelView;

    fn new() -> Self {
        Self::KeyboardTester(KeyboardTesterView::new())
    }

    fn update(&mut self, state: &PlatformState) -> Result<Option<Self::NextView>, String> {
        match self {
            TopLevelView::KeyboardTester(inner) => inner.update(state),
            TopLevelView::IconView(inner) => inner.update(state),
            TopLevelView::MainMenu(inner) => inner.update(state),
        }
    }

    fn draw(&self, target: &mut DrawBuffer<BinaryColor>) {
        match self {
            TopLevelView::KeyboardTester(inner) => inner.draw(target),
            TopLevelView::IconView(inner) => inner.draw(target),
            TopLevelView::MainMenu(inner) => inner.draw(target),
        }
    }
}

pub struct MiSTer {
    toolbar: toolbar::Toolbar,
    view: TopLevelView,
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
        Self {
            toolbar: Toolbar::default(),
            view: TopLevelView::new(),
        }
    }

    fn update(&mut self, state: &PlatformState) -> UpdateResult {
        let should_redraw_toolbar = self.toolbar.update();
        match self.view.update(state) {
            Ok(Some(next_view)) => self.view = next_view,
            Ok(None) => {}
            Err(e) => panic!("{}", e),
        };

        UpdateResult::Redraw(should_redraw_toolbar, true)
    }

    fn draw_title(&self, target: &mut DrawBuffer<BinaryColor>) {
        self.toolbar.draw(target);
    }

    fn draw_main(&self, target: &mut DrawBuffer<BinaryColor>) {
        self.view.draw(target);
    }
}

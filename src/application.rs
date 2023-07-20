use crate::display::OsdDisplayView;
use crate::osd::OsdSetSize;
use crate::{fpga, input, menu, osd, spi, user_io};
use embedded_fps::{StdClock, FPS};
use embedded_graphics::mono_font::ascii::{FONT_4X6, FONT_6X9};
use embedded_graphics::primitives::{Line, Rectangle};
use embedded_graphics::{
    mono_font,
    mono_font::MonoTextStyle,
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Triangle},
    text::Text,
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};
use embedded_layout::{layout::linear::LinearLayout, prelude::*};

pub struct Application {
    osd: OsdDisplayView<BinaryColor>,
    title: OsdDisplayView<BinaryColor>,

    fps_counter: FPS<1000, StdClock>,

    window_osd: Window,
    window_title: Window,
}

impl Default for Application {
    fn default() -> Self {
        OsdSetSize(19);

        let mut osd = OsdDisplayView::main();
        let mut title = OsdDisplayView::title();

        let display_area = osd.bounding_box();

        // Style objects
        let text_style = MonoTextStyle::new(&FONT_6X9, BinaryColor::On);

        let thin_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
        let thick_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 3);
        let fill_on = PrimitiveStyle::with_fill(BinaryColor::On);
        let fill_off = PrimitiveStyle::with_fill(BinaryColor::Off);

        // Primitives to be displayed
        let triangle = Triangle::new(Point::new(0, 0), Point::new(12, 0), Point::new(6, 12))
            .into_styled(thin_stroke);

        let circle = Circle::new(Point::zero(), 11).into_styled(thick_stroke);
        let circle2 = Circle::new(Point::zero(), 15).into_styled(fill_on);
        let triangle2 = Triangle::new(Point::new(0, 0), Point::new(10, 0), Point::new(5, 8))
            .into_styled(fill_off);
        let text = Text::new("embedded-layout", Point::zero(), text_style);

        // The layout
        LinearLayout::vertical(
            Chain::new(text)
                .append(LinearLayout::horizontal(Chain::new(triangle).append(circle)).arrange())
                .append(
                    Chain::new(triangle2.align_to(&circle2, horizontal::Center, vertical::Top))
                        .append(circle2),
                ),
        )
        .with_alignment(horizontal::Center)
        .arrange()
        .align_to(&display_area, horizontal::Center, vertical::Center)
        .draw(&mut osd)
        .unwrap();

        Line::new(Point::new(0, 0), Point::new(16, 0))
            .into_styled(thin_stroke)
            .draw(&mut title)
            .unwrap();
        Line::new(Point::new(8, 14), Point::new(24, 14))
            .into_styled(thin_stroke)
            .draw(&mut title)
            .unwrap();
        Line::new(Point::new(16, 15), Point::new(32, 15))
            .into_styled(thin_stroke)
            .draw(&mut title)
            .unwrap();
        Line::new(Point::new(24, 16), Point::new(40, 16))
            .into_styled(thin_stroke)
            .draw(&mut title)
            .unwrap();
        Line::new(Point::new(32, 17), Point::new(48, 17))
            .into_styled(thin_stroke)
            .draw(&mut title)
            .unwrap();

        // starts the StdClock
        // `200` MAX_FPS is more than enough since `SimulatorDisplay`
        // doesn't reach more than `15` FPS when using `BinaryColor`.
        let fps_counter = FPS::default();
        // let output_settings = OutputSettingsBuilder::new()
        //     .theme(BinaryColorTheme::OledBlue)
        //     .build();

        // Window::new("Hello World", &output_settings).show_static(&osd);
        let window_osd = Window::new(
            "OSD",
            &OutputSettingsBuilder::new()
                .theme(BinaryColorTheme::OledBlue)
                .build(),
        );

        let window_title = Window::new(
            "Title",
            &OutputSettingsBuilder::new()
                .theme(BinaryColorTheme::OledBlue)
                .build(),
        );

        Self {
            osd,
            title,
            fps_counter,
            window_osd,
            window_title,
        }
    }
}

impl Application {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(&mut self) -> Result<(), String> {
        unsafe {
            while fpga::is_fpga_ready(1) == 0 {
                fpga::fpga_wait_to_reset();
            }

            loop {
                // Polling coroutine.
                user_io::user_io_poll();
                input::input_poll(0);

                // UI coroutine.
                menu::HandleUI();

                // Update the FPS counter
                let fps = self.fps_counter.tick();
                Rectangle::new(Point::new(0, 0), Size::new(64, 10))
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
                    .draw(&mut self.osd)
                    .unwrap();
                let text_style = MonoTextStyle::new(&FONT_4X6, BinaryColor::On);
                Text::new(&format!("{} FPS", fps), Point::new(1, 8), text_style)
                    .draw(&mut self.osd)
                    .unwrap();

                #[cfg(feature = "de10")]
                {
                    let n = if user_io::is_menu() != 0 {
                        19
                    } else {
                        osd::OsdGetSize()
                    };

                    for line in self.osd.line_iter() {
                        let line_buffer = self.osd.get_binary_line_array(line);
                        spi::spi_osd_cmd_cont(osd::OSD_CMD_WRITE | (line as u8));
                        spi::spi_write(line_buffer.as_ptr(), 256, 0);
                        spi::DisableOsd();
                    }
                    for line in self.title.line_iter() {
                        let line_buffer = self.title.get_binary_line_array(line);
                        spi::spi_osd_cmd_cont(osd::OSD_CMD_WRITE | (line as u8));
                        spi::spi_write(line_buffer.as_ptr(), 256, 0);
                        spi::DisableOsd();
                    }

                    extern "C" {
                        fn mcd_poll();
                        fn neocd_poll();
                        fn pcecd_poll();
                        fn saturn_poll();
                    }

                    if user_io::is_megacd() != 0 {
                        mcd_poll();
                    }
                    if user_io::is_pce() != 0 {
                        pcecd_poll();
                    }
                    if user_io::is_saturn() != 0 {
                        saturn_poll();
                    }
                    if user_io::is_neogeo_cd() != 0 {
                        neocd_poll();
                    }
                }

                #[cfg(not(feature = "de10"))]
                {
                    self.window_osd.update(&self.osd.inner);
                    self.window_title.update(&self.title.inner);

                    if self
                        .window_osd
                        .events()
                        .any(|e| e == embedded_graphics_simulator::SimulatorEvent::Quit)
                    {
                        break;
                    }
                    if self
                        .window_title
                        .events()
                        .any(|e| e == embedded_graphics_simulator::SimulatorEvent::Quit)
                    {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }
        }
        Ok(())
    }
}

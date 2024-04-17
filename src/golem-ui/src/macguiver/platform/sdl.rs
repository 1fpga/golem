//! An SDL3 platform for MacGUIver.
use crate::macguiver::platform::sdl::settings::OutputSettings;
use crate::macguiver::platform::Platform;
use embedded_graphics::pixelcolor::raw::ToBytes;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::{PixelColor, Size};
use sdl3::event::Event;
use sdl3::EventPump;
use std::cell::RefCell;
use std::rc::Rc;

pub mod settings;
pub mod theme;
pub mod window;
pub use window::Window;

mod output;

thread_local! {
    static SDL_CONTEXT: std::cell::RefCell<sdl3::Sdl> = {
        std::cell::RefCell::new(sdl3::init().unwrap())
    };
}

#[derive(Default)]
pub struct SdlInitState {
    output_settings: OutputSettings,
}

impl SdlInitState {
    pub fn new(output_settings: OutputSettings) -> Self {
        Self { output_settings }
    }
}

pub struct SdlState {
    events: Vec<Event>,
}

impl SdlState {
    pub fn events(&self) -> impl Iterator<Item = Event> + '_ {
        self.events.iter().cloned()
    }
}

pub struct SdlPlatform<C: PixelColor> {
    pub event_pump: Rc<RefCell<EventPump>>,
    pub joystick: Rc<RefCell<sdl3::JoystickSubsystem>>,
    pub gamepad: Rc<RefCell<sdl3::GamepadSubsystem>>,
    pub video: Rc<RefCell<sdl3::VideoSubsystem>>,

    has_windows: bool,
    base_window: Option<Window<C>>,

    init_state: SdlInitState,

    phantom: std::marker::PhantomData<C>,
}

impl<C: PixelColor> SdlPlatform<C> {
    fn with<R>(&mut self, function: impl FnOnce(&mut sdl3::Sdl) -> R) -> R {
        SDL_CONTEXT.with(|ctx| function(&mut ctx.borrow_mut()))
    }

    pub fn events(&mut self) -> Vec<Event> {
        let mut event_pump = self.event_pump.borrow_mut();
        event_pump.pump_events();
        event_pump.poll_iter().collect()
    }
}

impl<C> Platform for SdlPlatform<C>
where
    C: PixelColor + Into<Rgb888> + From<Rgb888>,
    <<C as PixelColor>::Raw as ToBytes>::Bytes: AsRef<[u8]>,
    <C as PixelColor>::Raw: From<C>,
{
    type InitState = SdlInitState;
    type Window = Window<C>;
    type State = SdlState;
    type Event = sdl3::event::Event;

    fn init(init_state: SdlInitState) -> Self {
        let (joystick, gamepad, event_pump, video) = SDL_CONTEXT.with(|context| {
            let ctx = context.borrow();
            // Initialize subsystems.
            let joystick = ctx.joystick().unwrap();
            joystick.set_joystick_events_enabled(true);
            let gamepad = ctx.gamepad().unwrap();
            let event_pump = ctx.event_pump().unwrap();
            let video = ctx.video().unwrap();

            (joystick, gamepad, event_pump, video)
        });

        Self {
            init_state,
            event_pump: Rc::new(RefCell::new(event_pump)),
            joystick: Rc::new(RefCell::new(joystick)),
            gamepad: Rc::new(RefCell::new(gamepad)),
            video: Rc::new(RefCell::new(video)),
            has_windows: false,
            base_window: None,
            phantom: std::marker::PhantomData,
        }
    }

    fn window(&mut self, title: &str, size: Size) -> Self::Window {
        self.has_windows = true;
        Window::new(self, title, size)
    }

    fn event_loop(&mut self, mut loop_fn: impl FnMut(&mut Self, &Self::State) -> bool) {
        if !self.has_windows {
            self.base_window = Some(self.window("", Size::new(1, 1)));
        }

        let event_pump = self.event_pump.clone();

        'main: loop {
            event_pump.borrow_mut().pump_events();
            let events: Vec<Event> = event_pump.borrow_mut().poll_iter().collect();

            let state = SdlState { events };

            if loop_fn(self, &state) {
                break 'main;
            }
        }
    }
}

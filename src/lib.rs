#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_int, CStr};
use tracing_subscriber::fmt::time::SystemTime;

// TODO: make all these modules test friendly.
#[cfg(not(test))]
pub mod battery;
#[cfg(not(test))]
pub mod bootcore;
#[cfg(not(test))]
pub mod charrom;
#[cfg(not(test))]
pub mod fpga;
#[cfg(not(test))]
pub mod hardware;
#[cfg(not(test))]
pub mod input;
#[cfg(not(test))]
pub mod menu;
#[cfg(not(test))]
pub mod offload;
#[cfg(not(test))]
pub mod osd;
#[cfg(not(test))]
pub mod scheduler;
#[cfg(not(test))]
pub mod shmem;
#[cfg(not(test))]
pub mod smbus;
#[cfg(not(test))]
pub mod spi;
#[cfg(not(test))]
pub mod support;
#[cfg(not(test))]
pub mod user_io;

#[cfg(not(test))]
mod application;
#[cfg(not(test))]
mod main_inner;

pub mod cfg;
pub mod core;
pub mod file_io;
pub mod macguiver;
pub mod platform;
pub mod video;

#[cfg(feature = "platform_de10")]
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() -> isize {
    if std::env::var_os("SDL_VIDEO_DRIVER").is_none() {
        std::env::set_var("SDL_VIDEO_DRIVER", "evdev");
    }

    unsafe {
        let sdl = sdl3::init().unwrap();

        // eprintln!("================ evdev ================");
        // evdev::enumerate().for_each(|(path, dev)| eprintln!("evdev: {:?} {:?}", path, dev.name()));
        //
        // let now = std::time::SystemTime::now();
        // 'evdev_main: loop {
        //     let mut dev = evdev::Device::open("/dev/input/event0").unwrap();
        //
        //     for ev in dev.fetch_events().unwrap() {
        //         eprintln!("evdev event: {:?}", ev);
        //
        //         if now.elapsed().unwrap().as_secs() > 1 {
        //             break 'evdev_main;
        //         }
        //     }
        // }

        eprintln!("================ SDL ================");

        for driver in sdl3::video::drivers() {
            eprintln!("driver: {driver:?}");
        }

        let gamepad = sdl.game_controller().unwrap();
        eprintln!("gamepad: {:?}", gamepad);
        let mut gamepads = Vec::new();
        for i in 0..10 {
            let g = gamepad.name_for_index(i);
            let g = match g {
                Ok(g) => g,
                Err(e) => {
                    continue;
                }
            };

            eprintln!("gamepad {i}: {g:?}");
            gamepads.push(gamepad.open(i).unwrap());
            // eprintln!("  mapping: {:?}", sdl3::sys::SDL_Game)
        }

        let video = sdl.video().unwrap();
        eprintln!("Selected driver: {:?}", video.current_video_driver());
        let window = video
            .window("Hello world", 100, 100)
            .input_grabbed()
            .build()
            .unwrap();

        let keyboard = sdl.keyboard();
        // eprintln!("keyboard: {:?}", keyboard);

        let joystick = sdl.joystick().unwrap();
        eprintln!("joysticks: {:?}", joystick);
        for i in 0..10 {
            let joystick = joystick.open(i);
            let joystick = match joystick {
                Ok(j) => {
                    eprintln!("joystick {}: {:?}", i, j.name());
                }
                Err(e) => {}
            };
        }

        let mut event_pump = sdl.event_pump().unwrap();

        let mut event = sdl.event().unwrap();
        event.add_event_watch(|ev| {
            eprintln!("cb event: {ev:?}");
        });

        loop {
            event_pump.pump_events();

            for ev in event_pump.poll_iter() {
                eprintln!("sdl event: {ev:?}");

                if matches!(ev, sdl3::event::Event::Quit { .. }) {
                    std::process::exit(0);
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    std::process::exit(0);
    //
    // let sdl = sdl2::init().unwrap();
    // sdl2::video::drivers().for_each(|d| println!("driver: {}", d));
    //
    // let video = sdl.video().unwrap();
    // let mut ev_pump = sdl.event_pump().unwrap();
    //
    // let window = video
    //     .window("rust-sdl2 demo", 800, 600)
    //     .position_centered()
    //     .build()
    //     .unwrap();
    //
    // loop {
    //     for ev in ev_pump.poll_iter() {
    //         match ev {
    //             sdl2::event::Event::Quit { .. } => return 0,
    //             _ => {
    //                 eprintln!("sdl2 event: {:?}", ev);
    //             }
    //         }
    //     }
    //
    //     std::thread::sleep(std::time::Duration::from_millis(100));
    // }

    main_inner::main();
    0
}

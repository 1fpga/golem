// We don't want to build the binary in test mode.
#![cfg(not(test))]

// TODO: make all these modules test friendly.
pub mod battery;
pub mod bootcore;
pub mod charrom;
pub mod ffi;
pub mod hardware;
pub mod input;
pub mod menu;
pub mod osd;
pub mod scheduler;
pub mod shmem;
pub mod smbus;
pub mod spi;
pub mod support;
pub mod user_io;

mod application;
mod data;
mod main_inner;

pub mod config;
pub mod core;
pub mod file_io;
pub mod macguiver;
pub mod offload;
pub mod platform;
pub mod video;

fn main() {
    if let Err(e) = main_inner::main() {
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }
}

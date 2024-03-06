// We don't want to build the binary in test mode.
#![cfg(not(test))]

// TODO: make all these modules test friendly.
pub mod hardware;
pub mod scheduler;

mod application;
mod data;
mod main_inner;

pub mod file_io;
pub mod input;
pub mod macguiver;
pub mod platform;

fn main() {
    if let Err(e) = main_inner::main() {
        tracing::error!("Application error: {}", e);
        std::process::exit(1);
    }
}

// We don't want to build the binary in test mode.
#![cfg(not(test))]

// TODO: make all these modules test friendly.
pub mod battery;
pub mod bootcore;
pub mod charrom;
pub mod fpga;
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
mod main_inner;

pub mod cfg;
pub mod core;
pub mod file_io;
pub mod macguiver;
pub mod offload;
pub mod platform;
pub mod video;

#[cfg(feature = "platform_de10")]
fn main() {
    main_inner::main();
}

#[cfg(not(feature = "platform_de10"))]
fn main() {
    main_inner::main();
}

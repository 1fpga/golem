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
pub mod window_manager;

mod application;
mod main_inner;

pub mod cfg;
pub mod core;
pub mod file_io;
pub mod macgyver;
pub mod offload;
pub mod video;
pub mod widgets;

#[cfg(feature = "platform_de10")]
fn main() {
    main_inner::main();
}

#[cfg(not(feature = "platform_de10"))]
fn main() {
    main_inner::main();
}

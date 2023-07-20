// TODO: make all these modules test friendly.
#[cfg(not(test))]
pub mod application;
#[cfg(not(test))]
pub mod battery;
#[cfg(not(test))]
pub mod bootcore;
#[cfg(not(test))]
pub mod charrom;
#[cfg(not(test))]
pub mod display;
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

mod main_inner;

pub mod cfg;
pub mod core;
pub mod file_io;
pub mod offload;
pub mod video;

#[cfg(feature = "de10")]
#[no_mangle]
pub unsafe extern "C" fn main() {
    main_inner();
}

#[cfg(not(feature = "de10"))]
fn main() {
    main_inner::main();
}

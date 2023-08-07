#![allow(clippy::missing_safety_doc)]

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(not(test))] {
        pub mod battery;
        pub mod bootcore;
        pub mod charrom;
        pub mod hardware;
        pub mod input;
        pub mod menu;
        pub mod offload;
        pub mod osd;
        pub mod scheduler;
        pub mod shmem;
        pub mod smbus;
        pub mod spi;
        pub mod support;
        pub mod user_io;

        mod application;
        mod main_inner;
    }
}

pub mod config;
pub mod core;
pub mod ffi;
pub mod file_io;
pub mod macguiver;
pub mod platform;
pub mod video;

#[cfg(feature = "platform_de10")]
#[no_mangle]
pub extern "C" fn main() -> isize {
    use tracing::error;

    match main_inner::main() {
        Ok(_) => 0,
        Err(e) => {
            error!("Application error: {}", e);
            1
        }
    }
}

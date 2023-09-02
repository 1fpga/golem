#![allow(clippy::missing_safety_doc)]

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(not(test))] {
        pub mod bootcore;
        pub mod charrom;
        pub mod hardware;
        pub mod offload;
        pub mod scheduler;
    }
}

pub mod data;
mod main_inner;

pub mod application;
pub mod config;
pub mod core;
pub mod file_io;
pub mod macguiver;
pub mod platform;
pub mod video;

#[cfg(not(test))]
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

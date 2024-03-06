#![allow(clippy::missing_safety_doc)]

pub mod data;
mod main_inner;

pub mod hardware;

pub mod application;
pub mod file_io;
pub mod input;
pub mod macguiver;
pub mod platform;

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

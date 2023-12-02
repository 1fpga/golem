#![allow(clippy::missing_safety_doc)]

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(not(test))] {
        pub mod bootcore;
        pub mod hardware;
        pub mod scheduler;
    }
}

pub mod data;
mod main_inner;

pub mod application;
pub mod file_io;
pub mod input;
pub mod macguiver;
pub mod platform;
pub mod video;

// const LOGO: &[u8] = include_bytes!("../assets/logo.png");
// #[no_mangle]
// pub static mut _binary_logo_png_start: *const u8 = LOGO.as_ptr();
// #[no_mangle]
// pub static mut _binary_logo_png_end: *const u8 = unsafe { LOGO.as_ptr().add(LOGO.len()) };

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

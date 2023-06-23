//! Scheduler implementation for MiSTer.
//!
//! This is a dummy implementation of the Scheduler. The original scheduler
//! uses coroutines which don't really work well in Rust. Instead, we should
//! be using Async functions waiting for each others. However, this is a
//! complex task and I don't have the time to do it right now, so I'll push
//! it to later.
//!
//! Currently, this uses a single thread that polls both the FPGA and the
//! user input.
// TODO: Move this to proper async functions for coroutines.
use crate::{fpga, input, menu, osd, user_io};
use std::thread::{spawn, JoinHandle};

static mut SCHEDULER_THREAD: Option<JoinHandle<()>> = None;

#[no_mangle]
pub extern "C" fn scheduler_init() {
    unsafe {
        SCHEDULER_THREAD = Some(spawn(|| {
            while fpga::is_fpga_ready(1) == 0 {
                fpga::fpga_wait_to_reset();
            }

            loop {
                // Polling coroutine.
                user_io::user_io_poll();
                input::input_poll(0);

                // UI coroutine.
                menu::HandleUI();
                osd::OsdUpdate();
            }
        }));
    }
}

#[no_mangle]
pub extern "C" fn scheduler_run() {
    unsafe {
        if let Some(thread) = SCHEDULER_THREAD.take() {
            thread.join().unwrap();
        }
    }
}

#[no_mangle]
pub extern "C" fn scheduler_yield() {}

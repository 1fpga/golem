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

#[no_mangle]
pub extern "C" fn scheduler_yield() {}

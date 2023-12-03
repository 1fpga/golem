use crate::application::panels::progress::{progress_bar, ProgressBarUpdate};
use crate::application::GoLEmApp;
use std::sync::atomic::{AtomicI8, Ordering};

pub fn progress_tester(app: &mut GoLEmApp) {
    let state = AtomicI8::new(1);

    crossbeam_utils::thread::scope(|s| {
        s.spawn(|_| {
            for _ in 0..10 {
                state.fetch_add(1, Ordering::SeqCst);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        });

        progress_bar(app, "Progress Tester", 10, || {
            let s = state.load(Ordering::SeqCst) as u32;
            if s >= 10 {
                ProgressBarUpdate::Done
            } else {
                ProgressBarUpdate::UpdateBar(s)
            }
        });
    })
    .unwrap();
}

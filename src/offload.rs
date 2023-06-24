use core_affinity::CoreId;
use std::sync::mpsc;
use std::time::Duration;

/// A worker thread that executes work from a queue. The CPU affinity can be
/// specified at creation. The worker thread will run until the `quit` method
/// is called. Send work to it by calling the `queue` method with a Rust
/// function or an extern "C" function pointer.
pub struct Worker {
    thread: std::thread::JoinHandle<()>,
    queue: mpsc::Sender<Box<dyn FnOnce() -> () + Send>>,
    quit: mpsc::Sender<()>,
}

impl Worker {
    pub fn new(cpu: Option<CoreId>) -> Self {
        let (queue, queue_recv) = mpsc::channel::<Box<dyn FnOnce() -> () + Send>>();
        let (quit, quit_recv) = mpsc::channel::<()>();

        let thread = std::thread::spawn(move || {
            if let Some(cpu) = cpu {
                core_affinity::set_for_current(cpu);
            }

            loop {
                // Wait for work or check if we need to quit.
                // Don't stop as long as we have work to do.
                if let Ok(work) = queue_recv.recv_timeout(Duration::from_millis(100)) {
                    // Execute the function we received.
                    work();
                } else if quit_recv.try_recv().is_ok() {
                    break;
                }
            }
        });

        Self {
            thread,
            queue,
            quit,
        }
    }

    /// Request to quit the worker thread
    pub fn quit(self) -> std::thread::JoinHandle<()> {
        self.quit.send(()).unwrap();
        self.thread
    }

    /// Queue a function to be executed by the worker thread. The function must
    /// return and be able to be sent across threads.
    pub fn queue(&self, work: impl FnOnce() + Send + 'static) {
        self.queue.send(Box::new(work)).unwrap();
    }
}

static mut BASE_WORKER: Option<Worker> = None;

#[no_mangle]
pub unsafe extern "C" fn offload_start() {
    // Use the first CPU available.
    BASE_WORKER = Some(Worker::new(
        core_affinity::get_core_ids().and_then(|cids| cids.first().cloned()),
    ));
}

#[no_mangle]
pub unsafe extern "C" fn offload_stop() {
    println!("Waiting for offloaded work to finish...");
    BASE_WORKER.take().unwrap().quit().join().unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn offload_add_work(handler: *const fn() -> ()) {
    let h = *handler;
    BASE_WORKER.as_ref().unwrap().queue(move || h());
}

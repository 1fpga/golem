use std::collections::VecDeque;
use std::ffi::c_void;

static mut THREAD_HANDLE: Option<libc::pthread_t> = None;
static mut THREAD_COND_WORK: Option<libc::pthread_cond_t> = None;
static mut THREAD_COND_AVAILABLE: Option<libc::pthread_cond_t> = None;
static mut QUEUE_LOCK: Option<libc::pthread_mutex_t> = None;

static mut QUEUE: VecDeque<*const fn() -> ()> = VecDeque::new();
static mut QUIT: bool = false;

extern "C" fn worker_thread(_context: *mut c_void) -> *mut c_void {
    unsafe {
        loop {
            // Wait for work.
            libc::pthread_mutex_lock(QUEUE_LOCK.as_mut().unwrap());
            if QUEUE.is_empty() {
                if QUIT {
                    libc::pthread_mutex_unlock(QUEUE_LOCK.as_mut().unwrap());
                    break;
                }

                // Wait for work signal.
                libc::pthread_cond_wait(
                    THREAD_COND_WORK.as_mut().unwrap(),
                    QUEUE_LOCK.as_mut().unwrap(),
                );

                // Quit flag was set and quuee still empty, quit.
                if QUIT && QUEUE.is_empty() {
                    libc::pthread_mutex_unlock(QUEUE_LOCK.as_mut().unwrap());
                    break;
                }
            }

            // Get work.
            let work = QUEUE.pop_front().unwrap();
            libc::pthread_mutex_unlock(QUEUE_LOCK.as_mut().unwrap());

            // Execute.
            (*work)();

            libc::pthread_cond_signal(THREAD_COND_AVAILABLE.as_mut().unwrap());
        }

        std::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn offload_start() {
    THREAD_COND_AVAILABLE = Some(std::mem::zeroed());
    libc::pthread_cond_init(
        THREAD_COND_AVAILABLE.as_mut().unwrap(),
        std::ptr::null_mut(),
    );

    THREAD_COND_WORK = Some(std::mem::zeroed());
    libc::pthread_cond_init(THREAD_COND_WORK.as_mut().unwrap(), std::ptr::null_mut());

    QUEUE_LOCK = Some(std::mem::zeroed());
    libc::pthread_mutex_init(QUEUE_LOCK.as_mut().unwrap(), std::ptr::null_mut());

    QUEUE = VecDeque::new();
    QUIT = false;

    let mut attr: libc::pthread_attr_t = std::mem::zeroed();
    libc::pthread_attr_init(&mut attr);

    // Set affinity to core #0 since main runs on core #1
    let mut set: libc::cpu_set_t = std::mem::zeroed();
    libc::CPU_ZERO(&mut set);
    libc::CPU_SET(0, &mut set);
    libc::pthread_attr_setaffinity_np(&mut attr, std::mem::size_of::<libc::cpu_set_t>(), &set);

    THREAD_HANDLE = Some(std::mem::zeroed());
    libc::pthread_create(
        THREAD_HANDLE.as_mut().unwrap(),
        &attr,
        worker_thread,
        std::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "C" fn offload_stop() {
    libc::pthread_mutex_lock(QUEUE_LOCK.as_mut().unwrap());

    QUIT = true;
    libc::pthread_cond_signal(THREAD_COND_WORK.as_mut().unwrap());

    libc::pthread_mutex_unlock(QUEUE_LOCK.as_mut().unwrap());

    println!("Waiting for offloaded work to finish...");
    libc::pthread_join(THREAD_HANDLE.unwrap(), std::ptr::null_mut());
    println!("Done");
}

#[no_mangle]
pub unsafe extern "C" fn offload_add_work(handler: *const fn() -> ()) {
    libc::pthread_mutex_lock(QUEUE_LOCK.as_mut().unwrap());

    QUEUE.push_back(handler);
    libc::pthread_cond_signal(THREAD_COND_WORK.as_mut().unwrap());
    libc::pthread_mutex_unlock(QUEUE_LOCK.as_mut().unwrap());
}

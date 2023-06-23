use std::ffi::{CStr, CString};

pub mod battery;
pub mod charrom;
pub mod file_io;
pub mod fpga;
pub mod hardware;
pub mod input;
pub mod menu;
pub mod offload;
pub mod osd;
pub mod scheduler;
pub mod shmem;
pub mod smbus;
pub mod spi;
pub mod user_io;

extern "C" {
    static version: *const u8;
}

#[no_mangle]
pub unsafe extern "C" fn main() {
    let mut set: libc::cpu_set_t = std::mem::zeroed();

    libc::CPU_ZERO(&mut set);
    libc::CPU_SET(1, &mut set);
    libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &set);

    offload::offload_start();

    fpga::fpga_io_init();

    // DISKLED_OFF();

    println!(
        r#"
        Minimig by Dennis van Weeren
        ARM Controller by Jakub Bednarski
        MiSTer code by Sorgelig
        Rust code by hansl

    "#
    );

    println!(
        "Version {}\n",
        CStr::from_ptr(version.add(5)).to_string_lossy()
    );

    let mut args = std::env::args();
    let (core, xml) = match [args.next(), args.next()] {
        [Some(c), Some(x)] => (
            Some(CString::new(c).unwrap()),
            Some(CString::new(x).unwrap()),
        ),
        [Some(c), _] => (Some(CString::new(c).unwrap()), None),
        _ => (None, None),
    };

    if let Some(core) = &core {
        println!("Core path: {}", core.to_string_lossy());
    }
    if let Some(xml) = &xml {
        println!("XML path: {}", xml.to_string_lossy());
    }

    if fpga::is_fpga_ready(1) == 0 {
        println!("\nGPI[31]==1. FPGA is uninitialized or incompatible core loaded.");
        println!("Quitting. Bye bye...\n");
        std::process::exit(1);
    }

    file_io::FindStorage();
    user_io::user_io_init(
        core.map(|str| str.as_ptr()).unwrap_or(std::ptr::null()),
        xml.map(|str| str.as_ptr()).unwrap_or(std::ptr::null()),
    );

    scheduler::scheduler_init();
    scheduler::scheduler_run();
}

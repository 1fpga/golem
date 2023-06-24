use clap::Parser;
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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Opts {
    /// Path to the core to load instantly.
    #[clap()]
    core: Option<String>,

    /// Path to the XML configuration file for the core.
    #[clap()]
    xml: Option<String>,
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

    let v = CStr::from_ptr(version.offset(5)).to_string_lossy();
    println!(
        r#"
        Minimig by Dennis van Weeren
        ARM Controller by Jakub Bednarski
        MiSTer code by Sorgelig
        Rust code by Hans Larsen

        Version {v}"#
    );

    let Opts { core, xml } = Opts::parse();

    if let Some(core) = &core {
        println!("Core path: {}", core);
    }
    if let Some(xml) = &xml {
        println!("XML path: {}", xml);
    }

    if fpga::is_fpga_ready(1) == 0 {
        println!("\nGPI[31]==1. FPGA is uninitialized or incompatible core loaded.");
        println!("Quitting. Bye bye...\n");
        std::process::exit(1);
    }

    file_io::FindStorage();
    let (core, xml) = (
        core.map(|str| CString::new(str).unwrap()),
        xml.map(|str| CString::new(str).unwrap()),
    );
    user_io::user_io_init(
        core.map(|str| str.as_ptr()).unwrap_or(std::ptr::null()),
        xml.map(|str| str.as_ptr()).unwrap_or(std::ptr::null()),
    );

    scheduler::scheduler_init();
    scheduler::scheduler_run();
}

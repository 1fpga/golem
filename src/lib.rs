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
    #[clap(default_value = "")]
    core: String,

    /// Path to the XML configuration file for the core.
    #[clap()]
    xml: Option<String>,
}

#[no_mangle]
pub unsafe extern "C" fn main() {
    let cores = core_affinity::get_core_ids().unwrap();
    // Always use the second core available, or the first one if there is only one.
    let core = cores.get(1).unwrap_or(cores.get(0).unwrap());
    core_affinity::set_for_current(*core);

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

    if !core.is_empty() {
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
        CString::new(core).unwrap(),
        xml.map(|str| CString::new(str).unwrap()),
    );

    user_io::user_io_init(
        core.as_bytes_with_nul().as_ptr(),
        xml.map(|str| str.as_bytes_with_nul().as_ptr())
            .unwrap_or(std::ptr::null()),
    );

    scheduler::scheduler_init();
    scheduler::scheduler_run();
}

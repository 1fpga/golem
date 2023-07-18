#![allow(clippy::missing_safety_doc)]
use clap::Parser;
use clap_verbosity_flag::Level as VerbosityLevel;
use clap_verbosity_flag::Verbosity;
use std::ffi::{CStr, CString};
use tracing::Level;
use tracing_subscriber::fmt::Subscriber;

// TODO: make all these modules test friendly.
#[cfg(not(test))]
pub mod battery;
#[cfg(not(test))]
pub mod bootcore;
#[cfg(not(test))]
pub mod charrom;
#[cfg(not(test))]
pub mod fpga;
#[cfg(not(test))]
pub mod hardware;
#[cfg(not(test))]
pub mod input;
#[cfg(not(test))]
pub mod menu;
#[cfg(not(test))]
pub mod offload;
#[cfg(not(test))]
pub mod osd;
#[cfg(not(test))]
pub mod scheduler;
#[cfg(not(test))]
pub mod shmem;
#[cfg(not(test))]
pub mod smbus;
#[cfg(not(test))]
pub mod spi;
#[cfg(not(test))]
pub mod support;
#[cfg(not(test))]
pub mod user_io;

pub mod cfg;
pub mod core;
pub mod file_io;
pub mod video;

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

    #[command(flatten)]
    verbose: Verbosity<clap_verbosity_flag::InfoLevel>,
}

#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn main() -> isize {
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

       ^55555Y!       ~55555Y!  .P&@&#~   ^?PGBBBG5J! :YYYYYYYYYYYYYJ!
       &@@@@@@@&     ~@@@@@@@@& J@@@@@G J@@@@@@@@@@@@:&@@@@@@@@@@@@@@@&
      ~@@@@@@@@@^   ^@@@@@@@@@&  ~YYJ~ G@@@@@#7!7J5J..PPPPP@@@@@@GPP55Y
      #@@@@5@@@@!  :@@@@P@@@@@?.##B5.  @@@@@@B.           .@@@@@#    ~5#&@@@&#J. .####&7:G&@!
     :@@@@&^@@@@J .@@@@~#@@@@@ 5@@@@@  5@@@@@@@&P~        Y@@@@@~  5@@@@#5#@@@@& 7@@@@@&@@@@
     G@@@@5 @@@@5 &@@@^^@@@@@5 @@@@@#   ~B@@@@@@@@&^      @@@@@&  &@@@@5..G@@@@& &@@@@@&J!7^
    .@@@@@. &@@@B#@@@~ B@@@@@.?@@@@@~      :?&@@@@@&     7@@@@@7 5@@@@@@@@@@@#Y ^@@@@@P
    5@@@@B  #@@@@@@@! :@@@@@G &@@@@& :#J^.   5@@@@@&     &@@@@@  G@@@@@?^::...  B@@@@@
    &@@@@^  G@@@@@@7  ?@@@@@::@@@@@7 G@@@@@@@@@@@@&^    .@@@@@Y  ^@@@@@&BGB#@#  @@@@@5
    :B&@B   J@&&&&7    Y&&@B  7#&&#  ?B&@@@@@@@&G!       7#&&&.   .Y#@@@@@@&B^  ~B&&&.

                                                                       Version {v}
"#
    );

    let Opts { core, xml, verbose } = Opts::parse();

    if !core.is_empty() {
        println!("Core path: {}", core);
    }
    if let Some(xml) = &xml {
        println!("XML path: {}", xml);
    }

    // Initialize tracing.
    let subscriber = Subscriber::builder();
    let subscriber = match verbose.log_level() {
        None => subscriber,
        Some(VerbosityLevel::Error) => subscriber.with_max_level(Level::ERROR),
        Some(VerbosityLevel::Warn) => subscriber.with_max_level(Level::WARN),
        Some(VerbosityLevel::Info) => subscriber.with_max_level(Level::INFO),
        Some(VerbosityLevel::Trace) => subscriber.with_max_level(Level::TRACE),
        Some(VerbosityLevel::Debug) => subscriber.with_max_level(Level::DEBUG),
    };
    subscriber
        .with_ansi(true)
        .with_writer(std::io::stderr)
        .init();

    if fpga::is_fpga_ready(1) == 0 {
        eprintln!("\nGPI[31]==1. FPGA is uninitialized or incompatible core loaded.");
        eprintln!("Quitting. Bye bye...\n");
        return 1;
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

    // Make sure we're in the right directory. Otherwise, relative paths
    // won't work. We set the current directory to be in the MiSTer
    // executable.
    // TODO: fix relative paths everywhere.
    std::env::set_current_dir(std::env::current_exe().unwrap().parent().unwrap()).unwrap();

    scheduler::scheduler_init();
    scheduler::scheduler_run();

    0
}

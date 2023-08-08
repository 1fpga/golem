use crate::application;
use clap::Parser;
use clap_verbosity_flag::Level as VerbosityLevel;
use clap_verbosity_flag::Verbosity;

use tracing::Level;
use tracing_subscriber::fmt::Subscriber;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Flags {
    /// Path to the core to load instantly.
    #[clap(default_value = "")]
    pub core: String,

    /// Path to the XML configuration file for the core.
    #[clap()]
    pub xml: Option<String>,

    #[command(flatten)]
    pub verbose: Verbosity<clap_verbosity_flag::InfoLevel>,
}

#[allow(unused)]
pub fn main() -> Result<(), String> {
    let cores = core_affinity::get_core_ids().unwrap();
    // Always use the second core available, or the first one if there is only one.
    let core = cores.get(1).unwrap_or(cores.get(0).unwrap());
    core_affinity::set_for_current(*core);

    let v = {
        #[cfg(feature = "platform_de10")]
        unsafe {
            extern "C" {
                static version: *const u8;
            }
            std::ffi::CStr::from_ptr(version.offset(5)).to_string_lossy()
        }

        #[cfg(not(feature = "platform_de10"))]
        "Desktop".to_string()
    };
    println!(include_str!("../assets/header.txt"), v = v);

    let opts = Flags::parse();

    // Initialize tracing.
    let subscriber = Subscriber::builder();
    let subscriber = match opts.verbose.log_level() {
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

    // Make sure we're in the right directory. Otherwise, relative paths
    // won't work. We set the current directory to be in the MiSTer
    // executable.
    // TODO: fix relative paths everywhere.
    std::env::set_current_dir(std::env::current_exe().unwrap().parent().unwrap()).unwrap();

    // Create the application and run it.
    let mut app = application::MiSTer::new();
    app.run(opts)
}

use std::path::PathBuf;

use clap::Parser;
use clap_verbosity_flag::Level as VerbosityLevel;
use clap_verbosity_flag::{LogLevel, Verbosity};
use tracing::{info, Level};
use tracing_subscriber::fmt::Subscriber;

use golem_ui::application;

#[derive(Copy, Clone, Debug, Default)]
pub struct NoneLevel;

impl LogLevel for NoneLevel {
    fn default() -> Option<clap_verbosity_flag::Level> {
        None
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Flags {
    /// Path to root script to run instead of the embedded one.
    /// The script will be run without safeguards.
    #[clap(long)]
    pub script: Option<PathBuf>,

    #[command(flatten)]
    pub verbose: Verbosity<clap_verbosity_flag::InfoLevel>,
}

fn main() {
    let cores = core_affinity::get_core_ids().unwrap();
    // Always use the second core available, or the first one if there is only one.
    let core = cores
        .get(1)
        .unwrap_or(cores.first().expect("Could not find a CPU?!"));
    core_affinity::set_for_current(*core);

    let v = format!(
        "{}-{}",
        env!("CARGO_PKG_VERSION"),
        &env!("VERGEN_GIT_SHA")[..8]
    );
    println!(include_str!("../assets/header.txt"), v = v);

    let opts = Flags::parse();
    // Initialize tracing.
    let subscriber = Subscriber::builder();
    let subscriber = match opts.verbose.log_level() {
        Some(VerbosityLevel::Error) => subscriber.with_max_level(Level::ERROR),
        Some(VerbosityLevel::Warn) => subscriber.with_max_level(Level::WARN),
        Some(VerbosityLevel::Info) => subscriber.with_max_level(Level::INFO),
        Some(VerbosityLevel::Debug) => subscriber.with_max_level(Level::DEBUG),
        None | Some(VerbosityLevel::Trace) => subscriber.with_max_level(Level::TRACE),
    };
    subscriber
        .with_ansi(true)
        .with_writer(std::io::stderr)
        .init();

    tracing::debug!(?opts);

    // Create the application and run it.
    let start = std::time::Instant::now();
    info!("Starting application...");
    golem_script::run(opts.script.as_ref(), application::GoLEmApp::new())
        .expect("Failed to run script");
    let elapsed = start.elapsed();
    info!(?elapsed, "Done");
}

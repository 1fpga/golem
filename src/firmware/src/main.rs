use clap::Parser;
use clap_verbosity_flag::Level as VerbosityLevel;
use clap_verbosity_flag::{LogLevel, Verbosity};
use firmware_ui::application;
use std::io::Read;
use std::path::PathBuf;
use tracing::{info, warn};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::Subscriber;
use tracing_subscriber::EnvFilter;

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

    /// Reset 1FPGA entirely before start. This will clear all state and settings.
    /// This is useful for testing. A prompt will ensure that this is what you want.
    #[clap(long, default_value = "false")]
    pub reset: bool,
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
    let level_filter = match opts.verbose.log_level() {
        Some(VerbosityLevel::Error) => LevelFilter::ERROR,
        Some(VerbosityLevel::Warn) => LevelFilter::WARN,
        Some(VerbosityLevel::Info) => LevelFilter::INFO,
        Some(VerbosityLevel::Debug) => LevelFilter::DEBUG,
        None | Some(VerbosityLevel::Trace) => LevelFilter::TRACE,
    };

    Subscriber::builder()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(level_filter.into())
                .from_env_lossy()
                .add_directive("reqwest=error".parse().unwrap()),
        )
        .with_ansi(true)
        .with_writer(std::io::stderr)
        .init();

    tracing::debug!(?opts);

    if opts.reset {
        info!("Are you sure you want to reset 1FPGA? This will clear all state and settings.");
        info!("Type 'y' to confirm, anything else to abort.");
        let mut confirm = [0u8; 1];
        std::io::stdin().read(&mut confirm).unwrap();
        if confirm[0] != b'y' {
            warn!("Aborting reset.");
        } else {
            warn!("Resetting 1FPGA...");
            if let Err(r) = std::fs::remove_dir_all("/media/fat/1fpga") {
                warn!(?r, "Failed to remove /media/fat/1fpga directory.");
                return;
            }
        }
    }

    // Create the application and run it.
    let start = std::time::Instant::now();
    info!("Starting application...");
    firmware_script::run(opts.script.as_ref(), application::OneFpgaApp::new())
        .expect("Failed to run 1fpga");
    let elapsed = start.elapsed();
    info!(?elapsed, "Done");
}

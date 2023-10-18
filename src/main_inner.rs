use clap::Parser;
use clap_verbosity_flag::{LogLevel, Verbosity};

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
    /// Path to the core to load instantly.
    #[clap(default_value = "")]
    pub core: String,

    /// Path to the XML configuration file for the core.
    #[clap()]
    pub xml: Option<String>,

    #[command(flatten)]
    pub verbose: Verbosity<clap_verbosity_flag::InfoLevel>,
}

#[cfg(not(test))]
#[allow(unused)]
pub fn main() -> Result<(), String> {
    use crate::application;
    use crate::macguiver::application::Application;
    use crate::platform::WindowManager;
    use clap_verbosity_flag::Level as VerbosityLevel;
    use tracing::Level;
    use tracing_subscriber::fmt::Subscriber;

    let cores = core_affinity::get_core_ids().unwrap();
    // Always use the second core available, or the first one if there is only one.
    let core = cores.get(1).unwrap_or(cores.get(0).unwrap());
    core_affinity::set_for_current(*core);

    let v = format!(
        "{}-{}",
        env!("CARGO_PKG_VERSION"),
        &env!("VERGEN_GIT_SHA")[..8]
    );
    println!(include_str!("../assets/header.txt"), v = v);

    let opts = Flags::parse();

    tracing::debug!(?opts);

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

    // Make sure we're in the right directory. Otherwise, relative paths
    // won't work. We set the current directory to be in the MiSTer
    // executable.
    // TODO: fix relative paths everywhere.
    // std::env::set_current_dir(std::env::current_exe().unwrap().parent().unwrap()).unwrap();

    // Create the application and run it.
    let mut app = application::MiSTer::new(WindowManager::default());
    app.run(opts);
    Ok(())
}

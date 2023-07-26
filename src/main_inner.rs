use crate::macguiver::application::Application;
use crate::{application, offload};
use clap::Parser;
use clap_verbosity_flag::Level as VerbosityLevel;
use clap_verbosity_flag::Verbosity;
use std::process;
use tracing::field::debug;
use tracing::{debug, error, Level};
use tracing_subscriber::fmt::Subscriber;

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

#[allow(unused)]
pub fn main() {
    let cores = core_affinity::get_core_ids().unwrap();
    // Always use the second core available, or the first one if there is only one.
    let core = cores.get(1).unwrap_or(cores.get(0).unwrap());
    core_affinity::set_for_current(*core);

    unsafe {
        offload::offload_start();

        crate::fpga::fpga_io_init();
    }

    // DISKLED_OFF();

    let v = {
        #[cfg(target_arch = "armv7")]
        {
            extern "C" {
                static version: *const u8;
            }
            CStr::from_ptr(version.offset(5)).to_string_lossy();
        }

        #[cfg(not(target_arch = "armv7"))]
        "Desktop".to_string()
    };
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

    #[cfg(feature = "platform_de10")]
    unsafe {
        if crate::fpga::is_fpga_ready(1) == 0 {
            debug!("GPI[31] == 1");
            error!("FPGA is uninitialized or incompatible core loaded.");
            error!("Quitting. Bye bye...\n");
            process::exit(1);
        }

        crate::file_io::FindStorage();
        let (core, xml) = (
            std::ffi::CString::new(core).unwrap(),
            xml.map(|str| std::ffi::CString::new(str).unwrap()),
        );

        crate::user_io::user_io_init(
            core.as_bytes_with_nul().as_ptr(),
            xml.map(|str| str.as_bytes_with_nul().as_ptr())
                .unwrap_or(std::ptr::null()),
        );
    }

    // Make sure we're in the right directory. Otherwise, relative paths
    // won't work. We set the current directory to be in the MiSTer
    // executable.
    // TODO: fix relative paths everywhere.
    std::env::set_current_dir(std::env::current_exe().unwrap().parent().unwrap()).unwrap();

    // Create the application and run it.
    let mut app = application::MiSTer::new();
    match app.run() {
        Ok(_) => {}
        Err(e) => {
            error!("Application error: {}", e);
            process::exit(1);
        }
    }
}

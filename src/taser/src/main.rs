use clap::Parser;
use clap_verbosity_flag::Level as VerbosityLevel;
use clap_verbosity_flag::Verbosity;
use fce_movie_format::{FceInputButton, FceInputGamepad};
use mister_fpga::config::Config;
use mister_fpga::core::buttons::{ButtonMap, MisterFpgaButtons};
use mister_fpga::core::MisterFpgaCore;
use mister_fpga::fpga::user_io::UserIoButtonSwitch;
use one_fpga::Core;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{debug, error, info, trace, Level};
use tracing_subscriber::fmt::Subscriber;

/// `taser` is a simple command-line interface to the 1FPGA Mister core
/// library. It is intended to be used as a standalone application, or as a
/// testbed for cores.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Flags {
    /// Path to the core to load.
    core: PathBuf,

    /// Path to the ROM to load.
    rom: PathBuf,

    /// Path to a TAS file to play on the core.
    #[clap(long, short)]
    tas: Option<PathBuf>,

    /// Force playing the TAS even if the ROM being run doesn't match its
    /// checksum.
    #[clap(long)]
    skip_tas_check: bool,

    /// Set the volume of the core before start (from 0 to 255). Default is muted.
    #[clap(long, default_value = "0")]
    volume: u8,

    /// Length to wait at the start before starting the simulation after a reset.
    #[clap(long)]
    wait_start: Option<String>,

    /// Number of nanoseconds to wait after the frame changed.
    #[clap(long)]
    wait_inner_frame: Option<humantime::Duration>,

    #[command(flatten)]
    pub verbose: Verbosity<clap_verbosity_flag::InfoLevel>,
}

fn main() {
    let cpu_cores = core_affinity::get_core_ids().unwrap();
    // Always use the first core available.
    let core = cpu_cores
        .get(1)
        .unwrap_or_else(|| cpu_cores.first().expect("Could not find a CPU?!"));
    core_affinity::set_for_current(*core);

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

    debug!(?opts);
    let mut fpga = mister_fpga::fpga::MisterFpga::init().unwrap();

    let mut bytes = Vec::new();
    std::fs::File::open(opts.core)
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();

    fpga.load(bytes.as_slice()).unwrap();

    fpga.osd_enable();

    let options = Config::base().into_inner();

    let mut core = MisterFpgaCore::new(fpga.clone()).expect("Could not create the core");

    core.init().unwrap();
    core.send_volume(opts.volume).unwrap();
    core.spi_mut().execute(UserIoButtonSwitch::new()).unwrap();
    core.send_rtc().unwrap();

    core.load_file(&opts.rom, None).expect("Could not load rom");

    core.end_send_file().unwrap();
    while core.poll_mounts().unwrap() {}

    fpga.osd_disable();

    // Get the video info.
    let video_info = core.video_info().unwrap();
    info!(?video_info, "Video initialized");

    if let Some(tas) = opts.tas {
        // Showtime!
        core.soft_reset();

        let port0 = *core.gamepad(0).unwrap();
        let frames = read_frames(&tas, port0).expect("Could not read TAS file.");

        let trace_is_enabled = tracing::enabled!(Level::TRACE);

        const TRACE_EVERY_N_FRAMES: usize = 600;

        let mut wait_frames = 0;
        let wait_start: Duration = if let Some(wait_start) = &opts.wait_start {
            humantime::parse_duration_ex(wait_start, |unit, value| match unit {
                "fr" | "frame" | "frames" => {
                    wait_frames = value as u32;
                    Ok(Some(0))
                }
                _ => Ok(None),
            })
            .expect("Invalid wait_start argument.")
        } else {
            Duration::from_secs(0)
        };

        let wait_inner_frame: Duration =
            opts.wait_inner_frame.map(|x| x.into()).unwrap_or_default();

        let mut frame_it = core.frame_iter();
        for _ in 0..wait_frames {
            let _ = frame_it.next();
        }

        let mut next = std::time::Instant::now();
        next += wait_start;
        while std::time::Instant::now() < next {}

        let start = std::time::Instant::now();
        let mut last = start;

        for (frame, (p0, p1)) in frames.into_iter().enumerate() {
            let _ = frame_it.next();

            let wait_to_inner_frame = std::time::Instant::now() + wait_inner_frame;
            while std::time::Instant::now() < wait_to_inner_frame {}

            if trace_is_enabled && frame != 0 && frame % TRACE_EVERY_N_FRAMES == 0 {
                let elapsed = last.elapsed();
                let per_frame = elapsed / 600;
                let per_frame_entire = start.elapsed() / frame as u32;
                let fps = 1. / per_frame.as_secs_f64();
                let fps_entire = 1. / per_frame_entire.as_secs_f64();

                trace!(
                    ?frame,
                    ?elapsed,
                    ?per_frame,
                    ?fps,
                    ?per_frame_entire,
                    ?fps_entire,
                    "Frame"
                );
                last = std::time::Instant::now();
            }

            if let Some(p0) = p0 {
                core.send_gamepad(0, p0);
            }
            if let Some(p1) = p1 {
                core.send_gamepad(1, p1);
            }
        }
    } else {
        info!("No TAS file provided, running the core indefinitely.");
        loop {}
    }
}

fn fce_button_to_mister(button: FceInputButton) -> MisterFpgaButtons {
    match button {
        FceInputButton::A => MisterFpgaButtons::A,
        FceInputButton::B => MisterFpgaButtons::B,
        FceInputButton::Select => MisterFpgaButtons::Back,
        FceInputButton::Start => MisterFpgaButtons::Start,
        FceInputButton::Up => MisterFpgaButtons::DpadUp,
        FceInputButton::Down => MisterFpgaButtons::DpadDown,
        FceInputButton::Left => MisterFpgaButtons::DpadLeft,
        FceInputButton::Right => MisterFpgaButtons::DpadRight,
    }
}

fn fce_gamepad_to_button_map(gamepad: FceInputGamepad, mut map: ButtonMap) -> ButtonMap {
    map.clear();
    for button in gamepad.buttons() {
        map.press(fce_button_to_mister(button));
    }
    map
}

fn read_frames(
    tas_file: impl AsRef<Path>,
    base_map: ButtonMap,
) -> Result<Vec<(Option<ButtonMap>, Option<ButtonMap>)>, &'static str> {
    // let base_map = ButtonMap::new();
    let tas = tas_file.as_ref();
    match tas.extension().map(|e| e.to_str().unwrap_or_default()) {
        Some("fm2") => {
            info!("Reading FM2 file: {}", tas.display());
            // Read the file and decode it.
            let file = std::fs::File::open(tas).expect("Could not open TAS file");
            let fm = fce_movie_format::FceFile::load_stream(BufReader::new(file)).unwrap();

            let frames = fm.frames().map(|f| {
                let p0 = f
                    .port0
                    .as_ref()
                    .and_then(|p| p.as_gamepad())
                    .map(|buttons| fce_gamepad_to_button_map(*buttons, base_map));
                let p1 = f
                    .port1
                    .as_ref()
                    .and_then(|p| p.as_gamepad())
                    .map(|buttons| fce_gamepad_to_button_map(*buttons, base_map));
                (p0, p1)
            });

            Ok(frames.collect())
        }
        _ => {
            error!("Unsupported TAS file format.");
            Err("Unsupported TAS file format.")
        }
    }
}

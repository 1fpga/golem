use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use clap::Parser;
use clap_verbosity_flag::Level as VerbosityLevel;
use clap_verbosity_flag::Verbosity;
use tracing::{debug, error, info, Level, trace};
use tracing_subscriber::fmt::Subscriber;

use fce_movie_format::{FceFrame, FceInputButton, FceInputGamepad};
use mister_fpga::config::Config;
use mister_fpga::core::buttons::{ButtonMap, MisterFpgaButtons};
use mister_fpga::fpga::user_io::UserIoButtonSwitch;
use tasbot::{NESGamepadState, R08File, R08Frame, R08InputButton};

/// `taser` is a simple command-line interface to the GoLEm Mister core
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

    /// Set the frame time in nanoseconds. Default will use the core's video
    /// vertical refresh time.
    #[clap(long)]
    frame_nsec: Option<u64>,

    #[command(flatten)]
    pub verbose: Verbosity<clap_verbosity_flag::InfoLevel>,
}

fn main() {
    let cores = core_affinity::get_core_ids().unwrap();
    // Always use the first core available.
    let core = cores
        .get(1)
        .unwrap_or_else(|| cores.first().expect("Could not find a CPU?!"));
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

    let mut core =
        mister_fpga::core::MisterFpgaCore::new(fpga.clone()).expect("Could not create the core");

    core.init().unwrap();
    core.init_video(&options, false).unwrap();
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
        let sleepy_time =
            // std::time::Duration::from_secs_f64(1. / 60.00);
            // std::time::Duration::from_secs_f64(1. / 60.1);
            // std::time::Duration::from_secs_f64(1. / 60.099822938442230224609375);
            // std::time::Duration::from_nanos(16_638_984);
            // std::time::Duration::from_nanos(16_638_997);
            // std::time::Duration::from_nanos(16_641_160);

// gets star

            // std::time::Duration::from_nanos(16_641_170); // gets star
            // std::time::Duration::from_nanos(16_641_165); // gets star
            // std::time::Duration::from_nanos(16_641_160);
            // std::time::Duration::from_nanos(16_641_155);
            //    std::time::Duration::from_nanos(16_641_140);

            // std::time::Duration::from_nanos(16_641_180);
            opts.frame_nsec.map(std::time::Duration::from_nanos).unwrap_or_else(|| video_info.vtime());

        //     video_info.vtime();
        let sleepy_time_ns = sleepy_time.as_nanos() as u32;
        debug!(?sleepy_time, ?sleepy_time_ns, "Showtime");

        // Showtime!
        core.soft_reset();

        let port0 = *core.gamepad(0).unwrap();
        let frames = read_frames(&tas, port0).expect("Could not read TAS file.");

        let start = std::time::Instant::now();
        let mut last = std::time::Instant::now();
        let trace_is_enabled = tracing::enabled!(Level::TRACE);
        std::thread::sleep(sleepy_time / 2);

        const TRACE_EVERY_N_FRAMES: usize = 600;

        let mut next = std::time::Instant::now();

        for (frame, (p0, p1)) in frames.into_iter().enumerate() {
            while std::time::Instant::now() < next {}
            next += sleepy_time;

            if trace_is_enabled && frame > 0 && frame % TRACE_EVERY_N_FRAMES == 0 {
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

fn r08_button_to_mister(button: R08InputButton) -> MisterFpgaButtons {
    match button {
        R08InputButton::A => MisterFpgaButtons::A,
        R08InputButton::B => MisterFpgaButtons::B,
        R08InputButton::Select => MisterFpgaButtons::Back,
        R08InputButton::Start => MisterFpgaButtons::Start,
        R08InputButton::Up => MisterFpgaButtons::DpadUp,
        R08InputButton::Down => MisterFpgaButtons::DpadDown,
        R08InputButton::Left => MisterFpgaButtons::DpadLeft,
        R08InputButton::Right => MisterFpgaButtons::DpadRight,
        R08InputButton::None => MisterFpgaButtons::NoMapping,
    }
}

fn r08_gamepad_to_button_map(gamepad: NESGamepadState, mut map: ButtonMap) -> ButtonMap {
    map.clear();
    for button in gamepad.buttons() {
        if !button.is_none() {
            map.press(r08_button_to_mister(button));
        }
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
            let file = std::fs::File::open(&tas).expect("Could not open TAS file");
            let fm = fce_movie_format::FceFile::load_stream(BufReader::new(file)).unwrap();

            let frames = std::iter::repeat(FceFrame::empty(&fm.header))
                .take(10)
                .chain(fm.frames().copied())
                .map(|f| {
                    let p0 = f
                        .port0
                        .as_ref()
                        .and_then(|p| p.as_gamepad())
                        .map(|buttons| fce_gamepad_to_button_map(*buttons, base_map.clone()));
                    let p1 = f
                        .port1
                        .as_ref()
                        .and_then(|p| p.as_gamepad())
                        .map(|buttons| fce_gamepad_to_button_map(*buttons, base_map.clone()));
                    (p0, p1)
                });

            Ok(frames.collect())
        }
        Some("r08") => {
            info!("Reading R08 file: {}", tas.display());
            // Read the file and decode it.
            let file = std::fs::File::open(&tas).expect("Could not open TAS file");
            let r08 = R08File::read(BufReader::new(file)).expect("Could not read R08 file");

            let frames: Vec<_> = std::iter::repeat(R08Frame::empty())
                .take(10)
                .chain(r08.frames.iter().copied())
                .map(|f| {
                    (
                        Some(r08_gamepad_to_button_map(f.player1, base_map.clone())),
                        Some(r08_gamepad_to_button_map(f.player2, base_map.clone())),
                    )
                })
                .collect();

            eprintln!("frames: {:?}", r08.frames.iter().take(50).collect::<Vec<_>>());
            Ok(frames)
        }
        _ => {
            error!("Unsupported TAS file format.");
            Err("Unsupported TAS file format.")
        }
    }
}

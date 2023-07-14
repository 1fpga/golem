use crate::cfg;
use std::ffi::CStr;
use tracing::info;

extern "C" {
    static mut bootcoretype: [u8; 64];
    static mut btimeout: u16;
}

#[no_mangle]
pub unsafe extern "C" fn bootcore_init(path: *const u8) {
    let path = CStr::from_ptr(path).to_str().ok();
    let path = if path == Some("") { None } else { Some(path) };
    info!("bootcore_init: path = {path:?}");

    // TODO: figure out why this is needed, and why is this here.
    let timeout = cfg::cfg_bootcore_timeout() * 10;
    cfg::cfg_set_bootcore_timeout(timeout);
    btimeout = cfg::cfg_bootcore_timeout();

    info!("bootcore_init: timeout = {timeout}");

    let bootcore = cfg::Config::bootcore();
    info!("bootcore = {bootcore:?}");

    if bootcore.is_last_core() {
        let bootcore_str = match bootcore {
            cfg::BootCoreConfig::LastCore => "lastcore",
            cfg::BootCoreConfig::ExactLastCore => "lastexactcore",
            _ => unreachable!(),
        };
        bootcoretype[..bootcore_str.len()].copy_from_slice(bootcore_str.as_bytes());
        bootcoretype[bootcore_str.len()] = 0; // Make sure we're NUL terminated.
    }

    //
    // info!("bootcore_init: cfg_bootcore = {bootcore}");
    //
    // if is_last_core && !path.is_empty() {
    //     let core = if bootcore == "lastexactcore" || isXmlName(path.as_ptr() as *const _) != 0 {
    //         Path::new(path).file_name().unwrap().to_str().unwrap()
    //     } else {
    //         core_name_of(Path::new(path)).unwrap_or("")
    //     };
    //
    //     eprintln!("Loading last core: is_last_core={is_last_core} path='{path}' core='{core}' bootcore='{bootcore}'");
    //     if core != bootcore {
    //         std::fs::write("config/lastcore.dat", core).unwrap();
    //     }
    // }
    //
    // cfg::cfg_set_bootcore("\0".as_ptr())
}

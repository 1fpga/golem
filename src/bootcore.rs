// use crate::file_io::isXmlName;
// use crate::{cfg, file_io, fpga, support};
// use std::ffi::{CStr, OsStr};
// use std::path::{Path, PathBuf};
// use std::str::FromStr;
// use tracing::info;
//
// extern "C" {
//     static mut bootcoretype: [u8; 64];
//     static mut btimeout: u16;
// }
//
// pub enum BootCoreType {
//     LastCore,
//     ExactLastCore,
//     CoreName(String),
// }
//
// impl FromStr for BootCoreType {
//     type Err = ();
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s.to_lowercase().as_str() {
//             "lastcore" => Ok(BootCoreType::LastCore),
//             "exactlastcore" => Ok(BootCoreType::ExactLastCore),
//             _ => Ok(BootCoreType::CoreName(s.into())),
//         }
//     }
// }
//
// fn is_exact_core_name_(path: &Path) -> bool {
//     match path.extension().and_then(OsStr::to_str) {
//         Some("rbf") => true,
//         Some("mra") => true,
//         Some("mgl") => true,
//         _ => false,
//     }
// }
//
// fn load_last_core() -> Option<String> {
//     std::fs::read_to_string("config/lastcore.dat").ok()
// }
//
// #[no_mangle]
// pub unsafe extern "C" fn bootcore_init(path: *const u8) {
//     let path = CStr::from_ptr(path).to_str().unwrap_or("");
//     info!("bootcore_init: path = {path}");
//
//     let root = file_io::getRootDir();
//     let root = Path::new(CStr::from_ptr(root).to_str().unwrap_or(""));
//
//     info!("bootcore_init: root = {root:?}");
//
//     btimeout = cfg::cfg_bootcore_timeout();
//
//     // TODO: figure out why this is needed, and why is this here.
//     let timeout = cfg::cfg_bootcore_timeout() * 10;
//     cfg::cfg_set_bootcore_timeout(timeout);
//
//     info!("bootcore_init: timeout = {timeout}");
//
//     let cfg_bootcore = CStr::from_ptr(cfg::cfg_bootcore()).to_str().ok();
//     let mut bootcore = cfg_bootcore
//         .map(ToString::to_string)
//         .unwrap_or("".to_string());
//     let mut is_last_core = false;
//
//     info!("bootcore_init: cfg_bootcore = {bootcore}");
//
//     // Set the boot core type (for displaying later), and read the core path
//     // if the config was set to lastcore or lastexactcore.
//     let ty = match cfg_bootcore {
//         Some(ty @ "lastcore") | Some(ty @ "lastexactcore") => {
//             is_last_core = true;
//             if let Some(core) = load_last_core() {
//                 if core != "" {
//                     eprintln!("bootcore_init.3: lastcore = {:?}", core);
//                     cfg::cfg_set_bootcore(core.as_ptr() as *const _);
//                     bootcore = core;
//                 }
//             }
//             ty
//         }
//         Some(core) => {
//             if is_exact_core_name_(Path::new(core)) {
//                 "exactcorename\0"
//             } else {
//                 "corename\0"
//             }
//         }
//         None => "corename\0",
//     };
//     bootcoretype[0..ty.len()].copy_from_slice(ty.as_bytes());
//
//     let core_path = find_core_(root, &bootcore).unwrap();
//     info!("bootcore_init: core_path = {core_path:?}");
//     if let Some(core_path) = core_path {
//         if let Some(corename) = core_path.file_name() {
//             cfg::cfg_set_bootcore(corename.to_string_lossy().as_ptr() as *const _);
//
//             if path == "" {
//                 if timeout == 0 {
//                     if corename.to_string_lossy().ends_with(".rbf") {
//                         fpga::fpga_load_rbf(
//                             core_path.to_str().unwrap().as_ptr(),
//                             std::ptr::null(),
//                             std::ptr::null(),
//                         );
//                     } else {
//                         support::arcade::xml_load(core_path.to_str().unwrap().as_ptr());
//                     }
//                 }
//
//                 cfg::cfg_set_bootcore(if bootcore == "menu.rbf" {
//                     bootcore.as_ptr()
//                 } else {
//                     "\0".as_ptr()
//                 });
//                 return;
//             }
//         }
//     }
//
//     info!("bootcore_init: path = {path} is_last_core = {is_last_core}");
//
//     if is_last_core && !path.is_empty() {
//         let core = if bootcore == "lastexactcore" || isXmlName(path.as_ptr() as *const _) != 0 {
//             Path::new(path).file_name().unwrap().to_str().unwrap()
//         } else {
//             core_name_of(Path::new(path)).unwrap_or("")
//         };
//
//         eprintln!("Loading last core: is_last_core={is_last_core} path='{path}' core='{core}' bootcore='{bootcore}'");
//         if core != bootcore {
//             std::fs::write("config/lastcore.dat", core).unwrap();
//         }
//     }
//
//     cfg::cfg_set_bootcore("\0".as_ptr())
// }

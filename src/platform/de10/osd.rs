use super::spi;
use super::user_io;
use std::ffi::{c_char, c_int, c_uchar, CStr, CString};

const DISABLE_KEYBOARD: u8 = 0x02; // disable keyboard while OSD is active
pub const OSD_INFO: u8 = 0x04; // display info
const OSD_MSG: u8 = 0x08; // display message window

pub const OSD_CMD_WRITE: u8 = 0x20; // OSD write video data command
pub const OSD_CMD_ENABLE: u8 = 0x41; // OSD enable command
pub const OSD_CMD_DISABLE: u8 = 0x40; // OSD disable command

#[derive(Debug, Clone, Copy, PartialEq, strum::Display)]
pub enum OsdArrow {
    None = 0,
    Left = 1,
    Right = 2,
    Both = 3,
}

impl From<c_int> for OsdArrow {
    fn from(value: c_int) -> Self {
        match value {
            0 => OsdArrow::None,
            1 => OsdArrow::Left,
            2 => OsdArrow::Right,
            3 => OsdArrow::Both,
            _ => panic!("Invalid arrow value"),
        }
    }
}

static mut OSD_SIZE: usize = 0;
static mut ARROW: OsdArrow = OsdArrow::None;

#[no_mangle]
pub extern "C" fn OsdSetSize(n: c_int) {
    unsafe {
        OSD_SIZE = n as usize;
    }
}

#[no_mangle]
pub extern "C" fn OsdGetSize() -> c_int {
    unsafe { OSD_SIZE as c_int }
}

#[no_mangle]
pub extern "C" fn OsdSetArrow(arrow: c_int) {
    unsafe {
        ARROW = arrow.into();
    }
}

static mut OSD_CORE_NAME: Option<CString> = None;

#[no_mangle]
pub unsafe extern "C" fn OsdCoreNameSet(name: *const c_char) {
    // Verify this is a valid string too.
    let name = CStr::from_ptr(name);
    let _ = name.to_str().unwrap();

    OSD_CORE_NAME = Some(name.into());
}

#[no_mangle]
pub extern "C" fn OsdCoreNameGet() -> *const c_char {
    unsafe {
        match &OSD_CORE_NAME {
            None => "CORE\0".as_ptr() as *const c_char, // We expect a NUL-terminated string.
            Some(x) => x.as_ptr(),
        }
    }
}

/// Update the title, and optionally arrows on the last line (the exit line)
/// of the OSD.
#[no_mangle]
pub unsafe extern "C" fn OsdSetTitle(_title: *const c_char, _arrow: c_int) {}

#[no_mangle]
pub unsafe extern "C" fn OsdWrite(
    _n: u8,
    _s: *const c_char,
    _invert: c_uchar,
    _stipple: c_uchar,
    _usebg: c_char,
    _maxinv: c_int,
    _mininv: c_int,
) {
}

/// Write a null-terminated string <s> to the OSD buffer starting at line <n>.
#[no_mangle]
pub unsafe extern "C" fn OsdWriteOffset(
    _n: u8,
    _s: *const c_char,
    _invert: c_uchar,
    _stipple: c_uchar,
    _offset: c_char,
    _leftchar: c_char,
    _usebg: c_char,
    _maxinv: c_int,
    _mininv: c_int,
) {
}

#[no_mangle]
pub extern "C" fn OsdShiftDown(_n: u8) {}

// clear OSD frame buffer
#[no_mangle]
pub extern "C" fn OsdClear() {}

#[no_mangle]
pub extern "C" fn OsdEnable(mut mode: u8) {
    unsafe {
        user_io::user_io_osd_key_enable(mode & DISABLE_KEYBOARD);
        mode &= DISABLE_KEYBOARD | OSD_MSG;
        spi::spi_osd_cmd(OSD_CMD_ENABLE | mode);
    }
}

#[no_mangle]
pub extern "C" fn InfoEnable(x: c_int, y: c_int, width: c_int, height: c_int) {
    unsafe {
        user_io::user_io_osd_key_enable(0);
        spi::spi_osd_cmd_cont(OSD_CMD_ENABLE | OSD_INFO);
        spi::spi_w(x as u16);
        spi::spi_w(y as u16);
        spi::spi_w(width as u16);
        spi::spi_w(height as u16);
        spi::DisableOsd();
    }
}

#[no_mangle]
pub extern "C" fn OsdRotation(rotate: u8) {
    unsafe {
        spi::spi_osd_cmd_cont(OSD_CMD_DISABLE);
        spi::spi_w(0);
        spi::spi_w(0);
        spi::spi_w(0);
        spi::spi_w(0);
        spi::spi_w(rotate as u16);
        spi::DisableOsd();
    }
}

/// Disable displaying of OSD.
#[no_mangle]
pub extern "C" fn OsdDisable() {
    unsafe {
        user_io::user_io_osd_key_enable(0);
        spi::spi_osd_cmd(OSD_CMD_DISABLE);
    }
}

#[no_mangle]
pub extern "C" fn OsdMenuCtl(en: c_int) {
    unsafe {
        if en != 0 {
            spi::spi_osd_cmd(OSD_CMD_WRITE | 8);
            spi::spi_osd_cmd(OSD_CMD_ENABLE);
        } else {
            spi::spi_osd_cmd(OSD_CMD_DISABLE);
        }
    }
}

#[no_mangle]
pub extern "C" fn OsdUpdate() {}

#[no_mangle]
pub unsafe extern "C" fn OSD_PrintInfo(
    _message: *const c_char,
    _width: *mut c_int,
    _height: *mut c_int,
    _frame: c_int,
) {
}

#[no_mangle]
pub unsafe extern "C" fn OsdDrawLogo(_row: c_int) {}

#[no_mangle]
pub unsafe extern "C" fn ScrollText(
    _n: c_char,
    _str: *const c_char,
    _off: c_int,
    _len: c_int,
    _max_len: c_int,
    _invert: u8,
    _idx: c_int,
) {
}

#[no_mangle]
pub unsafe extern "C" fn ScrollReset(_idx: c_int) {}

#[no_mangle]
pub unsafe extern "C" fn StarsInit() {}

#[no_mangle]
pub unsafe extern "C" fn StarsUpdate() {}

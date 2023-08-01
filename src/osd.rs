#![cfg(feature = "platform_de10")]
use crate::spi;
use crate::user_io;
use std::ffi::{c_char, c_int, c_uchar, c_ulong, CStr, CString};

// time delay after which file/dir name starts to scroll
const SCROLL_DELAY: c_ulong = 1000;
const SCROLL_DELAY2: c_ulong = 10;

const INFO_MAXW: usize = 32;
const INFO_MAXH: usize = 16;

const DISABLE_KEYBOARD: u8 = 0x02; // disable keyboard while OSD is active
pub const OSD_INFO: u8 = 0x04; // display info
const OSD_MSG: u8 = 0x08; // display message window

pub const OSD_LINE_LENGTH: usize = 256; // single line length in bytes
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

impl OsdArrow {
    pub fn is_left(&self) -> bool {
        *self == OsdArrow::Left || *self == OsdArrow::Both
    }

    pub fn is_right(&self) -> bool {
        *self == OsdArrow::Right || *self == OsdArrow::Both
    }

    pub fn without_left(&self) -> Self {
        match self {
            OsdArrow::Right | OsdArrow::Both => OsdArrow::Right,
            _ => OsdArrow::None,
        }
    }

    pub fn without_right(&self) -> Self {
        match self {
            OsdArrow::Left | OsdArrow::Both => OsdArrow::Left,
            _ => OsdArrow::None,
        }
    }
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

fn rotate_char_(i: *const c_char, o: *mut c_char) {
    for b in 0..8 {
        let mut a = 0;
        unsafe {
            for c in 0..8 {
                a <<= 1;
                a |= (*(i.add(c)) >> b) & 1;
            }
            *o.add(b) = a;
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

fn osd_start_(_line: u8) {}

unsafe fn draw_title_(_p: *const u8) {}

// write a null-terminated string <s> to the OSD buffer starting at line <n>
unsafe fn print_line_(
    _line: u8,
    _hdr: *const c_char,
    _text: *const c_char,
    _width: c_ulong,
    _offset: c_ulong,
    _invert: u8,
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

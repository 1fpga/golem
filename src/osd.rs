use core::ffi::{c_char, c_int, CStr};
use libc_print::libc_println;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OsdArrow {
    None = 0,
    Left = 1,
    Right = 2,
    Both = 3,
}

impl Display for OsdArrow {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OsdArrow::None => write!(f, "None"),
            OsdArrow::Left => write!(f, "Left"),
            OsdArrow::Right => write!(f, "Right"),
            OsdArrow::Both => write!(f, "Both"),
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

static mut OSD_CORE_NAME: Option<String> = None;

#[no_mangle]
pub extern "C" fn OsdCoreNameSet(name: *const c_char) {
    let name = unsafe { CStr::from_ptr(name) };
    let name = name.to_str().unwrap();

    // *OSD_CORE_NAME.borrow_mut() = name.to_string();
    unsafe {
        *&mut OSD_CORE_NAME = Some(name.to_string());
    }
}

#[no_mangle]
pub extern "C" fn OsdCoreNameGet() -> *const c_char {
    unsafe {
        match &OSD_CORE_NAME {
            None => "CORE".as_ptr(),
            Some(x) => x.as_ptr(),
        }
    }
}

extern "C" {
    fn OsdSetTitleLEGACY(title: *const c_char, arrow: c_int);
}

#[no_mangle]
pub extern "C" fn OsdSetTitle(title: *const c_char, arrow: c_int) {
    let title_cstr = unsafe { CStr::from_ptr(title) };
    let title_str = title_cstr.to_str().unwrap();
    OsdSetArrow(arrow);
    libc_println!("OsdSetTitle: {title_str} {}", unsafe { ARROW });

    let mut zeros = 0u32;

    unsafe {
        OsdSetTitleLEGACY(title, arrow);
    }
}

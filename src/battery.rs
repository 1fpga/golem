//! Display the pi-top battery status.
use crate::smbus;
use std::ffi::{c_int, c_short};

static mut I2C_HANDLE: Option<c_int> = None;

// Maximum numbers of tries to read a register before failing.
const MAX_COUNT: u32 = 20;

const REGISTER_POLLING_RATE_IN_USEC: u32 = 500;

#[repr(C)]
pub struct BatteryData {
    pub load_current: c_short,
    pub capacity: c_short,
    pub current: c_short,
    pub time: c_short,
    pub voltage: c_short,
    pub cell: [c_short; 4],
}

unsafe fn get_reg_(reg: u8, min: i32, max: i32) -> Option<i32> {
    if let Some(handle) = I2C_HANDLE {
        for _ in 0..MAX_COUNT {
            let value = unsafe { smbus::i2c_smbus_read_word_data(handle, reg) };
            if value != -1 && (min..max).contains(&value) {
                return Some(value);
            }

            libc::usleep(REGISTER_POLLING_RATE_IN_USEC);
        }
    }
    return None;
}

#[no_mangle]
pub unsafe extern "C" fn getBattery(quick: c_int, data: *mut BatteryData) -> c_int {
    if I2C_HANDLE.is_none() {
        unsafe {
            let handle = smbus::i2c_open(0x16, 0);
            if handle < 0 {
                println!("No battery found.");
            }
            I2C_HANDLE = Some(handle);
        }
    }

    if I2C_HANDLE == Some(-1) {
        return 0;
    }

    (*data).capacity = get_reg_(0x0D, 0, 100).unwrap_or(-1) as c_short;
    (*data).load_current = get_reg_(0x0A, -5000, 5000).unwrap_or(-1) as c_short;
    if quick == 0 {
        (*data).time = 0;
        if (*data).load_current > 0 {
            (*data).time = get_reg_(0x13, 1, 999).unwrap_or(-1) as c_short;
        } else if (*data).load_current < -1 {
            (*data).time = get_reg_(0x12, 1, 960).unwrap_or(-1) as c_short;
        }

        (*data).current = get_reg_(0x0F, 0, 5000).unwrap_or(-1) as c_short;
        (*data).voltage = get_reg_(0x09, 5000, 20000).unwrap_or(-1) as c_short;
    }

    1
}

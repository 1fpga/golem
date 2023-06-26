use std::ffi::c_ulong;

#[export_name = "rstval"]
pub static mut RSTVAL: u8 = 0;

// TODO: move this to a proper debugging module.
#[no_mangle]
pub extern "C" fn hexdump(data: *const u8, size: u16, offset: u16) {
    let mut n = 0;
    let mut size = size as usize;
    let offset = offset as usize;
    let mut ptr = data;

    while size > 0 {
        print!("{:04x}: ", n + offset);
        let b2c = size.min(16);
        for i in 0..b2c {
            print!("{:02x} ", unsafe { *ptr.add(i) });
        }
        print!("  ");

        for _ in 0..(16 - b2c.max(16)) {
            print!("   ");
        }

        for i in 0..b2c {
            let ch = unsafe { *ptr.add(i) };
            print!(
                "{}",
                if ch.is_ascii_graphic() {
                    ch as char
                } else {
                    '.'
                }
            );
        }

        println!();
        ptr = unsafe { ptr.add(b2c) };
        size -= b2c;
        n += b2c;
    }
}

#[no_mangle]
pub extern "C" fn GetTimer(offset: c_ulong) -> c_ulong {
    let mut tp = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    unsafe {
        libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut tp);
    }

    let mut res = tp.tv_sec as c_ulong;
    res *= 1000;
    res += tp.tv_nsec as c_ulong / 1000000;
    res + offset
}

#[no_mangle]
pub extern "C" fn CheckTimer(time: c_ulong) -> c_ulong {
    if time == 0 || GetTimer(0) >= time {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn WaitTimer(t: c_ulong) {
    let time = GetTimer(t);
    while CheckTimer(time) == 0 {}
}

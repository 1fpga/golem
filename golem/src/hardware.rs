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

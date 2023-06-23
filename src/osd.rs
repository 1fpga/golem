use crate::charrom::CHAR_FONT;
use crate::spi::DisableOsd;
use crate::{charrom, hardware, spi, user_io};
use std::ffi::{c_char, c_int, c_uchar, c_ulong, CStr, CString};

// time delay after which file/dir name starts to scroll
const SCROLL_DELAY: c_ulong = 1000;
const SCROLL_DELAY2: c_ulong = 10;

const INFO_MAXW: usize = 32;
const INFO_MAXH: usize = 16;

const DISABLE_KEYBOARD: u8 = 0x02; // disable keyboard while OSD is active
const OSD_INFO: u8 = 0x04; // display info
const OSD_MSG: u8 = 0x08; // display message window

const OSD_LINE_LENGTH: usize = 256; // single line length in bytes
const OSD_CMD_WRITE: u8 = 0x20; // OSD write video data command
const OSD_CMD_ENABLE: u8 = 0x41; // OSD enable command
const OSD_CMD_DISABLE: u8 = 0x40; // OSD disable command

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

#[export_name = "titlebuffer"]
pub static mut TITLE_BUFFER: [u8; 256] = [0; 256];

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
pub extern "C" fn OsdCoreNameSet(name: *const c_char) {
    // Verify this is a valid string too.
    let name = unsafe { CStr::from_ptr(name) };
    let _ = name.to_str().unwrap();

    unsafe {
        *&mut OSD_CORE_NAME = Some(name.into());
    }
}

#[no_mangle]
pub extern "C" fn OsdCoreNameGet() -> *const c_char {
    unsafe {
        match &OSD_CORE_NAME {
            None => "CORE\0".as_ptr(), // We expect a NUL-terminated string.
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

#[no_mangle]
pub unsafe extern "C" fn OsdSetTitle(title: *const c_char, arrow: c_int) {
    let title_cstr = unsafe { CStr::from_ptr(title) };
    let title_str = title_cstr.to_str().unwrap();
    OsdSetArrow(arrow);

    // Reset the title buffer.
    TITLE_BUFFER.fill(0);

    // Remove spaces up to 5 pixels apart, making a variable-width font.
    let mut zeroes = 0;
    let mut outp = 0;

    let osd_size = OsdGetSize() as usize * 8;
    let buffer_len = unsafe { TITLE_BUFFER.len() };
    for c in title_str.chars() {
        if outp >= osd_size - 8 {
            break;
        }

        for j in 0..8 {
            let nc = charrom::CHAR_FONT[c as usize][j];
            if nc != 0 {
                zeroes = 0;
                TITLE_BUFFER[outp] = nc;
                outp += 1;
            } else if zeroes == 0 || (c == ' ' && zeroes < 5) {
                TITLE_BUFFER[outp] = 0;
                zeroes += 1;
                outp += 1;
            }

            if outp > buffer_len {
                break;
            }
        }
    }

    // Center the title.
    let center_offset = (osd_size - 1 - outp) / 2;
    TITLE_BUFFER.copy_within(0..outp, center_offset);
    TITLE_BUFFER[..center_offset].fill(0);

    // Rotate the characters one by one.
    for i in (0..osd_size).step_by(8) {
        let mut tmp = [0u8; 8];
        rotate_char_(TITLE_BUFFER.as_ptr().add(i), tmp.as_mut_ptr());
        for c in 0..8 {
            TITLE_BUFFER[i + c] = tmp[c];
        }
    }
}

static mut STAR_FRAME_BUFFER: [u8; 16 * 256] = [0; 16 * 256];
static mut OSD_BUFFER: [u8; 256 * 32] = [0; 256 * 32];
static mut OSD_BUFFER_POS: usize = 0;
static mut OSD_SET: c_int = 0;

#[no_mangle]
pub extern "C" fn OsdWrite(
    n: u8,
    s: *const c_char,
    invert: c_uchar,
    stipple: c_uchar,
    usebg: c_char,
    maxinv: c_int,
    mininv: c_int,
) {
    unsafe {
        OsdWriteOffset(n, s, invert, stipple, 0, 0, usebg, maxinv, mininv);
    }
}

fn osd_start_(line: u8) {
    let line = line & 0x1F;
    unsafe {
        OSD_SET |= 1 << line;
        OSD_BUFFER_POS = (line as usize) * 256;
    }
}

unsafe fn draw_title_(mut p: *const u8) {
    // left white border
    OSD_BUFFER[OSD_BUFFER_POS] = 0xFF;
    OSD_BUFFER_POS += 1;
    OSD_BUFFER[OSD_BUFFER_POS] = 0xFF;
    OSD_BUFFER_POS += 1;
    OSD_BUFFER[OSD_BUFFER_POS] = 0xFF;
    OSD_BUFFER_POS += 1;

    for _ in 0..8 {
        // Double the size of the characters for the title.
        OSD_BUFFER[OSD_BUFFER_POS] = 255 ^ *p;
        OSD_BUFFER[OSD_BUFFER_POS + 1] = 255 ^ *p;
        OSD_BUFFER_POS += 2;
        p = p.add(1);
    }

    // right white border
    OSD_BUFFER[OSD_BUFFER_POS] = 0xff;
    OSD_BUFFER_POS += 1;

    // blue gap
    OSD_BUFFER[OSD_BUFFER_POS] = 0;
    OSD_BUFFER_POS += 1;
    OSD_BUFFER[OSD_BUFFER_POS] = 0;
    OSD_BUFFER_POS += 1;
}

// write a null-terminated string <s> to the OSD buffer starting at line <n>
unsafe fn print_line_(
    line: u8,
    mut hdr: *const c_char,
    mut text: *const c_char,
    mut width: c_ulong,
    offset: c_ulong,
    invert: u8,
) {
    // line : OSD line number (0-7)
    // text : pointer to null-terminated string
    // start : start position (in pixels)
    // width : printed text length in pixels
    // offset : scroll offset in pixels counting from the start of the string (0-7)
    // invert : invertion flag

    let invert: u8 = if invert != 0 { 0xFF } else { 0x00 };
    // select buffer and line to write to
    osd_start_(line);
    draw_title_(
        TITLE_BUFFER
            .as_ptr()
            .add((OsdGetSize() as usize - 1 - line as usize) * 8),
    );

    while *hdr != 0 {
        width -= 8;
        let mut p = CHAR_FONT[*hdr as usize].as_ptr();
        hdr = hdr.add(1);
        for _ in 0..8 {
            OSD_BUFFER[OSD_BUFFER_POS] = invert ^ *p;
            p = p.add(1);
            OSD_BUFFER_POS += 1;
        }
    }

    if offset != 0 {
        width -= 8 - offset;
        let mut p = CHAR_FONT[*text as usize].as_ptr().add(offset as usize);
        text = text.add(1);
        for _ in offset..8 {
            OSD_BUFFER[OSD_BUFFER_POS] = invert ^ *p;
            p = p.add(1);
            OSD_BUFFER_POS += 1;
        }
    }

    while width > 8 {
        let mut p = CHAR_FONT[*text as usize].as_ptr();
        text = text.add(1);
        for _ in 0..8 {
            OSD_BUFFER[OSD_BUFFER_POS] = invert ^ *p;
            p = p.add(1);
            OSD_BUFFER_POS += 1;
        }
        width -= 8;
    }
    if width > 0 {
        let mut p = CHAR_FONT[*text as usize].as_ptr();
        for _ in 0..width {
            OSD_BUFFER[OSD_BUFFER_POS] = invert ^ *p;
            p = p.add(1);
            OSD_BUFFER_POS += 1;
        }
    }
}

/// Write a null-terminated string <s> to the OSD buffer starting at line <n>.
#[no_mangle]
pub unsafe extern "C" fn OsdWriteOffset(
    mut n: u8,
    mut s: *const c_char,
    invert: c_uchar,
    mut stipple: c_uchar,
    offset: c_char,
    mut leftchar: c_char,
    usebg: c_char,
    maxinv: c_int,
    mininv: c_int,
) {
    let invert = invert != 0;

    let mut i = 0;
    let mut p: *const c_uchar;

    let mut stipple_mask: u8 = 0xFF;
    let mut line_limit = OSD_LINE_LENGTH;
    let arrow = ARROW;
    let mut arrow_mask = arrow;

    if n as c_int == OsdGetSize() - 1 && arrow.is_right() {
        line_limit -= 22;
    }

    if n != 0 && (n as c_int) < OsdGetSize() - 1 {
        leftchar = 0;
    }

    if stipple != 0 {
        stipple_mask = 0x55;
        stipple = 0xff;
    } else {
        stipple = 0;
    }

    osd_start_(n);
    let mut xormask = 0u8;
    let mut xorchar = 0u8;

    // Send all characters in string to OSD.
    loop {
        if invert && ((i / 8) >= mininv) {
            xormask = 0xFF;
        }
        if invert && ((i / 8) >= maxinv) {
            xormask = 0;
        }

        if i == 0 && (n as c_int) < OsdGetSize() {
            let mut tmp = [0u8; 8];
            if leftchar != 0 {
                let mut tmp2 = unsafe { CHAR_FONT[leftchar as usize].clone() };
                rotate_char_(tmp2.as_mut_ptr(), tmp.as_mut_ptr());
                p = tmp.as_ptr();
            } else {
                p = TITLE_BUFFER
                    .as_ptr()
                    .add(((OsdGetSize() - 1 - (n as i32)) * 8) as usize);
            }

            draw_title_(p);
            i += 22; // Twenty two of what?
        } else if n as c_int == OsdGetSize() - 1 && arrow_mask.is_left() {
            for _ in 0..3 {
                OSD_BUFFER[OSD_BUFFER_POS] = xormask;
                OSD_BUFFER_POS += 1;
            }
            p = CHAR_FONT[0x10].as_ptr();
            for _ in 0..8 {
                OSD_BUFFER[OSD_BUFFER_POS] = (*p << offset) ^ xormask;
                OSD_BUFFER_POS += 1;
                p = p.add(1);
            }
            p = CHAR_FONT[0x14].as_ptr();
            for _ in 0..8 {
                OSD_BUFFER[OSD_BUFFER_POS] = (*p << offset) ^ xormask;
                OSD_BUFFER_POS += 1;
                p = p.add(1);
            }
            for _ in 0..5 {
                OSD_BUFFER[OSD_BUFFER_POS] = xormask;
                OSD_BUFFER_POS += 1;
            }

            i += 24;
            arrow_mask = arrow_mask.without_left();
            for _ in 0..3 {
                if *s == 0 {
                    break;
                }
                s = s.add(1);
            }
        } else {
            let b = unsafe { *s };
            s = s.add(1);
            if b == 0 {
                break;
            }

            if b == 0x0B {
                stipple_mask ^= 0xAA;
                stipple ^= 0xFF;
            } else if b == 0x0C {
                xorchar ^= 0xFF;
            } else if b == 0x0D || b == 0x0A {
                // cariage return / linefeed, go to next line increment line counter.
                n += 1;
                if n as usize >= line_limit {
                    n = 0;
                }
                osd_start_(n);
            } else if (i as usize) < (line_limit - 8) {
                // Normal character.
                p = CHAR_FONT[b as usize].as_ptr();
                for c in 0..8 {
                    let bg = if usebg != 0 {
                        STAR_FRAME_BUFFER[(n as usize) * 256 + (i as usize) + c - 22]
                    } else {
                        0
                    };
                    OSD_BUFFER[OSD_BUFFER_POS] =
                        (((*p << offset) & stipple_mask) ^ xorchar ^ xormask) | bg;
                    OSD_BUFFER_POS += 1;
                    p = p.add(1);
                    stipple_mask ^= stipple;
                }
                i += 8;
            }
        }
    }

    for i in (i as usize)..line_limit {
        let bg = if usebg != 0 {
            STAR_FRAME_BUFFER[(n as usize) * 256 + i - 22]
        } else {
            0
        };

        OSD_BUFFER[OSD_BUFFER_POS] = xormask | bg;
        OSD_BUFFER_POS += 1;
    }

    if n == (OsdGetSize() as u8 - 1) && arrow_mask.is_right() {
        for _ in 0..3 {
            OSD_BUFFER[OSD_BUFFER_POS] = xormask;
            OSD_BUFFER_POS += 1;
        }
        p = CHAR_FONT[0x15].as_ptr();
        for _ in 0..8 {
            OSD_BUFFER[OSD_BUFFER_POS] = *p ^ xormask;
            OSD_BUFFER_POS += 1;
            p = p.add(1);
        }
        p = CHAR_FONT[0x11].as_ptr();
        for _ in 0..8 {
            OSD_BUFFER[OSD_BUFFER_POS] = *p ^ xormask;
            OSD_BUFFER_POS += 1;
            p = p.add(1);
        }
        for _ in 0..5 {
            OSD_BUFFER[OSD_BUFFER_POS] = xormask;
            OSD_BUFFER_POS += 1;
        }
    }
}

#[no_mangle]
pub extern "C" fn OsdShiftDown(n: u8) {
    osd_start_(n);

    unsafe {
        OSD_BUFFER_POS += 22;
        for _ in 22..256 {
            OSD_BUFFER[OSD_BUFFER_POS] <<= 1;
            OSD_BUFFER_POS += 1;
        }
    }
}

// clear OSD frame buffer
#[no_mangle]
pub extern "C" fn OsdClear() {
    unsafe {
        OSD_SET = -1;
        OSD_BUFFER.fill(0);
    }
}

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
        DisableOsd();
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
pub extern "C" fn OsdUpdate() {
    extern "C" {
        fn mcd_poll();
        fn neocd_poll();
        fn pcecd_poll();
        fn saturn_poll();
    }

    unsafe {
        let n = if user_io::is_menu() != 0 {
            19
        } else {
            OsdGetSize()
        };

        for i in 0..n {
            if (OSD_SET & (1 << i)) != 0 {
                spi::spi_osd_cmd_cont(OSD_CMD_WRITE | (i as u8));
                spi::spi_write(OSD_BUFFER.as_ptr().add((i as usize) * 256), 256, 0);
                DisableOsd();

                if user_io::is_megacd() != 0 {
                    mcd_poll();
                }
                if user_io::is_pce() != 0 {
                    pcecd_poll();
                }
                if user_io::is_saturn() != 0 {
                    saturn_poll();
                }
                if user_io::is_neogeo_cd() != 0 {
                    neocd_poll();
                }
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn OSD_PrintInfo(
    message: *const c_char,
    width: *mut c_int,
    height: *mut c_int,
    frame: c_int,
) {
    unsafe {
        let message = CStr::from_ptr(message);
        let mut str = [' ' as u8; INFO_MAXH * INFO_MAXW];

        // calc height/width if none provided. Add frame to calculated size.
        // no frame will be added if width and height are provided.
        let calc = (*width != 0) || (*height != 0) || (frame != 0);

        let mut maxw = 0;
        let mut x = if calc { 1 } else { 0 };
        let mut y = if calc { 1 } else { 0 };

        for c in message.to_bytes() {
            match c {
                0x0D => {}
                0x0A => {
                    x = if calc { 1 } else { 0 };
                    y += 1;
                }
                c => {
                    if x < INFO_MAXW && y < INFO_MAXH {
                        str[(y * INFO_MAXW + x) as usize] = *c;
                    }
                }
            }
            x += 1;
            if x > maxw {
                maxw = x;
            }
        }

        let w: usize = (if !calc {
            (*width + 2) as usize
        } else {
            maxw + 1
        })
        .min(INFO_MAXW);
        *width = w as c_int;

        let h: usize = (if !calc { (*height + 2) as usize } else { y + 2 }).min(INFO_MAXH);
        *height = h as c_int;

        if frame != 0 {
            let frame: u8 = (frame as u8 - 1) * 6;
            for x in 1..(w - 1) {
                str[(0 * INFO_MAXW) + x] = 0x81 + frame;
                str[((h - 1) * INFO_MAXW) + x] = 0x81 + frame;
            }
            for y in 1..(h - 1) {
                str[(y * INFO_MAXW) + 0] = 0x83 + frame;
                str[(y * INFO_MAXW) + (w - 1)] = 0x83 + frame;
            }
            str[0] = 0x80 + frame;
            str[w - 1] = 0x82 + frame;
            str[(h - 1) * INFO_MAXW] = 0x85 + frame;
            str[((h - 1) * INFO_MAXW) + w - 1] = 0x84 + frame;
        }

        for y in 0..h {
            osd_start_(y as u8);

            for x in 0..w {
                let mut p = CHAR_FONT[str[(y * INFO_MAXW) + x] as usize].as_ptr();
                for _ in 0..8 {
                    OSD_BUFFER[OSD_BUFFER_POS] = *p;
                    OSD_BUFFER_POS += 1;
                    p = p.add(1);
                }
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn OsdDrawLogo(row: c_int) {
    let logo_data = include_bytes!("../assets/logo_about.dat");

    osd_start_(row as u8);

    let mut bt: u8 = 0;
    let mut lp = if row >= 10 {
        std::ptr::null()
    } else {
        logo_data.as_ptr().add(row as usize * 227)
    };
    let mut bytes = 227;
    let mut bg = STAR_FRAME_BUFFER.as_ptr().add(row as usize * 256);

    let mut i = 0;
    while i < OSD_LINE_LENGTH {
        if i == 0 {
            draw_title_(
                TITLE_BUFFER
                    .as_ptr()
                    .add((OsdGetSize() - 1 - row) as usize * 8),
            );
            i += 22;
        }

        if lp != std::ptr::null() && bytes != 0 {
            bt = *lp;
            lp = lp.add(1);
            bytes -= 1;
        }

        OSD_BUFFER[OSD_BUFFER_POS] = bt | *bg;
        bg = bg.add(1);
        OSD_BUFFER_POS += 1;
        i += 1;
    }
}

static mut SCROLL_OFFSET: [usize; 2] = [0; 2];
static mut SCROLL_TIMER: [usize; 2] = [0; 2];

#[no_mangle]
pub unsafe extern "C" fn ScrollText(
    n: c_char,
    str: *const c_char,
    off: c_int,
    len: c_int,
    max_len: c_int,
    invert: u8,
    idx: c_int,
) {
    let idx = idx as usize;
    let mut len = len as usize;
    let off = off as usize;
    let max_len = if max_len != 0 { max_len as usize } else { 30 };

    const BLANKSPACE: usize = 10;
    let mut hdr = [0u8; 40];

    let mut str = CStr::from_ptr(str).to_bytes_with_nul();

    if str.is_empty() || hardware::CheckTimer(idx as c_ulong) == 0 {
        return;
    }

    if off != 0 {
        hdr[0..off].copy_from_slice(&str[0..off]);
        str = &str[off..];
        if len > off {
            len -= off;
        }
    }

    SCROLL_TIMER[idx] = hardware::GetTimer(SCROLL_DELAY2) as usize;
    SCROLL_OFFSET[idx] += 1;

    let mut s = [b' '; 40];

    // get name length
    if len <= 0 {
        len = str.len();
    }

    // scroll name if longer than display size
    if (off + 2 + len) > max_len {
        // reset scroll position if it exceeds predefined maximum
        if SCROLL_OFFSET[idx] >= ((len + BLANKSPACE) << 3) {
            SCROLL_OFFSET[idx] = 0;
        }
        // get new starting character of the name (SCROLL_OFFSET is no longer in 2 pixel unit)
        let offset = SCROLL_OFFSET[idx] >> 3;

        // remaining number of characters in the name
        len -= offset;
        if len > max_len {
            len = max_len
        }

        // copy name substring
        if len > 0 && offset < str.len() {
            s[0..len].copy_from_slice(&str[offset..offset + len]);
        }

        // file name substring and blank space is shorter than display line size
        if len < (max_len - BLANKSPACE) {
            let overflow = max_len - BLANKSPACE - len;
            s[len + BLANKSPACE..len + BLANKSPACE + overflow].copy_from_slice(&str[0..overflow]);
        }

        print_line_(
            n,
            hdr.as_ptr(),
            s.as_ptr(),
            ((max_len - 1) << 3) as c_ulong,
            (SCROLL_OFFSET[idx] & 0x7) as c_ulong,
            invert,
        ); // OSD print function with pixel precision
    }
}

#[no_mangle]
pub unsafe extern "C" fn ScrollReset(idx: c_int) {
    let idx = idx as usize;

    // set timer to start name scrolling after predefined time delay
    SCROLL_TIMER[idx] = hardware::GetTimer(SCROLL_DELAY) as usize;
    // start scrolling from the start
    SCROLL_OFFSET[idx] = 0;
}

struct Star {
    pub pos: (i32, i32),
    pub dir: (i32, i32),
}

impl Star {
    /// Create a star for the about page. They travel left in various small
    /// deltas.
    pub fn new_about(maxx: i32, maxy: i32) -> Self {
        let x = unsafe { libc::rand() % maxx };
        let y = unsafe { libc::rand() % maxy };
        let dx = unsafe { libc::rand() % 8 };

        Self {
            pos: (x << 4, y << 4),
            dir: (-dx - 3, 0),
        }
    }

    pub fn reset_to_right(&mut self, x: i32, maxy: i32) {
        let y = unsafe { libc::rand() % maxy };
        let dx = unsafe { libc::rand() % 8 };
        self.pos = (x << 4, y << 4);
        self.dir = (-dx - 3, 0);
    }

    pub fn update(&mut self) {
        self.pos.0 += self.dir.0;
        self.pos.1 += self.dir.1;
    }

    pub fn is_outside(&self, min: (i32, i32), max: (i32, i32)) -> bool {
        self.pos.0 < min.0 || self.pos.0 > max.0 || self.pos.1 < min.1 || self.pos.1 > max.1
    }
}

static mut STARS: Option<Box<[Star]>> = None;

#[no_mangle]
pub unsafe extern "C" fn StarsInit() {
    libc::srand(libc::time(std::ptr::null_mut()) as u32);
    let mut stars = Vec::with_capacity(64);
    for _ in 0..64 {
        stars.push(Star::new_about(228, 128));
    }
    STARS = Some(stars.into_boxed_slice());
}

#[no_mangle]
pub unsafe extern "C" fn StarsUpdate() {
    STAR_FRAME_BUFFER.fill(0);

    for star in STARS.as_mut().unwrap().iter_mut() {
        star.update();

        if star.is_outside((0, 0), (228 << 4, 128 << 4)) {
            star.reset_to_right(228, 128);
        }

        let x = star.pos.0 as usize >> 4;
        let y = star.pos.1 as usize >> 4;
        // 	framebuffer[y / 8][x] |= (1 << (y & 7));
        STAR_FRAME_BUFFER[(y / 8) * 256 + x] |= 1 << (y & 7);
    }
    OSD_SET -= 1;
}

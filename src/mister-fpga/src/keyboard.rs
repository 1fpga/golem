const LCTRL: u32 = 0x000100;
const LSHIFT: u32 = 0x000200;
const LALT: u32 = 0x000400;
const LGUI: u32 = 0x000800;
const RCTRL: u32 = 0x001000;
const RSHIFT: u32 = 0x002000;
const RALT: u32 = 0x004000;
const RGUI: u32 = 0x008000;
const MODMASK: u32 = 0x00FF00;

const CAPS_TOGGLE: u32 = 0x040000;
// caps lock toggle behaviour
const EXT: u32 = 0x080000;
const EMU_SWITCH_1: u32 = 0x100000;
const EMU_SWITCH_2: u32 = 0x200000;

const UPSTROKE: u32 = 0x400000;

/// LUT from SDL3 scan code to PS/2.
#[rustfmt::skip]
const SDL3_TO_PS2: array_map::ArrayMap<u8, Option<Ps2Scancode>, 256> = array_map::ArrayMap::new([
    None,                          // SDL_SCANCODE_UNKNOWN = 0,
    None,                          // Unknown / Reserved
    None,                          // Unknown / Reserved
    None,                          // Unknown / Reserved
    Some(Ps2Scancode::A),          // SDL_SCANCODE_A = 4,
    Some(Ps2Scancode::B),          // SDL_SCANCODE_B = 5,
    Some(Ps2Scancode::C),          // SDL_SCANCODE_C = 6,
    Some(Ps2Scancode::D),          // SDL_SCANCODE_D = 7,
    Some(Ps2Scancode::E),          // SDL_SCANCODE_E = 8,
    Some(Ps2Scancode::F),          // SDL_SCANCODE_F = 9,
    Some(Ps2Scancode::G),          // SDL_SCANCODE_G = 10,
    Some(Ps2Scancode::H),          // SDL_SCANCODE_H = 11,
    Some(Ps2Scancode::I),          // SDL_SCANCODE_I = 12,
    Some(Ps2Scancode::J),          // SDL_SCANCODE_J = 13,
    Some(Ps2Scancode::K),          // SDL_SCANCODE_K = 14,
    Some(Ps2Scancode::L),          // SDL_SCANCODE_L = 15,
    Some(Ps2Scancode::M),          // SDL_SCANCODE_M = 16,
    Some(Ps2Scancode::N),          // SDL_SCANCODE_N = 17,
    Some(Ps2Scancode::O),          // SDL_SCANCODE_O = 18,
    Some(Ps2Scancode::P),          // SDL_SCANCODE_P = 19,
    Some(Ps2Scancode::Q),          // SDL_SCANCODE_Q = 20,
    Some(Ps2Scancode::R),          // SDL_SCANCODE_R = 21,
    Some(Ps2Scancode::S),          // SDL_SCANCODE_S = 22,
    Some(Ps2Scancode::T),          // SDL_SCANCODE_T = 23,
    Some(Ps2Scancode::U),          // SDL_SCANCODE_U = 24,
    Some(Ps2Scancode::V),          // SDL_SCANCODE_V = 25,
    Some(Ps2Scancode::W),          // SDL_SCANCODE_W = 26,
    Some(Ps2Scancode::X),          // SDL_SCANCODE_X = 27,
    Some(Ps2Scancode::Y),          // SDL_SCANCODE_Y = 28,
    Some(Ps2Scancode::Z),          // SDL_SCANCODE_Z = 29,
    Some(Ps2Scancode::Key1),       // SDL_SCANCODE_1 = 30,
    Some(Ps2Scancode::Key2),       // SDL_SCANCODE_2 = 31,
    Some(Ps2Scancode::Key3),       // SDL_SCANCODE_3 = 32,
    Some(Ps2Scancode::Key4),       // SDL_SCANCODE_4 = 33,
    Some(Ps2Scancode::Key5),       // SDL_SCANCODE_5 = 34,
    Some(Ps2Scancode::Key6),       // SDL_SCANCODE_6 = 35,
    Some(Ps2Scancode::Key7),       // SDL_SCANCODE_7 = 36,
    Some(Ps2Scancode::Key8),       // SDL_SCANCODE_8 = 37,
    Some(Ps2Scancode::Key9),       // SDL_SCANCODE_9 = 38,
    Some(Ps2Scancode::Key0),       // SDL_SCANCODE_0 = 39,
    Some(Ps2Scancode::Enter),      // SDL_SCANCODE_RETURN = 40,
    Some(Ps2Scancode::Esc),        // SDL_SCANCODE_ESCAPE = 41,
    Some(Ps2Scancode::Backspace),  // SDL_SCANCODE_BACKSPACE = 42,
    Some(Ps2Scancode::Tab),        // SDL_SCANCODE_TAB = 43,
    Some(Ps2Scancode::Space),      // SDL_SCANCODE_SPACE = 44,
    Some(Ps2Scancode::Minus),      // SDL_SCANCODE_MINUS = 45,
    Some(Ps2Scancode::Equal),      // SDL_SCANCODE_EQUALS = 46,
    Some(Ps2Scancode::LeftBrace),  // SDL_SCANCODE_LEFTBRACKET = 47,
    Some(Ps2Scancode::RightBrace), // SDL_SCANCODE_RIGHTBRACKET = 48,
    Some(Ps2Scancode::Backslash),  // SDL_SCANCODE_BACKSLASH = 49,
    None,                          // SDL_SCANCODE_NONUSHASH = 50,
    Some(Ps2Scancode::SemiColon),  // SDL_SCANCODE_SEMICOLON = 51,
    Some(Ps2Scancode::Apostrophe), // SDL_SCANCODE_APOSTROPHE = 52,
    Some(Ps2Scancode::Grave),      // SDL_SCANCODE_GRAVE = 53,
    Some(Ps2Scancode::Comma),      // SDL_SCANCODE_COMMA = 54,
    Some(Ps2Scancode::Dot),        // SDL_SCANCODE_PERIOD = 55,
    Some(Ps2Scancode::Slash),      // SDL_SCANCODE_SLASH = 56,
    Some(Ps2Scancode::CapsLock),   // SDL_SCANCODE_CAPSLOCK = 57,
    Some(Ps2Scancode::F1),         // SDL_SCANCODE_F1 = 58,
    Some(Ps2Scancode::F2),         // SDL_SCANCODE_F2 = 59,
    Some(Ps2Scancode::F3),         // SDL_SCANCODE_F3 = 60,
    Some(Ps2Scancode::F4),         // SDL_SCANCODE_F4 = 61,
    Some(Ps2Scancode::F5),         // SDL_SCANCODE_F5 = 62,
    Some(Ps2Scancode::F6),         // SDL_SCANCODE_F6 = 63,
    Some(Ps2Scancode::F7),         // SDL_SCANCODE_F7 = 64,
    Some(Ps2Scancode::F8),         // SDL_SCANCODE_F8 = 65,
    Some(Ps2Scancode::F9),         // SDL_SCANCODE_F9 = 66,
    Some(Ps2Scancode::F10),        // SDL_SCANCODE_F10 = 67,
    Some(Ps2Scancode::F11),        // SDL_SCANCODE_F11 = 68,
    Some(Ps2Scancode::F12),        // SDL_SCANCODE_F12 = 69,
    Some(Ps2Scancode::SysReq),     // SDL_SCANCODE_PRINTSCREEN = 70,
    Some(Ps2Scancode::ScrollLock), // SDL_SCANCODE_SCROLLLOCK = 71,
    Some(Ps2Scancode::Pause),      // SDL_SCANCODE_PAUSE = 72,
    Some(Ps2Scancode::Insert),     // SDL_SCANCODE_INSERT = 73,
    Some(Ps2Scancode::Home),       // SDL_SCANCODE_HOME = 74,
    Some(Ps2Scancode::PageUp),     // SDL_SCANCODE_PAGEUP = 75,
    Some(Ps2Scancode::Delete),     // SDL_SCANCODE_DELETE = 76,
    Some(Ps2Scancode::End),        // SDL_SCANCODE_END = 77,
    Some(Ps2Scancode::PageDown),   // SDL_SCANCODE_PAGEDOWN = 78,
    Some(Ps2Scancode::Right),      // SDL_SCANCODE_RIGHT = 79,
    Some(Ps2Scancode::Left),       // SDL_SCANCODE_LEFT = 80,
    Some(Ps2Scancode::Down),       // SDL_SCANCODE_DOWN = 81,
    Some(Ps2Scancode::Up),         // SDL_SCANCODE_UP = 82,
    Some(Ps2Scancode::NumLock),    // SDL_SCANCODE_NUMLOCKCLEAR = 83,
    Some(Ps2Scancode::KpSlash),    // SDL_SCANCODE_KP_DIVIDE = 84,
    Some(Ps2Scancode::KpAsterisk), // SDL_SCANCODE_KP_MULTIPLY = 85,
    Some(Ps2Scancode::KpMinus),    // SDL_SCANCODE_KP_MINUS = 86,
    Some(Ps2Scancode::KpPlus),     // SDL_SCANCODE_KP_PLUS = 87,
    Some(Ps2Scancode::KpEnter),    // SDL_SCANCODE_KP_ENTER = 88,
    Some(Ps2Scancode::Kp1),        // SDL_SCANCODE_KP_1 = 89,
    Some(Ps2Scancode::Kp2),        // SDL_SCANCODE_KP_2 = 90,
    Some(Ps2Scancode::Kp3),        // SDL_SCANCODE_KP_3 = 91,
    Some(Ps2Scancode::Kp4),        // SDL_SCANCODE_KP_4 = 92,
    Some(Ps2Scancode::Kp5),        // SDL_SCANCODE_KP_5 = 93,
    Some(Ps2Scancode::Kp6),        // SDL_SCANCODE_KP_6 = 94,
    Some(Ps2Scancode::Kp7),        // SDL_SCANCODE_KP_7 = 95,
    Some(Ps2Scancode::Kp8),        // SDL_SCANCODE_KP_8 = 96,
    Some(Ps2Scancode::Kp9),        // SDL_SCANCODE_KP_9 = 97,
    Some(Ps2Scancode::Kp0),        // SDL_SCANCODE_KP_0 = 98,
    Some(Ps2Scancode::KpDot),      // SDL_SCANCODE_KP_PERIOD = 99,
    Some(Ps2Scancode::Backslash),  // SDL_SCANCODE_NONUSBACKSLASH = 100,
    None,                          // SDL_SCANCODE_APPLICATION = 101,
    None,                          // SDL_SCANCODE_POWER = 102,
    None,                          // SDL_SCANCODE_KP_EQUALS = 103,
    None,                          // SDL_SCANCODE_F13 = 104,
    None,                          // SDL_SCANCODE_F14 = 105,
    None,                          // SDL_SCANCODE_F15 = 106,
    None,                          // SDL_SCANCODE_F16 = 107,
    Some(Ps2Scancode::F17),        // SDL_SCANCODE_F17 = 108,
    Some(Ps2Scancode::F18),        // SDL_SCANCODE_F18 = 109,
    Some(Ps2Scancode::F19),        // SDL_SCANCODE_F19 = 110,
    Some(Ps2Scancode::F20),        // SDL_SCANCODE_F20 = 111,
    None,                          // SDL_SCANCODE_F21 = 112,
    None,                          // SDL_SCANCODE_F22 = 113,
    None,                          // SDL_SCANCODE_F23 = 114,
    None,                          // SDL_SCANCODE_F24 = 115,
    None,                          // SDL_SCANCODE_EXECUTE = 116,
    None,                          // SDL_SCANCODE_HELP = 117,
    None,                          // SDL_SCANCODE_MENU = 118,
    None,                          // SDL_SCANCODE_SELECT = 119,
    None,                          // SDL_SCANCODE_STOP = 120,
    None,                          // SDL_SCANCODE_AGAIN = 121,
    None,                          // SDL_SCANCODE_UNDO = 122,
    None,                          // SDL_SCANCODE_CUT = 123,
    None,                          // SDL_SCANCODE_COPY = 124,
    None,                          // SDL_SCANCODE_PASTE = 125,
    None,                          // SDL_SCANCODE_FIND = 126,
    None,                          // SDL_SCANCODE_MUTE = 127,
    None,                          // SDL_SCANCODE_VOLUMEUP = 128,
    None,                          // SDL_SCANCODE_VOLUMEDOWN = 129,
    None,                          // NOT MAPPED IN SDL3 130,
    None,                          // NOT MAPPED IN SDL3 131,
    None,                          // NOT MAPPED IN SDL3 132,
    None,                          // SDL_SCANCODE_KP_COMMA = 133,
    None,                          // SDL_SCANCODE_KP_EQUALSAS400 = 134,
    None,                          // SDL_SCANCODE_INTERNATIONAL1 = 135,
    None,                          // SDL_SCANCODE_INTERNATIONAL2 = 136,
    None,                          // SDL_SCANCODE_INTERNATIONAL3 = 137,
    None,                          // SDL_SCANCODE_INTERNATIONAL4 = 138,
    None,                          // SDL_SCANCODE_INTERNATIONAL5 = 139,
    None,                          // SDL_SCANCODE_INTERNATIONAL6 = 140,
    None,                          // SDL_SCANCODE_INTERNATIONAL7 = 141,
    None,                          // SDL_SCANCODE_INTERNATIONAL8 = 142,
    None,                          // SDL_SCANCODE_INTERNATIONAL9 = 143,
    None,                          // SDL_SCANCODE_LANG1 = 144,
    None,                          // SDL_SCANCODE_LANG2 = 145,
    None,                          // SDL_SCANCODE_LANG3 = 146,
    None,                          // SDL_SCANCODE_LANG4 = 147,
    None,                          // SDL_SCANCODE_LANG5 = 148,
    None,                          // SDL_SCANCODE_LANG6 = 149,
    None,                          // SDL_SCANCODE_LANG7 = 150,
    None,                          // SDL_SCANCODE_LANG8 = 151,
    None,                          // SDL_SCANCODE_LANG9 = 152,
    None,                          // SDL_SCANCODE_ALTERASE = 153,
    Some(Ps2Scancode::SysReq),     // SDL_SCANCODE_SYSREQ = 154,
    None,                          // SDL_SCANCODE_CANCEL = 155,
    None,                          // SDL_SCANCODE_CLEAR = 156,
    None,                          // SDL_SCANCODE_PRIOR = 157,
    None,                          // SDL_SCANCODE_RETURN2 = 158,
    None,                          // SDL_SCANCODE_SEPARATOR = 159,
    None,                          // SDL_SCANCODE_OUT = 160,
    None,                          // SDL_SCANCODE_OPER = 161,
    None,                          // SDL_SCANCODE_CLEARAGAIN = 162,
    None,                          // SDL_SCANCODE_CRSEL = 163,
    None,                          // SDL_SCANCODE_EXSEL = 164,
    None,                          // NOT MAPPED IN SDL3 165
    None,                          // NOT MAPPED IN SDL3 166
    None,                          // NOT MAPPED IN SDL3 167
    None,                          // NOT MAPPED IN SDL3 168
    None,                          // NOT MAPPED IN SDL3 169
    None,                          // NOT MAPPED IN SDL3 170
    None,                          // NOT MAPPED IN SDL3 171
    None,                          // NOT MAPPED IN SDL3 172
    None,                          // NOT MAPPED IN SDL3 173
    None,                          // NOT MAPPED IN SDL3 174
    None,                          // NOT MAPPED IN SDL3 175
    None,                          // SDL_SCANCODE_KP_00 = 176,
    None,                          // SDL_SCANCODE_KP_000 = 177,
    None,                          // SDL_SCANCODE_THOUSANDSSEPARATOR = 178,
    None,                          // SDL_SCANCODE_DECIMALSEPARATOR = 179,
    None,                          // SDL_SCANCODE_CURRENCYUNIT = 180,
    None,                          // SDL_SCANCODE_CURRENCYSUBUNIT = 181,
    None,                          // SDL_SCANCODE_KP_LEFTPAREN = 182,
    None,                          // SDL_SCANCODE_KP_RIGHTPAREN = 183,
    None,                          // SDL_SCANCODE_KP_LEFTBRACE = 184,
    None,                          // SDL_SCANCODE_KP_RIGHTBRACE = 185,
    None,                          // SDL_SCANCODE_KP_TAB = 186,
    None,                          // SDL_SCANCODE_KP_BACKSPACE = 187,
    None,                          // SDL_SCANCODE_KP_A = 188,
    None,                          // SDL_SCANCODE_KP_B = 189,
    None,                          // SDL_SCANCODE_KP_C = 190,
    None,                          // SDL_SCANCODE_KP_D = 191,
    None,                          // SDL_SCANCODE_KP_E = 192,
    None,                          // SDL_SCANCODE_KP_F = 193,
    None,                          // SDL_SCANCODE_KP_XOR = 194,
    None,                          // SDL_SCANCODE_KP_POWER = 195,
    None,                          // SDL_SCANCODE_KP_PERCENT = 196,
    None,                          // SDL_SCANCODE_KP_LESS = 197,
    None,                          // SDL_SCANCODE_KP_GREATER = 198,
    None,                          // SDL_SCANCODE_KP_AMPERSAND = 199,
    None,                          // SDL_SCANCODE_KP_DBLAMPERSAND = 200,
    None,                          // SDL_SCANCODE_KP_VERTICALBAR = 201,
    None,                          // SDL_SCANCODE_KP_DBLVERTICALBAR = 202,
    None,                          // SDL_SCANCODE_KP_COLON = 203,
    None,                          // SDL_SCANCODE_KP_HASH = 204,
    None,                          // SDL_SCANCODE_KP_SPACE = 205,
    None,                          // SDL_SCANCODE_KP_AT = 206,
    None,                          // SDL_SCANCODE_KP_EXCLAM = 207,
    None,                          // SDL_SCANCODE_KP_MEMSTORE = 208,
    None,                          // SDL_SCANCODE_KP_MEMRECALL = 209,
    None,                          // SDL_SCANCODE_KP_MEMCLEAR = 210,
    None,                          // SDL_SCANCODE_KP_MEMADD = 211,
    None,                          // SDL_SCANCODE_KP_MEMSUBTRACT = 212,
    None,                          // SDL_SCANCODE_KP_MEMMULTIPLY = 213,
    None,                          // SDL_SCANCODE_KP_MEMDIVIDE = 214,
    None,                          // SDL_SCANCODE_KP_PLUSMINUS = 215,
    None,                          // SDL_SCANCODE_KP_CLEAR = 216,
    None,                          // SDL_SCANCODE_KP_CLEARENTRY = 217,
    None,                          // SDL_SCANCODE_KP_BINARY = 218,
    None,                          // SDL_SCANCODE_KP_OCTAL = 219,
    None,                          // SDL_SCANCODE_KP_DECIMAL = 220,
    None,                          // SDL_SCANCODE_KP_HEXADECIMAL = 221,
    None,                          // NOT MAPPED IN SDL3 222
    None,                          // NOT MAPPED IN SDL3 223
    Some(Ps2Scancode::LeftCtrl),   // SDL_SCANCODE_LCTRL = 224,
    Some(Ps2Scancode::LeftShift),  // SDL_SCANCODE_LSHIFT = 225,
    Some(Ps2Scancode::LeftAlt),    // SDL_SCANCODE_LALT = 226,
    Some(Ps2Scancode::LeftMeta),   // SDL_SCANCODE_LGUI = 227,
    Some(Ps2Scancode::RightCtrl),  // SDL_SCANCODE_RCTRL = 228,
    Some(Ps2Scancode::RightShift), // SDL_SCANCODE_RSHIFT = 229,
    Some(Ps2Scancode::RightAlt),   // SDL_SCANCODE_RALT = 230,
    Some(Ps2Scancode::RightMeta),  // SDL_SCANCODE_RGUI = 231,
    None,                          // NOT MAPPED IN SDL3 232
    None,                          // NOT MAPPED IN SDL3 233
    None,                          // NOT MAPPED IN SDL3 234
    None,                          // NOT MAPPED IN SDL3 235
    None,                          // NOT MAPPED IN SDL3 236
    None,                          // NOT MAPPED IN SDL3 237
    None,                          // NOT MAPPED IN SDL3 238
    None,                          // NOT MAPPED IN SDL3 239
    None,                          // NOT MAPPED IN SDL3 240
    None,                          // NOT MAPPED IN SDL3 241
    None,                          // NOT MAPPED IN SDL3 242
    None,                          // NOT MAPPED IN SDL3 243
    None,                          // NOT MAPPED IN SDL3 244
    None,                          // NOT MAPPED IN SDL3 245
    None,                          // NOT MAPPED IN SDL3 246
    None,                          // NOT MAPPED IN SDL3 247
    None,                          // NOT MAPPED IN SDL3 248
    None,                          // NOT MAPPED IN SDL3 249
    None,                          // NOT MAPPED IN SDL3 250
    None,                          // NOT MAPPED IN SDL3 251
    None,                          // NOT MAPPED IN SDL3 252
    None,                          // NOT MAPPED IN SDL3 253
    None,                          // NOT MAPPED IN SDL3 254
    None,                          // NOT MAPPED IN SDL3 255
    // The following would make the map overflow, so we don't include them.
    // None,                          // NOT MAPPED IN SDL3 256
    // None,                          // SDL_SCANCODE_MODE = 257,
    // None,                          // SDL_SCANCODE_AUDIONEXT = 258,
    // None,                          // SDL_SCANCODE_AUDIOPREV = 259,
    // None,                          // SDL_SCANCODE_AUDIOSTOP = 260,
    // None,                          // SDL_SCANCODE_AUDIOPLAY = 261,
    // None,                          // SDL_SCANCODE_AUDIOMUTE = 262,
    // None,                          // SDL_SCANCODE_MEDIASELECT = 263,
    // None,                          // SDL_SCANCODE_WWW = 264,
    // None,                          // SDL_SCANCODE_MAIL = 265,
    // None,                          // SDL_SCANCODE_CALCULATOR = 266,
    // None,                          // SDL_SCANCODE_COMPUTER = 267,
    // None,                          // SDL_SCANCODE_AC_SEARCH = 268,
    // None,                          // SDL_SCANCODE_AC_HOME = 269,
    // None,                          // SDL_SCANCODE_AC_BACK = 270,
    // None,                          // SDL_SCANCODE_AC_FORWARD = 271,
    // None,                          // SDL_SCANCODE_AC_STOP = 272,
    // None,                          // SDL_SCANCODE_AC_REFRESH = 273,
    // None,                          // SDL_SCANCODE_AC_BOOKMARKS = 274,
    // None,                          // SDL_SCANCODE_BRIGHTNESSDOWN = 275,
    // None,                          // SDL_SCANCODE_BRIGHTNESSUP = 276,
    // None,                          // SDL_SCANCODE_DISPLAYSWITCH = 277,
    // None,                          // SDL_SCANCODE_KBDILLUMTOGGLE = 278,
    // None,                          // SDL_SCANCODE_KBDILLUMDOWN = 279,
    // None,                          // SDL_SCANCODE_KBDILLUMUP = 280,
    // None,                          // SDL_SCANCODE_EJECT = 281,
    // None,                          // SDL_SCANCODE_SLEEP = 282,
    // None,                          // SDL_SCANCODE_APP1 = 283,
    // None,                          // SDL_SCANCODE_APP2 = 284,
    // None,                          // SDL_SCANCODE_AUDIOREWIND = 285,
    // None,                          // SDL_SCANCODE_AUDIOFASTFORWARD = 286,
    // None,                          // SDL_SCANCODE_SOFTLEFT = 287,
    // None,                          // SDL_SCANCODE_SOFTRIGHT = 288,
    // None,                          // SDL_SCANCODE_CALL = 289,
    // None,                          // SDL_SCANCODE_ENDCALL = 290,
]);

/// PS/2 keyboard scancodes. This is the IBM PS/2 ports.
/// Use one of the many `From<>` implementations to instantiate this
/// type.
#[derive(strum::EnumCount, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
#[rustfmt::skip]
pub enum Ps2Scancode {
    None = 0xFF,
    F9 = 0x01,                        //67  KEY_F9
    F5 = 0x03,                        //63  KEY_F5
    F3 = 0x04,                        //61  KEY_F3
    F1 = 0x05,                        //59  KEY_F1
    F2 = 0x06,                        //60  KEY_F2
    F12 = 0x07,                       //88  KEY_F12
    F10 = 0x09,                       //68  KEY_F10
    F8 = 0x0a,                        //66  KEY_F8
    F6 = 0x0b,                        //64  KEY_F6
    F4 = 0x0c,                        //62  KEY_F4
    Tab = 0x0d,                       //15  KEY_TAB
    Grave = 0x0e,                     //41  KEY_GRAVE
    RO = 0x13,                        //89  KEY_RO
    Q = 0x15,                         //16  KEY_Q
    Key1 = 0x16,                      //2   KEY_1
    Z = 0x1a,                         //44  KEY_Z
    S = 0x1b,                         //31  KEY_S
    A = 0x1c,                         //30  KEY_A
    W = 0x1d,                         //17  KEY_W
    Key2 = 0x1e,                      //3   KEY_2
    C = 0x21,                         //46  KEY_C
    X = 0x22,                         //45  KEY_X
    D = 0x23,                         //32  KEY_D
    E = 0x24,                         //18  KEY_E
    Key4 = 0x25,                      //5   KEY_4
    Key3 = 0x26,                      //4   KEY_3
    Space = 0x29,                     //57  KEY_SPACE
    V = 0x2a,                         //47  KEY_V
    F = 0x2b,                         //33  KEY_F
    T = 0x2c,                         //20  KEY_T
    R = 0x2d,                         //19  KEY_R
    Key5 = 0x2e,                      //6   KEY_5
    N = 0x31,                         //49  KEY_N
    B = 0x32,                         //48  KEY_B
    H = 0x33,                         //35  KEY_H
    G = 0x34,                         //34  KEY_G
    Y = 0x35,                         //21  KEY_Y
    Key6 = 0x36,                      //7   KEY_6
    M = 0x3a,                         //50  KEY_M
    J = 0x3b,                         //36  KEY_J
    U = 0x3c,                         //22  KEY_U
    Key7 = 0x3d,                      //8   KEY_7
    Key8 = 0x3e,                      //9   KEY_8
    Comma = 0x41,                     //51  KEY_COMMA
    K = 0x42,                         //37  KEY_K
    I = 0x43,                         //23  KEY_I
    O = 0x44,                         //24  KEY_O
    Key0 = 0x45,                      //11  KEY_0
    Key9 = 0x46,                      //10  KEY_9
    Dot = 0x49,                       //52  KEY_DOT
    Slash = 0x4a,                     //53  KEY_SLASH
    L = 0x4b,                         //38  KEY_L
    SemiColon = 0x4c,                 //39  KEY_SEMICOLON
    P = 0x4d,                         //25  KEY_P
    Minus = 0x4e,                     //12  KEY_MINUS
    Apostrophe = 0x52,                //40  KEY_APOSTROPHE
    LeftBrace = 0x54,                 //26  KEY_LEFTBRACE
    Equal = 0x55,                     //13  KEY_EQUAL
    CapsLock = 0x58,                  //58  KEY_CAPSLOCK
    Enter = 0x5a,                     //28  KEY_ENTER
    RightBrace = 0x5b,                //27  KEY_RIGHTBRACE
    Backslash = 0x5d,                 //43  KEY_BACKSLASH
    Key102Nd = 0x61,                  //86  KEY_102ND
    Henkan = 0x64,                    //92  KEY_HENKAN
    Backspace = 0x66,                 //14  KEY_BACKSPACE
    Muhenkan = 0x67,                  //94  KEY_MUHENKAN
    Kp1 = 0x69,                       //79  KEY_KP1
    Yen = 0x6a,                       //124 KEY_YEN
    Kp4 = 0x6b,                       //75  KEY_KP4
    Kp7 = 0x6c,                       //71  KEY_KP7
    Kp0 = 0x70,                       //82  KEY_KP0
    KpDot = 0x71,                     //83  KEY_KPDOT
    Kp2 = 0x72,                       //80  KEY_KP2
    Kp5 = 0x73,                       //76  KEY_KP5
    Kp6 = 0x74,                       //77  KEY_KP6
    Kp8 = 0x75,                       //72  KEY_KP8
    Esc = 0x76,                       //1   KEY_ESC
    F11 = 0x78,                       //87  KEY_F11
    KpPlus = 0x79,                    //78  KEY_KPPLUS
    Kp3 = 0x7a,                       //81  KEY_KP3
    KpMinus = 0x7b,                   //74  KEY_KPMINUS
    KpAsterisk = 0x7c,                //55  KEY_KPASTERISK
    Kp9 = 0x7d,                       //73  KEY_KP9
    F7 = 0x83,                        //65  KEY_F7
    Pause = 0xE1,                     //119 KEY_PAUSE
    SysReq = 0xE2,                    //99  KEY_SYSRQ
    ScrollLock = EMU_SWITCH_1 + 0x7E, //70  KEY_SCROLLLOCK
    F17 = EMU_SWITCH_1 + 1,           //187 KEY_F17
    F18 = EMU_SWITCH_1 + 2,           //188 KEY_F18
    F19 = EMU_SWITCH_1 + 3,           //189 KEY_F19
    F20 = EMU_SWITCH_1 + 4,           //190 KEY_F20
    NumLock = EMU_SWITCH_2 + 0x77,    //69  KEY_NUMLOCK
    Compose = EXT + 0x2f,             //127 KEY_COMPOSE
    KpSlash = EXT + 0x4a,             //98  KEY_KPSLASH
    KpEnter = EXT + 0x5a,             //96  KEY_KPENTER
    End = EXT + 0x69,                 //107 KEY_END
    Left = EXT + 0x6b,                //105 KEY_LEFT
    Home = EXT + 0x6c,                //102 KEY_HOME
    Insert = EXT + 0x70,              //110 KEY_INSERT
    Delete = EXT + 0x71,              //111 KEY_DELETE
    Down = EXT + 0x72,                //108 KEY_DOWN
    Right = EXT + 0x74,               //106 KEY_RIGHT
    Up = EXT + 0x75,                  //103 KEY_UP
    PageDown = EXT + 0x7a,            //109 KEY_PAGEDOWN
    PageUp = EXT + 0x7d,              //104 KEY_PAGEUP
    LeftAlt = LALT + 0x11,            //56  KEY_LEFTALT
    LeftCtrl = LCTRL + 0x14,          //29  KEY_LEFTCTRL
    LeftMeta = LGUI + EXT + 0x1f,     //125 KEY_LEFTMETA
    LeftShift = LSHIFT + 0x12,        //42  KEY_LEFTSHIFT
    RightAlt = RALT + EXT + 0x11,     //100 KEY_RIGHTALT
    RightCtrl = RCTRL + EXT + 0x14,   //97  KEY_RIGHTCTRL
    RightMeta = RGUI + EXT + 0x27,    //126 KEY_RIGHTMETA
    RightShift = RSHIFT + 0x59,       //54  KEY_RIGHTSHIFT
}

impl From<one_fpga::inputs::Scancode> for Ps2Scancode {
    fn from(scancode: one_fpga::inputs::Scancode) -> Self {
        SDL3_TO_PS2[scancode.as_repr() as u8].unwrap_or(Ps2Scancode::None)
    }
}

impl Ps2Scancode {
    pub(crate) fn as_u32(&self) -> u32 {
        *self as u32
    }
}

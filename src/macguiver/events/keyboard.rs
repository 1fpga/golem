//! A generic Keycode for the Macguiver framework.
use std::fmt;
use strum::{Display, EnumCount, FromRepr, IntoEnumIterator};

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Hash,
    Debug,
    Display,
    EnumCount,
    FromRepr,
    strum::EnumIter,
    strum::IntoStaticStr,
)]
#[repr(u8)]
pub enum Keycode {
    /// This keycode is only used for arrays that are indexed by scancode. It is used as a
    /// filler for the array.
    None = 0,

    Backspace,
    Tab,
    Return,
    Escape,
    Space,
    Exclaim,
    Quotedbl,
    Hash,
    Dollar,
    Percent,
    Ampersand,
    Quote,
    LeftParen,
    RightParen,
    Asterisk,
    Plus,
    Comma,
    Minus,
    Period,
    Slash,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Colon,
    Semicolon,
    Less,
    Equals,
    Greater,
    Question,
    At,
    LeftBracket,
    Backslash,
    RightBracket,
    Caret,
    Underscore,
    Backquote,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Delete,
    CapsLock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    PrintScreen,
    ScrollLock,
    Pause,
    Insert,
    Home,
    PageUp,
    End,
    PageDown,
    Right,
    Left,
    Down,
    Up,
    NumLockClear,
    KpDivide,
    KpMultiply,
    KpMinus,
    KpPlus,
    KpEnter,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    Kp0,
    KpPeriod,
    Application,
    Power,
    KpEquals,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    Execute,
    Help,
    Menu,
    Select,
    Stop,
    Again,
    Undo,
    Cut,
    Copy,
    Paste,
    Find,
    Mute,
    VolumeUp,
    VolumeDown,
    KpComma,
    KpEqualsAS400,
    AltErase,
    Sysreq,
    Cancel,
    Clear,
    Prior,
    Return2,
    Separator,
    Out,
    Oper,
    ClearAgain,
    CrSel,
    ExSel,
    Kp00,
    Kp000,
    ThousandsSeparator,
    DecimalSeparator,
    CurrencyUnit,
    CurrencySubUnit,
    KpLeftParen,
    KpRightParen,
    KpLeftBrace,
    KpRightBrace,
    KpTab,
    KpBackspace,
    KpA,
    KpB,
    KpC,
    KpD,
    KpE,
    KpF,
    KpXor,
    KpPower,
    KpPercent,
    KpLess,
    KpGreater,
    KpAmpersand,
    KpDblAmpersand,
    KpVerticalBar,
    KpDblVerticalBar,
    KpColon,
    KpHash,
    KpSpace,
    KpAt,
    KpExclam,
    KpMemStore,
    KpMemRecall,
    KpMemClear,
    KpMemAdd,
    KpMemSubtract,
    KpMemMultiply,
    KpMemDivide,
    KpPlusMinus,
    KpClear,
    KpClearEntry,
    KpBinary,
    KpOctal,
    KpDecimal,
    KpHexadecimal,
    LCtrl,
    LShift,
    LAlt,
    LGui,
    RCtrl,
    RShift,
    RAlt,
    RGui,
    Mode,
    AudioNext,
    AudioPrev,
    AudioStop,
    AudioPlay,
    AudioMute,
    MediaSelect,
    Www,
    Mail,
    Calculator,
    Computer,
    AcSearch,
    AcHome,
    AcBack,
    AcForward,
    AcStop,
    AcRefresh,
    AcBookmarks,
    BrightnessDown,
    BrightnessUp,
    DisplaySwitch,
    KbdIllumToggle,
    KbdIllumDown,
    KbdIllumUp,
    Eject,
    Sleep,
}

impl Keycode {
    pub fn is_modifier(&self) -> bool {
        matches!(
            self,
            Keycode::LCtrl
                | Keycode::LShift
                | Keycode::LAlt
                | Keycode::LGui
                | Keycode::RCtrl
                | Keycode::RShift
                | Keycode::RAlt
                | Keycode::RGui
                | Keycode::Mode
        )
    }
}

/// A Map of pressed keycodes. We only keep up to 8 keycodes by default pressed at once,
/// which is more than the USB HID report can handle anyway. Certain keyboards can go up
/// to 16, so we allow that to be configured.
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct KeycodeMap<const N: usize = 16> {
    codes: [Keycode; N],
}

impl<const N: usize> Default for KeycodeMap<N> {
    fn default() -> Self {
        Self {
            codes: [Keycode::None; N],
        }
    }
}

impl<const N: usize> fmt::Debug for KeycodeMap<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fmt = f.debug_tuple("KeycodeMap");

        for code in self.codes.iter() {
            fmt.field(&code);
        }

        fmt.finish()
    }
}

impl ToString for KeycodeMap {
    fn to_string(&self) -> String {
        let mut string = String::new();

        self.iter_modifiers()
            .for_each(|key| string.push_str(&format!("{:?} + ", key)));
        self.iter_non_modifiers()
            .for_each(|key| string.push_str(&format!("{:?} + ", key)));

        if string.len() > 3 {
            string.truncate(string.len() - 3);
        }
        string
    }
}

impl<const N: usize> KeycodeMap<N> {
    pub fn down(&mut self, key: Keycode) {
        if self.codes.contains(&key) {
            return;
        }

        for i in 0..N {
            if self.codes[i] == Keycode::None {
                self.codes[i] = key;
                break;
            }
        }
    }

    pub fn up(&mut self, key: Keycode) {
        'root: for i in 0..N {
            if self.codes[i] == key {
                for j in i..(N - 1) {
                    self.codes[j] = self.codes[j + 1];
                }
                self.codes[N - 1] = Keycode::None;
                break 'root;
            }
        }
    }

    pub fn is_down(&self, key: Keycode) -> bool {
        !self.is_up(key)
    }

    pub fn is_up(&self, key: Keycode) -> bool {
        self.codes.contains(&key)
    }

    pub fn clear(&mut self) {
        self.codes = [Keycode::None; N];
    }

    pub fn to_array(&self) -> [Keycode; N] {
        self.codes
    }

    pub fn iter_modifiers(&self) -> impl Iterator<Item = Keycode> + '_ {
        self.iter_down().filter(|code| code.is_modifier())
    }

    pub fn iter_non_modifiers(&self) -> impl Iterator<Item = Keycode> + '_ {
        self.iter_down().filter(|code| !code.is_modifier())
    }

    pub fn iter_down(&self) -> impl Iterator<Item = Keycode> + '_ {
        self.codes
            .iter()
            .filter(|code| **code != Keycode::None)
            .copied()
    }

    pub fn matches(&self, codes: &[Keycode]) -> bool {
        codes.iter().all(|code| self.codes.contains(code))
    }
}

impl AsRef<[Keycode]> for KeycodeMap {
    fn as_ref(&self) -> &[Keycode] {
        &self.codes
    }
}

#[test]
fn keymap() {
    let mut map = KeycodeMap::default();

    map.down(Keycode::A);
    map.down(Keycode::A);
    map.down(Keycode::A);
    map.down(Keycode::A);
    map.up(Keycode::A);
    map.down(Keycode::A);
    map.up(Keycode::A);
    map.down(Keycode::A);
    map.up(Keycode::A);
    map.down(Keycode::A);
    map.up(Keycode::A);

    assert_eq!(map.codes, [Keycode::None; 8]);
}

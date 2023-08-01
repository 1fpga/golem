use sdl3::keyboard::{KeyboardState, Keycode as SdlKeycode, Scancode};
use std::fmt;
use std::iter::FromIterator;

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Hash,
    Debug,
    strum::Display,
    strum::EnumCount,
    strum::FromRepr,
    strum::EnumIter,
    strum::IntoStaticStr,
)]
#[repr(i32)]
pub enum Keycode {
    /// This keycode is only used for arrays that are indexed by scancode. It is used as a
    /// filler for the array.
    None = 0,
    Backspace = SdlKeycode::Backspace as i32,
    Tab = SdlKeycode::Tab as i32,
    Return = SdlKeycode::Return as i32,
    Escape = SdlKeycode::Escape as i32,
    Space = SdlKeycode::Space as i32,
    Exclaim = SdlKeycode::Exclaim as i32,
    Quotedbl = SdlKeycode::Quotedbl as i32,
    Hash = SdlKeycode::Hash as i32,
    Dollar = SdlKeycode::Dollar as i32,
    Percent = SdlKeycode::Percent as i32,
    Ampersand = SdlKeycode::Ampersand as i32,
    Quote = SdlKeycode::Quote as i32,
    LeftParen = SdlKeycode::LeftParen as i32,
    RightParen = SdlKeycode::RightParen as i32,
    Asterisk = SdlKeycode::Asterisk as i32,
    Plus = SdlKeycode::Plus as i32,
    Comma = SdlKeycode::Comma as i32,
    Minus = SdlKeycode::Minus as i32,
    Period = SdlKeycode::Period as i32,
    Slash = SdlKeycode::Slash as i32,
    Num0 = SdlKeycode::Num0 as i32,
    Num1 = SdlKeycode::Num1 as i32,
    Num2 = SdlKeycode::Num2 as i32,
    Num3 = SdlKeycode::Num3 as i32,
    Num4 = SdlKeycode::Num4 as i32,
    Num5 = SdlKeycode::Num5 as i32,
    Num6 = SdlKeycode::Num6 as i32,
    Num7 = SdlKeycode::Num7 as i32,
    Num8 = SdlKeycode::Num8 as i32,
    Num9 = SdlKeycode::Num9 as i32,
    Colon = SdlKeycode::Colon as i32,
    Semicolon = SdlKeycode::Semicolon as i32,
    Less = SdlKeycode::Less as i32,
    Equals = SdlKeycode::Equals as i32,
    Greater = SdlKeycode::Greater as i32,
    Question = SdlKeycode::Question as i32,
    At = SdlKeycode::At as i32,
    LeftBracket = SdlKeycode::LeftBracket as i32,
    Backslash = SdlKeycode::Backslash as i32,
    RightBracket = SdlKeycode::RightBracket as i32,
    Caret = SdlKeycode::Caret as i32,
    Underscore = SdlKeycode::Underscore as i32,
    Backquote = SdlKeycode::Backquote as i32,
    A = SdlKeycode::A as i32,
    B = SdlKeycode::B as i32,
    C = SdlKeycode::C as i32,
    D = SdlKeycode::D as i32,
    E = SdlKeycode::E as i32,
    F = SdlKeycode::F as i32,
    G = SdlKeycode::G as i32,
    H = SdlKeycode::H as i32,
    I = SdlKeycode::I as i32,
    J = SdlKeycode::J as i32,
    K = SdlKeycode::K as i32,
    L = SdlKeycode::L as i32,
    M = SdlKeycode::M as i32,
    N = SdlKeycode::N as i32,
    O = SdlKeycode::O as i32,
    P = SdlKeycode::P as i32,
    Q = SdlKeycode::Q as i32,
    R = SdlKeycode::R as i32,
    S = SdlKeycode::S as i32,
    T = SdlKeycode::T as i32,
    U = SdlKeycode::U as i32,
    V = SdlKeycode::V as i32,
    W = SdlKeycode::W as i32,
    X = SdlKeycode::X as i32,
    Y = SdlKeycode::Y as i32,
    Z = SdlKeycode::Z as i32,
    Delete = SdlKeycode::Delete as i32,
    CapsLock = SdlKeycode::CapsLock as i32,
    F1 = SdlKeycode::F1 as i32,
    F2 = SdlKeycode::F2 as i32,
    F3 = SdlKeycode::F3 as i32,
    F4 = SdlKeycode::F4 as i32,
    F5 = SdlKeycode::F5 as i32,
    F6 = SdlKeycode::F6 as i32,
    F7 = SdlKeycode::F7 as i32,
    F8 = SdlKeycode::F8 as i32,
    F9 = SdlKeycode::F9 as i32,
    F10 = SdlKeycode::F10 as i32,
    F11 = SdlKeycode::F11 as i32,
    F12 = SdlKeycode::F12 as i32,
    PrintScreen = SdlKeycode::PrintScreen as i32,
    ScrollLock = SdlKeycode::ScrollLock as i32,
    Pause = SdlKeycode::Pause as i32,
    Insert = SdlKeycode::Insert as i32,
    Home = SdlKeycode::Home as i32,
    PageUp = SdlKeycode::PageUp as i32,
    End = SdlKeycode::End as i32,
    PageDown = SdlKeycode::PageDown as i32,
    Right = SdlKeycode::Right as i32,
    Left = SdlKeycode::Left as i32,
    Down = SdlKeycode::Down as i32,
    Up = SdlKeycode::Up as i32,
    NumLockClear = SdlKeycode::NumLockClear as i32,
    KpDivide = SdlKeycode::KpDivide as i32,
    KpMultiply = SdlKeycode::KpMultiply as i32,
    KpMinus = SdlKeycode::KpMinus as i32,
    KpPlus = SdlKeycode::KpPlus as i32,
    KpEnter = SdlKeycode::KpEnter as i32,
    Kp1 = SdlKeycode::Kp1 as i32,
    Kp2 = SdlKeycode::Kp2 as i32,
    Kp3 = SdlKeycode::Kp3 as i32,
    Kp4 = SdlKeycode::Kp4 as i32,
    Kp5 = SdlKeycode::Kp5 as i32,
    Kp6 = SdlKeycode::Kp6 as i32,
    Kp7 = SdlKeycode::Kp7 as i32,
    Kp8 = SdlKeycode::Kp8 as i32,
    Kp9 = SdlKeycode::Kp9 as i32,
    Kp0 = SdlKeycode::Kp0 as i32,
    KpPeriod = SdlKeycode::KpPeriod as i32,
    Application = SdlKeycode::Application as i32,
    Power = SdlKeycode::Power as i32,
    KpEquals = SdlKeycode::KpEquals as i32,
    F13 = SdlKeycode::F13 as i32,
    F14 = SdlKeycode::F14 as i32,
    F15 = SdlKeycode::F15 as i32,
    F16 = SdlKeycode::F16 as i32,
    F17 = SdlKeycode::F17 as i32,
    F18 = SdlKeycode::F18 as i32,
    F19 = SdlKeycode::F19 as i32,
    F20 = SdlKeycode::F20 as i32,
    F21 = SdlKeycode::F21 as i32,
    F22 = SdlKeycode::F22 as i32,
    F23 = SdlKeycode::F23 as i32,
    F24 = SdlKeycode::F24 as i32,
    Execute = SdlKeycode::Execute as i32,
    Help = SdlKeycode::Help as i32,
    Menu = SdlKeycode::Menu as i32,
    Select = SdlKeycode::Select as i32,
    Stop = SdlKeycode::Stop as i32,
    Again = SdlKeycode::Again as i32,
    Undo = SdlKeycode::Undo as i32,
    Cut = SdlKeycode::Cut as i32,
    Copy = SdlKeycode::Copy as i32,
    Paste = SdlKeycode::Paste as i32,
    Find = SdlKeycode::Find as i32,
    Mute = SdlKeycode::Mute as i32,
    VolumeUp = SdlKeycode::VolumeUp as i32,
    VolumeDown = SdlKeycode::VolumeDown as i32,
    KpComma = SdlKeycode::KpComma as i32,
    KpEqualsAS400 = SdlKeycode::KpEqualsAS400 as i32,
    AltErase = SdlKeycode::AltErase as i32,
    Sysreq = SdlKeycode::Sysreq as i32,
    Cancel = SdlKeycode::Cancel as i32,
    Clear = SdlKeycode::Clear as i32,
    Prior = SdlKeycode::Prior as i32,
    Return2 = SdlKeycode::Return2 as i32,
    Separator = SdlKeycode::Separator as i32,
    Out = SdlKeycode::Out as i32,
    Oper = SdlKeycode::Oper as i32,
    ClearAgain = SdlKeycode::ClearAgain as i32,
    CrSel = SdlKeycode::CrSel as i32,
    ExSel = SdlKeycode::ExSel as i32,
    Kp00 = SdlKeycode::Kp00 as i32,
    Kp000 = SdlKeycode::Kp000 as i32,
    ThousandsSeparator = SdlKeycode::ThousandsSeparator as i32,
    DecimalSeparator = SdlKeycode::DecimalSeparator as i32,
    CurrencyUnit = SdlKeycode::CurrencyUnit as i32,
    CurrencySubUnit = SdlKeycode::CurrencySubUnit as i32,
    KpLeftParen = SdlKeycode::KpLeftParen as i32,
    KpRightParen = SdlKeycode::KpRightParen as i32,
    KpLeftBrace = SdlKeycode::KpLeftBrace as i32,
    KpRightBrace = SdlKeycode::KpRightBrace as i32,
    KpTab = SdlKeycode::KpTab as i32,
    KpBackspace = SdlKeycode::KpBackspace as i32,
    KpA = SdlKeycode::KpA as i32,
    KpB = SdlKeycode::KpB as i32,
    KpC = SdlKeycode::KpC as i32,
    KpD = SdlKeycode::KpD as i32,
    KpE = SdlKeycode::KpE as i32,
    KpF = SdlKeycode::KpF as i32,
    KpXor = SdlKeycode::KpXor as i32,
    KpPower = SdlKeycode::KpPower as i32,
    KpPercent = SdlKeycode::KpPercent as i32,
    KpLess = SdlKeycode::KpLess as i32,
    KpGreater = SdlKeycode::KpGreater as i32,
    KpAmpersand = SdlKeycode::KpAmpersand as i32,
    KpDblAmpersand = SdlKeycode::KpDblAmpersand as i32,
    KpVerticalBar = SdlKeycode::KpVerticalBar as i32,
    KpDblVerticalBar = SdlKeycode::KpDblVerticalBar as i32,
    KpColon = SdlKeycode::KpColon as i32,
    KpHash = SdlKeycode::KpHash as i32,
    KpSpace = SdlKeycode::KpSpace as i32,
    KpAt = SdlKeycode::KpAt as i32,
    KpExclam = SdlKeycode::KpExclam as i32,
    KpMemStore = SdlKeycode::KpMemStore as i32,
    KpMemRecall = SdlKeycode::KpMemRecall as i32,
    KpMemClear = SdlKeycode::KpMemClear as i32,
    KpMemAdd = SdlKeycode::KpMemAdd as i32,
    KpMemSubtract = SdlKeycode::KpMemSubtract as i32,
    KpMemMultiply = SdlKeycode::KpMemMultiply as i32,
    KpMemDivide = SdlKeycode::KpMemDivide as i32,
    KpPlusMinus = SdlKeycode::KpPlusMinus as i32,
    KpClear = SdlKeycode::KpClear as i32,
    KpClearEntry = SdlKeycode::KpClearEntry as i32,
    KpBinary = SdlKeycode::KpBinary as i32,
    KpOctal = SdlKeycode::KpOctal as i32,
    KpDecimal = SdlKeycode::KpDecimal as i32,
    KpHexadecimal = SdlKeycode::KpHexadecimal as i32,
    LCtrl = SdlKeycode::LCtrl as i32,
    LShift = SdlKeycode::LShift as i32,
    LAlt = SdlKeycode::LAlt as i32,
    LGui = SdlKeycode::LGui as i32,
    RCtrl = SdlKeycode::RCtrl as i32,
    RShift = SdlKeycode::RShift as i32,
    RAlt = SdlKeycode::RAlt as i32,
    RGui = SdlKeycode::RGui as i32,
    Mode = SdlKeycode::Mode as i32,
    AudioNext = SdlKeycode::AudioNext as i32,
    AudioPrev = SdlKeycode::AudioPrev as i32,
    AudioStop = SdlKeycode::AudioStop as i32,
    AudioPlay = SdlKeycode::AudioPlay as i32,
    AudioMute = SdlKeycode::AudioMute as i32,
    MediaSelect = SdlKeycode::MediaSelect as i32,
    Www = SdlKeycode::Www as i32,
    Mail = SdlKeycode::Mail as i32,
    Calculator = SdlKeycode::Calculator as i32,
    Computer = SdlKeycode::Computer as i32,
    AcSearch = SdlKeycode::AcSearch as i32,
    AcHome = SdlKeycode::AcHome as i32,
    AcBack = SdlKeycode::AcBack as i32,
    AcForward = SdlKeycode::AcForward as i32,
    AcStop = SdlKeycode::AcStop as i32,
    AcRefresh = SdlKeycode::AcRefresh as i32,
    AcBookmarks = SdlKeycode::AcBookmarks as i32,
    BrightnessDown = SdlKeycode::BrightnessDown as i32,
    BrightnessUp = SdlKeycode::BrightnessUp as i32,
    DisplaySwitch = SdlKeycode::DisplaySwitch as i32,
    KbdIllumToggle = SdlKeycode::KbdIllumToggle as i32,
    KbdIllumDown = SdlKeycode::KbdIllumDown as i32,
    KbdIllumUp = SdlKeycode::KbdIllumUp as i32,
    Eject = SdlKeycode::Eject as i32,
    Sleep = SdlKeycode::Sleep as i32,
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

impl From<Scancode> for Keycode {
    fn from(value: Scancode) -> Self {
        match SdlKeycode::from_scancode(value) {
            None => Keycode::None,
            Some(kc) => kc.into(),
        }
    }
}

impl From<SdlKeycode> for Keycode {
    fn from(value: SdlKeycode) -> Self {
        unsafe { std::mem::transmute(value as i32) }
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

impl From<KeyboardState<'_>> for KeycodeMap {
    fn from(value: KeyboardState) -> Self {
        value.pressed_scancodes().map(Keycode::from).collect()
    }
}

impl FromIterator<Keycode> for KeycodeMap {
    fn from_iter<T: IntoIterator<Item = Keycode>>(iter: T) -> Self {
        let mut s = KeycodeMap::default();
        iter.into_iter().for_each(|kc| s.down(kc));
        s
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

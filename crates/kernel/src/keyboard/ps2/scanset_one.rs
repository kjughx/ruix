#[repr(u8)]
#[derive(Clone, Copy)]
enum SetOne {
    None,
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Dash,
    Equal,
    Backspace,
    Tab,
    Q,
    W,
    E,
    R,
    T,
    Y,
    U,
    I,
    O,
    P,
    LeftBracket,
    RightBracket,
    Enter,
    LeftCtrl, // TODO
    A,
    S,
    D,
    F,
    G,
    H,
    J,
    K,
    L,
    SemiColon,
    Tick,
    Grave,
    LeftShift, // TODO,
    BackSlash,
    Z,
    X,
    C,
    V,
    B,
    N,
    M,
    Comma,
    Dot,
    ForwardSlash,
    RightShift, // TODO
    Asterix,
    LeftAlt,
    Space,
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
    NumLock,
    ScrollLock,
    KeyPad7,
    KeyPad8,
    KeyPad9,
    KeyPadDash,
    KeyPad4,
    KeyPad5,
    KeyPad6,
    KeyPadPlus,
    KeyPad1,
    KeyPad2,
    KeyPad3,
    KeyPad0,
    KeypadDot,
    F11,
    F12,
    Count,
}

impl From<u8> for SetOne {
    fn from(value: u8) -> Self {
        // # Safety
        // 1. The enum is #[repr(u8)]
        // 2. We use '%' to handle values above the "max"
        unsafe { core::mem::transmute(value % Self::Count as u8) }
    }
}

impl SetOne {
    pub fn into_scancode(self, pressed: bool) -> u8 {
        if pressed {
            self as u8 + 0x80
        } else {
            self as u8
        }
    }
}

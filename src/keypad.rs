use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

pub struct KeyPad {
    alpha_lock: bool,
    last_state: u16,
}

impl KeyPad {
    pub fn new() -> Self {
        Self {
            alpha_lock: false,
            last_state: 0,
        }
    }

    pub fn read(&mut self, ahb_frequency: u32) -> [Key; 46] {
        let state = KeyMatrix::scan(ahb_frequency);
        let sum: u16 = state.iter().map(|s| *s as u16).sum();
        let switches = state_to_switches(state);
        let shift = switches.contains(&Switch::R2C1);
        let alpha = switches.contains(&Switch::R2C2) || self.alpha_lock;
        let switch_to_key = if alpha && shift {
            |sw: &Switch| sw.to_key_alpha(true)
        } else if alpha {
            |sw: &Switch| sw.to_key_alpha(false)
        } else if shift {
            |sw: &Switch| sw.to_key_shift()
        } else {
            |sw: &Switch| sw.to_key()
        };
        let iter = switches.iter().map(switch_to_key);
        let mut keys: [Key; 46] = [Key::NONE; 46];
        let mut index = 0;
        iter.for_each(|k| {
            keys[index] = k;
            index += 1;
        });
        if sum != self.last_state {
            self.last_state = sum;
            if keys.contains(&Key::AlphaLock) {
                self.alpha_lock = !self.alpha_lock;
            }
        }
        keys
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum Switch {
    R0C1 = 0x01,
    R0C2 = 0x02,
    R0C3 = 0x03,
    R0C4 = 0x04,
    R0C5 = 0x05,
    R0C6 = 0x06,
    R1C1 = 0x11,
    R1C3 = 0x13,
    R2C1 = 0x21,
    R2C2 = 0x22,
    R2C3 = 0x23,
    R2C4 = 0x24,
    R2C5 = 0x25,
    R2C6 = 0x26,
    R3C1 = 0x31,
    R3C2 = 0x32,
    R3C3 = 0x33,
    R3C4 = 0x34,
    R3C5 = 0x35,
    R3C6 = 0x36,
    R4C1 = 0x41,
    R4C2 = 0x42,
    R4C3 = 0x43,
    R4C4 = 0x44,
    R4C5 = 0x45,
    R4C6 = 0x46,
    R5C1 = 0x51,
    R5C2 = 0x52,
    R5C3 = 0x53,
    R5C4 = 0x54,
    R5C5 = 0x55,
    R6C1 = 0x61,
    R6C2 = 0x62,
    R6C3 = 0x63,
    R6C4 = 0x64,
    R6C5 = 0x65,
    R7C1 = 0x71,
    R7C2 = 0x72,
    R7C3 = 0x73,
    R7C4 = 0x74,
    R7C5 = 0x75,
    R8C1 = 0x81,
    R8C2 = 0x82,
    R8C3 = 0x83,
    R8C4 = 0x84,
    R8C5 = 0x85,
    NONE = 0xff,
}

impl Switch {
    pub fn to_key(&self) -> Key {
        match self {
            Self::R0C1 => Key::Left,
            Self::R0C2 => Key::Up,
            Self::R0C3 => Key::Down,
            Self::R0C4 => Key::Right,
            Self::R0C5 => Key::Ok,
            Self::R0C6 => Key::Back,
            Self::R1C1 => Key::Home,
            Self::R1C3 => Key::Power,
            Self::R2C1 => Key::Shift,
            Self::R2C2 => Key::Alpha,
            Self::R2C3 => Key::XNT,
            Self::R2C4 => Key::Var,
            Self::R2C5 => Key::Toolbox,
            Self::R2C6 => Key::Delete,
            Self::R3C1 => Key::Euler,
            Self::R3C2 => Key::Ln,
            Self::R3C3 => Key::Log,
            Self::R3C4 => Key::Imaginary,
            Self::R3C5 => Key::Comma,
            Self::R3C6 => Key::Pow,
            Self::R4C1 => Key::Sin,
            Self::R4C2 => Key::Cos,
            Self::R4C3 => Key::Tan,
            Self::R4C4 => Key::Pi,
            Self::R4C5 => Key::Sqrt,
            Self::R4C6 => Key::Square,
            Self::R5C1 => Key::Seven,
            Self::R5C2 => Key::Eight,
            Self::R5C3 => Key::Nine,
            Self::R5C4 => Key::LBracket,
            Self::R5C5 => Key::RBracket,
            Self::R6C1 => Key::Four,
            Self::R6C2 => Key::Five,
            Self::R6C3 => Key::Six,
            Self::R6C4 => Key::Multiply,
            Self::R6C5 => Key::Divide,
            Self::R7C1 => Key::One,
            Self::R7C2 => Key::Two,
            Self::R7C3 => Key::Three,
            Self::R7C4 => Key::Add,
            Self::R7C5 => Key::Subtract,
            Self::R8C1 => Key::Zero,
            Self::R8C2 => Key::Dot,
            Self::R8C3 => Key::EE,
            Self::R8C4 => Key::Ans,
            Self::R8C5 => Key::EXE,
            Self::NONE => Key::NONE,
        }
    }
    pub fn to_key_shift(&self) -> Key {
        match self {
            Self::R2C2 => Key::AlphaLock,
            Self::R2C3 => Key::Cut,
            Self::R2C4 => Key::Copy,
            Self::R2C5 => Key::Paste,
            Self::R2C6 => Key::Clear,
            Self::R3C1 => Key::RSqBracket,
            Self::R3C2 => Key::LSqBracket,
            Self::R3C3 => Key::RCurlyBrace,
            Self::R3C4 => Key::LCurlyBrace,
            Self::R3C5 => Key::Underscore,
            Self::R3C6 => Key::Sto,
            Self::R4C1 => Key::ASin,
            Self::R4C2 => Key::ACos,
            Self::R4C3 => Key::ATan,
            Self::R4C4 => Key::Equals,
            Self::R4C5 => Key::Less,
            Self::R4C6 => Key::Greater,
            _ => self.to_key(),
        }
    }
    pub fn to_key_alpha(&self, shift: bool) -> Key {
        match self {
            Self::R2C3 => Key::Colon,
            Self::R2C4 => Key::SemiColon,
            Self::R2C5 => Key::Quote,
            Self::R2C6 => Key::Percent,
            Self::R3C1 => Key::A,
            Self::R3C2 => Key::B,
            Self::R3C3 => Key::C,
            Self::R3C4 => Key::D,
            Self::R3C5 => Key::E,
            Self::R3C6 => Key::F,
            Self::R4C1 => Key::G,
            Self::R4C2 => Key::H,
            Self::R4C3 => Key::I,
            Self::R4C4 => Key::J,
            Self::R4C5 => Key::K,
            Self::R4C6 => Key::L,
            Self::R5C1 => Key::M,
            Self::R5C2 => Key::N,
            Self::R5C3 => Key::O,
            Self::R5C4 => Key::P,
            Self::R5C5 => Key::Q,
            Self::R6C1 => Key::R,
            Self::R6C2 => Key::S,
            Self::R6C3 => Key::T,
            Self::R6C4 => Key::U,
            Self::R6C5 => Key::V,
            Self::R7C1 => Key::W,
            Self::R7C2 => Key::X,
            Self::R7C3 => Key::Y,
            Self::R7C4 => Key::Z,
            Self::R7C5 => Key::Space,
            Self::R8C1 => Key::Question,
            Self::R8C2 => Key::Exclamation,
            _ => {
                if shift {
                    self.to_key_shift()
                } else {
                    self.to_key()
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum Key {
    Left,
    Up,
    Down,
    Right,
    Ok,
    Back,
    Home,
    Power,
    Shift,
    Alpha,
    AlphaLock,
    XNT,
    Var,
    Toolbox,
    Delete,
    Euler,
    Ln,
    Log,
    Imaginary,
    Comma,
    Pow,
    Sin,
    Cos,
    Tan,
    Pi,
    Sqrt,
    Square,
    Seven,
    Eight,
    Nine,
    LBracket,
    RBracket,
    Four,
    Five,
    Six,
    Multiply,
    Divide,
    One,
    Two,
    Three,
    Add,
    Subtract,
    Zero,
    Dot,
    EE,
    Ans,
    EXE,
    Cut,
    Copy,
    Paste,
    Clear,
    RSqBracket,
    LSqBracket,
    RCurlyBrace,
    LCurlyBrace,
    Underscore,
    Sto,
    ASin,
    ACos,
    ATan,
    Equals,
    Less,
    Greater,
    Colon,
    SemiColon,
    Quote,
    Percent,
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
    Space,
    Question,
    Exclamation,
    NONE,
}

impl From<Key> for char {
    fn from(key: Key) -> char {
        match key {
            Key::XNT => 'x',
            Key::Euler => 'e',
            Key::Imaginary => 'i',
            Key::Pow => '^',
            Key::Zero => '0',
            Key::One => '1',
            Key::Two => '2',
            Key::Three => '3',
            Key::Four => '4',
            Key::Five => '5',
            Key::Six => '6',
            Key::Seven => '7',
            Key::Eight => '8',
            Key::Nine => '9',
            Key::LBracket => '(',
            Key::RBracket => ')',
            Key::RSqBracket => '[',
            Key::LSqBracket => ']',
            Key::RCurlyBrace => '{',
            Key::LCurlyBrace => '}',
            Key::Underscore => '_',
            Key::Equals => '=',
            Key::Less => '<',
            Key::Greater => '>',
            Key::Multiply => '*',
            Key::Divide => '/',
            Key::Add => '+',
            Key::Subtract => '-',
            Key::Colon => ':',
            Key::SemiColon => ';',
            Key::Dot => '.',
            Key::Comma => ',',
            Key::Quote => '"',
            Key::Percent => '%',
            Key::A => 'a',
            Key::B => 'b',
            Key::C => 'c',
            Key::D => 'd',
            Key::E => 'e',
            Key::F => 'f',
            Key::G => 'g',
            Key::H => 'h',
            Key::I => 'i',
            Key::J => 'j',
            Key::K => 'k',
            Key::L => 'l',
            Key::M => 'm',
            Key::N => 'n',
            Key::O => 'o',
            Key::P => 'p',
            Key::Q => 'q',
            Key::R => 'r',
            Key::S => 's',
            Key::T => 't',
            Key::U => 'u',
            Key::V => 'v',
            Key::W => 'w',
            Key::X => 'x',
            Key::Y => 'y',
            Key::Z => 'z',
            Key::Space => ' ',
            Key::Question => '?',
            Key::Exclamation => '!',
            Key::EXE => '\n',
            _ => '\0',
        }
    }
}

struct KeyColumns;

impl KeyColumns {
    fn read() -> u8 {
        let gpioc = unsafe { &(*crate::pac::GPIOC::ptr()) };
        let columns = gpioc.idr.read().bits();
        columns as u8
    }
}

pub struct KeyMatrix;

impl KeyMatrix {
    pub fn init() {
        let rcc = unsafe { &(*crate::pac::RCC::ptr()) };
        let gpioa = unsafe { &(*crate::pac::GPIOA::ptr()) };
        let gpioc = unsafe { &(*crate::pac::GPIOC::ptr()) };

        // Enable GPIOs A, B and C (B for the LED, A & C for the keypad)
        rcc.ahb1enr
            .modify(|_, w| w.gpioaen().set_bit().gpiocen().set_bit());

        // Configure pin for keypad row G (output driven low)
        gpioa.moder.modify(|_, w| {
            w.moder0()
                .output()
                .moder1()
                .output()
                .moder2()
                .output()
                .moder3()
                .output()
                .moder4()
                .output()
                .moder5()
                .output()
                .moder6()
                .output()
                .moder7()
                .output()
                .moder8()
                .output()
        });
        gpioa.odr.modify(|_, w| {
            w.odr0()
                .low()
                .odr1()
                .low()
                .odr2()
                .low()
                .odr3()
                .low()
                .odr4()
                .low()
                .odr5()
                .low()
                .odr6()
                .low()
                .odr7()
                .low()
                .odr8()
                .low()
        });

        gpioc.moder.modify(|_, w| {
            w.moder0()
                .input()
                .moder1()
                .input()
                .moder2()
                .input()
                .moder3()
                .input()
                .moder4()
                .input()
                .moder5()
                .input()
        });
        gpioc.pupdr.modify(|_, w| {
            w.pupdr0()
                .pull_up()
                .pupdr1()
                .pull_up()
                .pupdr2()
                .pull_up()
                .pupdr3()
                .pull_up()
                .pupdr4()
                .pull_up()
                .pupdr5()
                .pull_up()
        });
    }

    pub fn scan(ahb_frequency: u32) -> [u8; 9] {
        let mut state = [
            0b111111, 0b000101, 0b111111, 0b111111, 0b111111, 0b011111, 0b011111, 0b011111,
            0b011111,
        ];

        let gpioa = unsafe { &(*crate::pac::GPIOA::ptr()) };

        let mut delay = cortex_m::delay::Delay::new(
            unsafe { cortex_m::Peripherals::steal().SYST },
            ahb_frequency,
        );

        for (row_pin, row_state) in (0..=8u32).zip(&mut state) {
            gpioa
                .odr
                .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << row_pin)) });
            delay.delay_us(10);
            *row_state &= !KeyColumns::read();
            gpioa
                .odr
                .modify(|r, w| unsafe { w.bits(r.bits() | (1 << row_pin)) });
        }

        state
    }
}

fn state_to_switches(state: [u8; 9]) -> [Switch; 46] {
    let mut keys = [Switch::NONE; 46];
    let mut index = 0;
    for (n, row) in state.iter().enumerate() {
        let start = 0x10 * n as u8;
        for col in [1u8, 2, 4, 8, 16, 32].iter() {
            if let Some(key) = col_to_key(start, *row, *col) {
                keys[index] = key;
                index += 1;
            }
        }
    }
    keys
}

fn col_to_key(start: u8, row: u8, col: u8) -> Option<Switch> {
    let key = match col {
        1 => 1,
        2 => 2,
        4 => 3,
        8 => 4,
        16 => 5,
        32 => 6,
        _ => return None,
    };
    if row & col == col {
        Switch::from_u8(start + key)
    } else {
        None
    }
}

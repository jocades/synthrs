use crate::Hz;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(unused)]
pub enum KeyCode {
    Q = 12,
    W = 13,
    E = 14,
    R = 15,
    T = 17,
    Y = 16,
    U = 32,
    I = 34,
    O = 31,
    P = 35,
    LBracket = 33,
    RBracket = 30,
    BSlash = 42,

    A = 0,
    S = 1,
    D = 2,
    F = 3,
    G = 5,
    H = 4,
    J = 38,
    K = 40,
    L = 37,
    Semi = 41,
    Quote = 39,
    Enter = 36,
    Home = 115,

    Z = 6,
    X = 7,
    C = 8,
    V = 9,
    B = 11,
    N = 45,
    M = 46,
    Comma = 43,
    Dot = 47,
    FSLash = 44,
}

#[inline]
pub fn is_key_down(key: KeyCode) -> bool {
    unsafe extern "C" {
        #[link_name = "is_key_down"]
        fn c_is_key_down(keycode: u16) -> bool;
    }
    unsafe { c_is_key_down(key as u16) }
}

#[derive(Clone, Copy)]
pub struct Key {
    pub code: KeyCode,
    pub freq: Hz,
    pub pressed: bool,
}

pub struct Keyboard {
    pub keys: [Key; 18],
    // map: [Option<usize>; 128], // keycode -> index in `keys`
}

impl Keyboard {
    pub fn new() -> Self {
        /* let drums = [
            kbd::Key {
                code: KeyCode::Z,
                freq: Hz(60.0),
                pressed: false,
            },
            kbd::Key {
                code: KeyCode::X,
                freq: Hz(180.0),
                pressed: false,
            },
            kbd::Key {
                code: KeyCode::C,
                freq: Hz(100.0),
                pressed: false,
            },
        ]; */
        Self {
            keys: [
                Key {
                    code: KeyCode::A,
                    freq: Hz::from_pitch_std(-9),
                    pressed: false,
                },
                Key {
                    code: KeyCode::W,
                    freq: Hz::from_pitch_std(-8),
                    pressed: false,
                },
                Key {
                    code: KeyCode::S,
                    freq: Hz::from_pitch_std(-7),
                    pressed: false,
                },
                Key {
                    code: KeyCode::E,
                    freq: Hz::from_pitch_std(-6),
                    pressed: false,
                },
                Key {
                    code: KeyCode::D,
                    freq: Hz::from_pitch_std(-5),
                    pressed: false,
                },
                Key {
                    code: KeyCode::F,
                    freq: Hz::from_pitch_std(-4),
                    pressed: false,
                },
                Key {
                    code: KeyCode::T,
                    freq: Hz::from_pitch_std(-3),
                    pressed: false,
                },
                Key {
                    code: KeyCode::G,
                    freq: Hz::from_pitch_std(-2),
                    pressed: false,
                },
                Key {
                    code: KeyCode::Y,
                    freq: Hz::from_pitch_std(-1),
                    pressed: false,
                },
                Key {
                    code: KeyCode::H,
                    freq: Hz::from_pitch_std(0),
                    pressed: false,
                },
                Key {
                    code: KeyCode::U,
                    freq: Hz::from_pitch_std(1),
                    pressed: false,
                },
                Key {
                    code: KeyCode::J,
                    freq: Hz::from_pitch_std(2),
                    pressed: false,
                },
                Key {
                    code: KeyCode::K,
                    freq: Hz::from_pitch_std(3),
                    pressed: false,
                },
                Key {
                    code: KeyCode::O,
                    freq: Hz::from_pitch_std(4),
                    pressed: false,
                },
                Key {
                    code: KeyCode::L,
                    freq: Hz::from_pitch_std(5),
                    pressed: false,
                },
                Key {
                    code: KeyCode::P,
                    freq: Hz::from_pitch_std(6),
                    pressed: false,
                },
                Key {
                    code: KeyCode::Semi,
                    freq: Hz::from_pitch_std(7),
                    pressed: false,
                },
                Key {
                    code: KeyCode::Quote,
                    freq: Hz::from_pitch_std(8),
                    pressed: false,
                },
            ],
        }
    }

    #[allow(unused)]
    fn press(&mut self, _keycode: KeyCode) {
        // if let Some(index) = self.map[keycode as usize] {
        //     self.keys[index].pressed = true;
        // }
    }

    #[allow(unused)]
    fn release(&mut self, _keycode: KeyCode) {
        // if let Some(index) = self.map[keycode as usize] {
        //     self.keys[index].pressed = true;
        // }
    }
}

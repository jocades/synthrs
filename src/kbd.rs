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
}

#[inline]
pub fn is_key_down(key: KeyCode) -> bool {
    unsafe extern "C" {
        #[link_name = "is_key_down"]
        fn c_is_key_down(keycode: u16) -> bool;
    }
    unsafe { c_is_key_down(key as u16) }
}

/* struct Keyboard {
    keys: [Key; 4],
    map: [Option<usize>; 128], // keycode -> index in `keys`
}

impl Keyboard {
    fn press(&mut self, keycode: kbd::KeyCode) {
        if let Some(index) = self.map[keycode as usize] {
            self.keys[index].pressed = true;
        }
    }

    fn release(&mut self, keycode: kbd::KeyCode) {
        if let Some(index) = self.map[keycode as usize] {
            self.keys[index].pressed = true;
        }
    }
} */

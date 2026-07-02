#[allow(unused)]
pub mod keys {
    // Цифровой ряд (сверху)
    pub const KEY_1: u8 = 10;
    pub const KEY_2: u8 = 11;
    pub const KEY_3: u8 = 12;
    pub const KEY_4: u8 = 13;
    pub const KEY_5: u8 = 14;
    pub const KEY_6: u8 = 15;
    pub const KEY_7: u8 = 16;
    pub const KEY_8: u8 = 17;
    pub const KEY_9: u8 = 18;
    pub const KEY_0: u8 = 19;

    // Буквы: Верхний ряд
    pub const Q: u8 = 24;
    pub const W: u8 = 25;
    pub const E: u8 = 26;
    pub const R: u8 = 27;
    pub const T: u8 = 28;
    pub const Y: u8 = 29;
    pub const U: u8 = 30;
    pub const I: u8 = 31;
    pub const O: u8 = 32;
    pub const P: u8 = 33;

    // Буквы: Средний ряд (Home row)
    pub const A: u8 = 38;
    pub const S: u8 = 39;
    pub const D: u8 = 40;
    pub const F: u8 = 41;
    pub const G: u8 = 42;
    pub const H: u8 = 43;
    pub const J: u8 = 44;
    pub const K: u8 = 45;
    pub const L: u8 = 46;

    // Буквы: Нижний ряд
    pub const Z: u8 = 52;
    pub const X: u8 = 53;
    pub const C: u8 = 54;
    pub const V: u8 = 55;
    pub const B: u8 = 56;
    pub const N: u8 = 57;
    pub const M: u8 = 58;

    // Системные и управляющие
    pub const ESC: u8 = 9;
    pub const ENTER: u8 = 36;
    pub const SPACE: u8 = 65;
    pub const TAB: u8 = 23;
    pub const BACKSPACE: u8 = 22;

    // Стрелочный блок
    pub const UP: u8 = 111;
    pub const DOWN: u8 = 116;
    pub const LEFT: u8 = 113;
    pub const RIGHT: u8 = 114;

    // Функциональные (F-ряд)
    pub const F1: u8 = 67;
    pub const F2: u8 = 68;
    pub const F3: u8 = 69;
    pub const F4: u8 = 70;
    pub const F5: u8 = 71;
    pub const F6: u8 = 72;
    pub const F7: u8 = 73;
    pub const F8: u8 = 74;
    pub const F9: u8 = 75;
    pub const F10: u8 = 76;
    pub const F11: u8 = 95;
    pub const F12: u8 = 96;
}

#[derive(Clone, Copy, PartialEq)]
pub enum Direction {
    Next,
    Prev
}

/// Действия, которые могут выполнять кеймапы
#[derive(Clone, Copy)]
pub enum Action {
    Spawn(&'static str),
    KillWindows,
    Focus(Direction),
    Quit,
}

/// Перечесление модификаторов
#[allow(unused)]
pub mod key_mods {
    pub const SHIFT: u16 = 1 << 0;
    pub const CAPS: u16 = 1 << 1;
    pub const CTRL: u16 = 1 << 2;
    pub const ALT: u16 = 1 << 3;
    pub const SUPER: u16 = 1 << 6;
}

/// Описание кеймапа
pub struct Keybind {
    pub mods: u16,
    pub keysym: u8,
    pub action: Action,
}

impl Keybind {
    pub const fn new(mods: &[u16], button: u8, action: Action) -> Self {
        let mut mask = 0u16;
        let mut i = 0;
        while i < mods.len() {
            mask |= mods[i];
            i += 1;
        }
        Self {
            mods: mask,
            keysym: button,
            action,
        }
    }
}

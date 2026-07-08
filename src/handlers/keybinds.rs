use x11rb::{connection::Connection, errors::ReplyError, protocol::xproto::ConnectionExt};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Next,
    Prev,
}

/// Действия, которые могут выполнять keybind'ы.
#[derive(Clone, Copy, Debug)]
pub enum WindowAction {
    Spawn(&'static str),
    KillWindows,
    Focus(Direction),
    Quit,
    /// Переключить видимость тега с индексом `0..NUM_TAGS`.
    ToggleTag(usize),
    /// Переместить фокусное окно в тег `0..NUM_TAGS` (single-tag move).
    MoveToTag(usize),
    /// Сфокусировать тег с индексом `0..NUM_TAGS` (показать только его).
    View(usize),
    /// Показать все теги.
    ViewAll,
}

/// Перечисление модификаторов.
#[allow(unused)]
pub mod key_mods {
    pub const SHIFT: u16 = 1 << 0;
    pub const CAPS: u16 = 1 << 1;
    pub const CTRL: u16 = 1 << 2;
    pub const ALT: u16 = 1 << 3;
    pub const SUPER: u16 = 1 << 6;
}

/// Описание keybind'а.
pub struct Keybind {
    pub mods: u16,
    pub keysym: u32,
    pub action: WindowAction,
}

impl Keybind {
    pub const fn new(mods: u16, keysym: u32, action: WindowAction) -> Self {
        Self {
            mods,
            keysym,
            action,
        }
    }
}

pub fn keysym_to_keycode<C: Connection>(
    conn: &C,
    target_keysym: u32,
) -> Result<Option<u8>, ReplyError> {
    let setup = conn.setup();
    let min_kc = setup.min_keycode;
    let max_kc = setup.max_keycode;
    let count = max_kc - min_kc + 1;

    let reply = conn.get_keyboard_mapping(min_kc, count)?.reply()?;
    let keysyms_per_keycode = reply.keysyms_per_keycode as usize;

    for (i, chunk) in reply.keysyms.chunks(keysyms_per_keycode).enumerate() {
        for &sym in chunk {
            if sym == target_keysym {
                let keycode = min_kc + i as u8;
                return Ok(Some(keycode));
            }
        }
    }

    Ok(None)
}

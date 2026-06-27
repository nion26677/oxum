use crate::keybinds::{Action::Quit, key_mods::*, keys};

const MOD: u16 = SUPER;

pub const KEYBINDS: &[Keybind] = &[
    Keybind::new(&[MOD], keys::L, Quit),
    Keybind::new(&[MOD], keys::Q, Spawn("rio"))
];

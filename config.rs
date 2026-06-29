const MASTER_RATIO: f32 = 0.50;
const MOD: u16 = SUPER;

pub const KEYBINDS: &[Keybind] = &[
    Keybind::new(&[MOD, SHIFT], keys::L, Quit),
    Keybind::new(&[MOD], keys::Q, Spawn("rio")),
    Keybind::new(&[MOD], keys::C, KillWindows),
];

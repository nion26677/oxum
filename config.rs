const BORDER_WIDTH: u32 = 2;
const FOCUSED_COLOR: u32 = 0x89b4fa;
const UNFOCUSED_COLOR: u32 = 0x45475a;

// Ширина master панели
const MASTER_RATIO: f32 = 0.50;

// Modkey
const MOD: u16 = SUPER;

pub const KEYBINDS: &[Keybind] = &[
    Keybind::new(&[MOD, SHIFT], keys::L, Quit),
    Keybind::new(&[MOD], keys::Q, Spawn("alacritty")),
    Keybind::new(&[MOD, SHIFT], keys::S, Spawn("flameshot gui")),
    Keybind::new(&[MOD], keys::C, KillWindows),

    Keybind::new(&[MOD], keys::J, Focus(Direction::Next)),
    Keybind::new(&[MOD], keys::K, Focus(Direction::Prev)),
];



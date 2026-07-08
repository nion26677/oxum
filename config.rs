// ========  Global  ==========
const BORDER_WIDTH: u32 = 2;
const FOCUSED_COLOR: u32 = 0x89b4fa;
const UNFOCUSED_COLOR: u32 = 0x45475a;

const MASTER_RATIO: f32 = 0.50;

/// Количество тегов (dwm-стиль: 9 = Super+1..9).
const NUM_TAGS: usize = 9;

// ========== Keybinds ==========
pub const KEYBINDS: &[crate::handlers::keybinds::Keybind] = &[
    // --- Spawn / Quit ---
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER,
        x11_dl::keysym::XK_Q,
        crate::handlers::keybinds::WindowAction::Spawn("kitty"),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER,
        x11_dl::keysym::XK_C,
        crate::handlers::keybinds::WindowAction::KillWindows,
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER | crate::handlers::keybinds::key_mods::SHIFT,
        x11_dl::keysym::XK_L,
        crate::handlers::keybinds::WindowAction::Quit,
    ),

    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER | crate::handlers::keybinds::key_mods::SHIFT,
        x11_dl::keysym::XK_L,
        crate::handlers::keybinds::WindowAction::Spawn("flameshot gui"),
    ),

    // --- Focus cycle (только среди видимых окон активных тегов) ---
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER,
        x11_dl::keysym::XK_J,
        crate::handlers::keybinds::WindowAction::Focus(crate::handlers::keybinds::Direction::Next),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER,
        x11_dl::keysym::XK_K,
        crate::handlers::keybinds::WindowAction::Focus(crate::handlers::keybinds::Direction::Prev),
    ),
    // --- View tag: Super+1..9 — показать только соответствующий тег ---
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER,
        x11_dl::keysym::XK_1,
        crate::handlers::keybinds::WindowAction::View(0),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER,
        x11_dl::keysym::XK_2,
        crate::handlers::keybinds::WindowAction::View(1),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER,
        x11_dl::keysym::XK_3,
        crate::handlers::keybinds::WindowAction::View(2),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER,
        x11_dl::keysym::XK_4,
        crate::handlers::keybinds::WindowAction::View(3),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER,
        x11_dl::keysym::XK_5,
        crate::handlers::keybinds::WindowAction::View(4),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER,
        x11_dl::keysym::XK_6,
        crate::handlers::keybinds::WindowAction::View(5),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER,
        x11_dl::keysym::XK_7,
        crate::handlers::keybinds::WindowAction::View(6),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER,
        x11_dl::keysym::XK_8,
        crate::handlers::keybinds::WindowAction::View(7),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER,
        x11_dl::keysym::XK_9,
        crate::handlers::keybinds::WindowAction::View(8),
    ),
    // --- Toggle tag: Super+Shift+1..9 — добавить/убрать тег из видимых ---
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER | crate::handlers::keybinds::key_mods::SHIFT,
        x11_dl::keysym::XK_1,
        crate::handlers::keybinds::WindowAction::ToggleTag(0),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER | crate::handlers::keybinds::key_mods::SHIFT,
        x11_dl::keysym::XK_2,
        crate::handlers::keybinds::WindowAction::ToggleTag(1),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER | crate::handlers::keybinds::key_mods::SHIFT,
        x11_dl::keysym::XK_3,
        crate::handlers::keybinds::WindowAction::ToggleTag(2),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER | crate::handlers::keybinds::key_mods::SHIFT,
        x11_dl::keysym::XK_4,
        crate::handlers::keybinds::WindowAction::ToggleTag(3),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER | crate::handlers::keybinds::key_mods::SHIFT,
        x11_dl::keysym::XK_5,
        crate::handlers::keybinds::WindowAction::ToggleTag(4),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER | crate::handlers::keybinds::key_mods::SHIFT,
        x11_dl::keysym::XK_6,
        crate::handlers::keybinds::WindowAction::ToggleTag(5),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER | crate::handlers::keybinds::key_mods::SHIFT,
        x11_dl::keysym::XK_7,
        crate::handlers::keybinds::WindowAction::ToggleTag(6),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER | crate::handlers::keybinds::key_mods::SHIFT,
        x11_dl::keysym::XK_8,
        crate::handlers::keybinds::WindowAction::ToggleTag(7),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER | crate::handlers::keybinds::key_mods::SHIFT,
        x11_dl::keysym::XK_9,
        crate::handlers::keybinds::WindowAction::ToggleTag(8),
    ),
    // --- Move window to tag: Super+Ctrl+1..9 ---
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER | crate::handlers::keybinds::key_mods::CTRL,
        x11_dl::keysym::XK_1,
        crate::handlers::keybinds::WindowAction::MoveToTag(0),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER | crate::handlers::keybinds::key_mods::CTRL,
        x11_dl::keysym::XK_2,
        crate::handlers::keybinds::WindowAction::MoveToTag(1),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER | crate::handlers::keybinds::key_mods::CTRL,
        x11_dl::keysym::XK_3,
        crate::handlers::keybinds::WindowAction::MoveToTag(2),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER | crate::handlers::keybinds::key_mods::CTRL,
        x11_dl::keysym::XK_4,
        crate::handlers::keybinds::WindowAction::MoveToTag(3),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER | crate::handlers::keybinds::key_mods::CTRL,
        x11_dl::keysym::XK_5,
        crate::handlers::keybinds::WindowAction::MoveToTag(4),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER | crate::handlers::keybinds::key_mods::CTRL,
        x11_dl::keysym::XK_6,
        crate::handlers::keybinds::WindowAction::MoveToTag(5),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER | crate::handlers::keybinds::key_mods::CTRL,
        x11_dl::keysym::XK_7,
        crate::handlers::keybinds::WindowAction::MoveToTag(6),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER | crate::handlers::keybinds::key_mods::CTRL,
        x11_dl::keysym::XK_8,
        crate::handlers::keybinds::WindowAction::MoveToTag(7),
    ),
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER | crate::handlers::keybinds::key_mods::CTRL,
        x11_dl::keysym::XK_9,
        crate::handlers::keybinds::WindowAction::MoveToTag(8),
    ),
    // --- View all tags: Super+0 ---
    crate::handlers::keybinds::Keybind::new(
        crate::handlers::keybinds::key_mods::SUPER,
        x11_dl::keysym::XK_0,
        crate::handlers::keybinds::WindowAction::ViewAll,
    ),
];

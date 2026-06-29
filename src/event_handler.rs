use log::info;
use x11rb::{
    CURRENT_TIME,
    connection::Connection,
    protocol::xproto::{
        AtomEnum, ClientMessageData, ClientMessageEvent, ConnectionExt, EventMask, Screen,
    },
    rust_connection::RustConnection,
};

use crate::keybinds::Action;

const PROP_LIST_LEN: u32 = 100;
const POINTER_ROOT: u32 = 1;

/// Обработать действие, привязанное к окну.
pub fn action_for_window(
    conn: &RustConnection,
    action: Action,
    screen: &Screen,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        Action::Quit => quit(),
        Action::Spawn(command) => spawn_program(command),
        Action::KillWindows => kill_focused_window(conn, screen)?,
    }
    Ok(())
}

/// Завершить работу оконного менеджера.
fn quit() {
    println!("Закрытие оконного менеджера");
    std::process::exit(0);
}

/// Запустить внешнюю программу через оболочку.
fn spawn_program(command: &'static str) {
    #[allow(clippy::zombie_processes)]
    std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .spawn()
        .expect("Не удалось запустить команду");
}

/// Закрыть (или принудительно убить) окно, находящееся в фокусе.
fn kill_focused_window(
    conn: &RustConnection,
    screen: &Screen,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Получен ивент на удаление окна");

    // 1. Один раз запрашиваем нужные атомы и переиспользуем их.
    let atoms = WmAtoms::intern(conn)?;

    // 2. Определяем целевое окно: фокус или дочернее окно под курсором.
    let Some(target) = resolve_focused_window(conn, screen)? else {
        println!("Фокус на Root или отсутствует. Нечего закрывать.");
        return Ok(());
    };

    // 3. Поднимаемся по дереву до top-level окна.
    let Some(toplevel) = find_toplevel_window(conn, screen.root, target)? else {
        println!("Не удалось найти top-level родителя, жёстко убиваем фокусное окно");
        conn.kill_client(target)?;
        conn.flush()?;
        return Ok(());
    };
    println!("Окно найдено: {toplevel}");

    // 4. Проверяем поддержку WM_DELETE_WINDOW у клиента.
    if window_supports_wm_delete(conn, toplevel, &atoms)? {
        println!("Окно поддерживает WM_DELETE_WINDOW, отправляем ClientMessage");
        send_wm_delete(conn, toplevel, &atoms)?;
    } else {
        println!("Окно НЕ поддерживает протокол закрытия. Применяем kill_client");
        conn.kill_client(toplevel)?;
    }

    conn.flush()?;
    Ok(())
}

/// Атомы, которые нам нужны для закрытия окна.
struct WmAtoms {
    wm_protocols: u32,
    wm_delete_window: u32,
}

impl WmAtoms {
    fn intern(conn: &RustConnection) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            wm_protocols: conn.intern_atom(false, b"WM_PROTOCOLS")?.reply()?.atom,
            wm_delete_window: conn.intern_atom(false, b"WM_DELETE_WINDOW")?.reply()?.atom,
        })
    }
}

/// Id окна, к которому применять действие:
/// окно в фокусе, либо дочернее окно под указателем (если фокус = PointerRoot).
fn resolve_focused_window(
    conn: &RustConnection,
    screen: &Screen,
) -> Result<Option<u32>, Box<dyn std::error::Error>> {
    let focus_reply = conn.get_input_focus()?.reply()?;
    let mut focused = focus_reply.focus;

    // Если фокус привязан к курсору мыши — берём окно, на которое он указывает.
    if focused == POINTER_ROOT
        && let Ok(pointer) = conn.query_pointer(screen.root)?.reply()
        && pointer.child != 0
    {
        focused = pointer.child;
        println!("Фокус был PointerRoot. Мышь над окном: {focused}");
    }

    // 0 или 1 — Root/None: закрывать нечего.
    Ok((focused > 1).then_some(focused))
}

/// Поддерживает ли окно протокол WM_DELETE_WINDOW.
fn window_supports_wm_delete(
    conn: &RustConnection,
    window: u32,
    atoms: &WmAtoms,
) -> Result<bool, Box<dyn std::error::Error>> {
    let reply = conn
        .get_property(
            false,
            window,
            atoms.wm_protocols,
            AtomEnum::ATOM,
            0,
            PROP_LIST_LEN,
        )?
        .reply();

    let Ok(reply) = reply else {
        return Ok(false);
    };

    let supports = reply
        .value32()
        .is_some_and(|mut iter| iter.any(|a| a == atoms.wm_delete_window));
    Ok(supports)
}

/// Отправить клиенту WM_DELETE_WINDOW ClientMessage.
fn send_wm_delete(
    conn: &RustConnection,
    window: u32,
    atoms: &WmAtoms,
) -> Result<(), Box<dyn std::error::Error>> {
    let event = ClientMessageEvent {
        response_type: x11rb::protocol::xproto::CLIENT_MESSAGE_EVENT,
        format: 32,
        sequence: 0,
        window,
        type_: atoms.wm_protocols,
        data: ClientMessageData::from([atoms.wm_delete_window, CURRENT_TIME, 0, 0, 0]),
    };
    conn.send_event(false, window, EventMask::NO_EVENT, event)?;
    Ok(())
}

/// Подняться по дереву окон до top-level (родитель == root или 0).
fn find_toplevel_window(
    conn: &impl Connection,
    root: u32,
    mut win: u32,
) -> Result<Option<u32>, Box<dyn std::error::Error>> {
    loop {
        let tree = conn.query_tree(win)?.reply()?;
        if tree.parent == root || tree.parent == 0 {
            return Ok(Some(win));
        }
        win = tree.parent;
    }
}

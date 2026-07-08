use log::{debug, error, info};
use x11rb::{
    CURRENT_TIME,
    connection::Connection,
    protocol::xproto::{
        AtomEnum, ChangeWindowAttributesAux, ClientMessageData, ClientMessageEvent, ConnectionExt,
        EventMask, InputFocus,
    },
    rust_connection::RustConnection,
};

use crate::core::client::WmState;
use crate::handlers::keybinds::{Direction, WindowAction};
use crate::{FOCUSED_COLOR, UNFOCUSED_COLOR};

const PROP_LIST_LEN: u32 = 100;
const POINTER_ROOT: u32 = 1;

/// Диспетчер действий, привязанных к keybind'ам.
pub fn action_for_window(
    conn: &RustConnection,
    action: WindowAction,
    wm: &mut WmState,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        WindowAction::Quit => quit(),
        WindowAction::Spawn(command) => spawn_program(command),
        WindowAction::KillWindows => kill_focused_window(conn)?,
        WindowAction::Focus(direction) => cycle_focus(conn, wm, direction)?,
        WindowAction::ToggleTag(i) => toggle_tag(conn, wm, i)?,
        WindowAction::MoveToTag(i) => move_to_tag(conn, wm, i)?,
        WindowAction::View(i) => view_tag(conn, wm, i)?,
        WindowAction::ViewAll => view_all_tags(conn, wm)?,
    }
    Ok(())
}

/// Завершить работу оконного менеджера.
fn quit() {
    debug!("Закрытие оконного менеджера");
    std::process::exit(0);
}

/// Запустить внешнюю программу через оболочку.
fn spawn_program(command: &'static str) {
    #[allow(clippy::zombie_processes)]
    match std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .spawn()
    {
        Ok(_) => {}
        Err(e) => error!("Не удалось запустить процесс: {}", e),
    }
}

/// Закрыть (или принудительно убить) окно, находящееся в фокусе.
fn kill_focused_window(conn: &RustConnection) -> Result<(), Box<dyn std::error::Error>> {
    info!("Получен ивент на удаление окна");

    let atoms = WmAtoms::intern(conn)?;

    let Some(win) = resolve_focused_window(conn)? else {
        debug!("Фокус на Root или отсутствует. Нечего закрывать.");
        return Ok(());
    };

    let Some(toplevel_win) = find_toplevel_window(conn, win)? else {
        info!("Не удалось найти top-level родителя, жёстко убиваем фокусное окно");
        conn.kill_client(win)?;
        conn.flush()?;
        return Ok(());
    };
    debug!("Окно найдено: {toplevel_win}");

    if window_supports_wm_delete(conn, toplevel_win, &atoms)? {
        info!("Окно поддерживает WM_DELETE_WINDOW, отправляем ClientMessage");
        send_wm_delete(conn, toplevel_win, &atoms)?;
    } else {
        info!("Окно НЕ поддерживает протокол закрытия. Применяем kill_client");
        conn.kill_client(toplevel_win)?;
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
) -> Result<Option<u32>, Box<dyn std::error::Error>> {
    let focus_reply = conn.get_input_focus()?.reply()?;
    let mut focused = focus_reply.focus;

    if focused == POINTER_ROOT
        && let Some(screen) = conn.setup().roots.first()
        && let Ok(pointer) = conn.query_pointer(screen.root)?.reply()
        && pointer.child != 0
    {
        focused = pointer.child;
        info!("Фокус был PointerRoot. Мышь над окном: {focused}");
    }

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

/// Подняться по дереву окон до top-level (родитель == root).
fn find_toplevel_window(
    conn: &impl Connection,
    mut win: u32,
) -> Result<Option<u32>, Box<dyn std::error::Error>> {
    loop {
        let tree = conn.query_tree(win)?.reply()?;
        if tree.parent == 0 {
            return Ok(None);
        }
        if tree.parent == tree.root {
            return Ok(Some(win));
        }
        win = tree.parent;
    }
}

/// Циклически перенести фокус на следующее/предыдущее окно среди видимых.
fn cycle_focus(
    conn: &RustConnection,
    wm: &WmState,
    direction: Direction,
) -> Result<(), Box<dyn std::error::Error>> {
    let windows = wm.visible_windows();
    if windows.is_empty() {
        return Ok(());
    }

    let current = conn.get_input_focus()?.reply()?.focus;
    let len = windows.len();

    let next_idx = match windows.iter().position(|&w| w == current) {
        Some(i) => match direction {
            Direction::Next => (i + 1) % len,
            Direction::Prev => (i + len - 1) % len,
        },
        None => 0,
    };

    let next_win = windows[next_idx];
    set_focus_with_notify(conn, current, next_win)?;
    Ok(())
}

// ============ Tag-related actions ============

/// Переключить видимость тега `i` (toggle, dwm-style).
fn toggle_tag(
    conn: &RustConnection,
    wm: &mut WmState,
    i: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    if i >= wm.tags.len() {
        return Ok(());
    }
    wm.tags.toggle(i);
    wm.tags.set_focused(i);
    apply_tag_change(conn, wm)
}

/// Сфокусировать (показать только) тег `i`.
fn view_tag(
    conn: &RustConnection,
    wm: &mut WmState,
    i: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    if i >= wm.tags.len() {
        return Ok(());
    }
    wm.tags.view(i);
    wm.tags.set_focused(i);
    apply_tag_change(conn, wm)
}

/// Показать все теги разом.
fn view_all_tags(
    conn: &RustConnection,
    wm: &mut WmState,
) -> Result<(), Box<dyn std::error::Error>> {
    wm.tags.view_all();
    apply_tag_change(conn, wm)
}

/// Переместить фокусное окно в тег `i` (single-tag semantics).
fn move_to_tag(
    conn: &RustConnection,
    wm: &mut WmState,
    i: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    if i >= wm.tags.len() {
        return Ok(());
    }
    let Some(win) = resolve_focused_window(conn)? else {
        debug!("Нет фокусного окна для перемещения");
        return Ok(());
    };
    wm.tags.move_to(win, i);
    // Показать целевой тег (вычислит, что нужно скрыть/показать).
    wm.tags.view(i);
    wm.tags.set_focused(i);
    apply_tag_change(conn, wm)
}

/// Атомы, которые нам нужны для переключения фокуса через ICCCM/EWMH.
struct FocusAtoms {
    wm_take_focus: u32,
    wm_protocols: u32,
    net_active_window: u32,
}

impl FocusAtoms {
    fn intern(conn: &RustConnection) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            wm_protocols: conn.intern_atom(false, b"WM_PROTOCOLS")?.reply()?.atom,
            wm_take_focus: conn.intern_atom(false, b"WM_TAKE_FOCUS")?.reply()?.atom,
            net_active_window: conn
                .intern_atom(false, b"_NET_ACTIVE_WINDOW")?
                .reply()?
                .atom,
        })
    }

    /// Поддерживает ли окно `WM_TAKE_FOCUS` (ICCCM-протокол фокуса).
    fn window_takes_focus(
        &self,
        conn: &RustConnection,
        window: u32,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let reply = conn
            .get_property(false, window, self.wm_protocols, AtomEnum::ATOM, 0, 100)?
            .reply();
        let Ok(reply) = reply else {
            return Ok(false);
        };
        Ok(reply
            .value32()
            .is_some_and(|mut iter| iter.any(|a| a == self.wm_take_focus)))
    }
}

/// Сфокусировать окно `next` с уведомлением.
///
/// Проблема: многие клиенты (alacritty, kitty, gtk с `Focus`/`WM_TAKE_FOCUS`)
/// **отказываются** принимать фокус через `XSetInputFocus` напрямую и
/// требуют, чтобы WM послал им `ClientMessage` с `WM_TAKE_FOCUS`.
/// Только тогда клиент сам вызовет `XSetInputFocus` на себя, и X-сервер
/// пришлёт `FocusIn` — на который среагирует наш `focus_in` хендлер.
///
/// Стратегия:
/// 1) Всегда `XSetInputFocus` — это работает для клиентов без `WM_TAKE_FOCUS`.
/// 2) Если окно поддерживает `WM_TAKE_FOCUS` — шлём ему `ClientMessage`.
///    Клиент сам установит себе фокус, и X пришлёт `FocusIn`.
/// 3) `SetInputFocus` на root с `PointerRoot` как fallback — на случай
///    если клиент по какой-то причине не среагировал.
fn set_focus_with_notify(
    conn: &RustConnection,
    prev: u32,
    next: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1) Попытка прямой установки фокуса.
    conn.set_input_focus(InputFocus::POINTER_ROOT, next, CURRENT_TIME)?;

    // 2) Если клиент принимает фокус через WM_TAKE_FOCUS — шлём ему запрос.
    //    Это нужно для alacritty/kitty/gtk-приложений с focus-stealing prevent.
    let atoms = FocusAtoms::intern(conn)?;
    if atoms.window_takes_focus(conn, next)? {
        let event = ClientMessageEvent {
            response_type: x11rb::protocol::xproto::CLIENT_MESSAGE_EVENT,
            format: 32,
            sequence: 0,
            window: next,
            type_: atoms.wm_protocols,
            data: ClientMessageData::from([atoms.wm_take_focus, CURRENT_TIME, 0, 0, 0]),
        };
        conn.send_event(false, next, EventMask::NO_EVENT, event)?;
    }

    // 3) EWMH: на всякий случай шлём и _NET_ACTIVE_WINDOW.
    let event = ClientMessageEvent {
        response_type: x11rb::protocol::xproto::CLIENT_MESSAGE_EVENT,
        format: 32,
        sequence: 0,
        window: next,
        type_: atoms.net_active_window,
        data: ClientMessageData::from([2, CURRENT_TIME, 0, 0, 0]),
    };
    let _ = conn.send_event(false, next, EventMask::NO_EVENT, event);

    // 4) Страховка: перекрашиваем border_pixel.
    if prev != next && prev > 1 {
        let aux = ChangeWindowAttributesAux::new().border_pixel(UNFOCUSED_COLOR);
        let _ = conn.change_window_attributes(prev, &aux);
    }
    let aux = ChangeWindowAttributesAux::new().border_pixel(FOCUSED_COLOR);
    let _ = conn.change_window_attributes(next, &aux);

    conn.flush()?;
    Ok(())
}

/// Общая логика после изменения видимости тегов: спрятать невидимые
/// через `unmap_window`, показать новые видимые через `map_window`,
/// переразложить тайлинг, поставить фокус.
///
/// Используем настоящие `unmap`/`map` (dwm-style), а не «прячем за экран».
/// Это даёт оптимизацию: X-сервер освобождает ресурсы для скрытых окон.
/// Чтобы избежать рекурсии (наш `unmap` → `UnmapNotify` → `unmap_notify`
/// не должен удалить окно из тегов), помечаем прячемые окна в
/// `wm.unmapping` ПЕРЕД отправкой команды.
fn apply_tag_change(
    conn: &RustConnection,
    wm: &mut WmState,
) -> Result<(), Box<dyn std::error::Error>> {
    let visible: std::collections::HashSet<u32> = wm.visible_windows().into_iter().collect();
    let all_managed = wm.tags.all_managed();

    // 1) Скрыть окна, чей тег стал неактивен.
    for win in all_managed.iter() {
        if !visible.contains(win) {
            wm.unmapping.insert(*win);
            if let Err(e) = conn.unmap_window(*win) {
                error!("Не удалось скрыть окно {}: {}", win, e);
                wm.unmapping.remove(win);
            }
        }
    }

    // 2) Показать окна, которые должны быть видны, но сейчас скрыты.
    //    Помечаем в `mapping`, чтобы `map_request` не пытался их
    //    повторно регистрировать и не рекурсил.
    for win in visible.iter() {
        if !wm.visible_managed().contains(win) {
            wm.mapping.insert(*win);
            if let Err(e) = conn.map_window(*win) {
                error!("Не удалось показать окно {}: {}", win, e);
                wm.mapping.remove(win);
            }
        }
    }

    conn.flush()?;

    // 3) Переразложить тайлинг видимых окон.
    wm.arrange(conn)?;

    // 4) Поставить фокус.
    let prev_focus = conn
        .get_input_focus()?
        .reply()
        .map(|r| r.focus)
        .unwrap_or(0);
    if let Some(last) = wm.visible_windows().last().copied() {
        if let Err(e) = set_focus_with_notify(conn, prev_focus, last) {
            debug!("Не удалось сменить фокус: {}", e);
        }
    }
    Ok(())
}

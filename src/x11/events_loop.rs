use log::{debug, info};
use x11rb::{
    CURRENT_TIME,
    connection::Connection,
    protocol::xproto::{
        ChangeWindowAttributesAux, ConfigureRequestEvent, ConfigureWindowAux, ConnectionExt,
        EventMask, InputFocus, KeyPressEvent, MapRequestEvent, UnmapNotifyEvent,
    },
    rust_connection::RustConnection,
};

use crate::core::client::WmState;
use crate::handlers::event_handler::action_for_window;
use crate::{BORDER_WIDTH, FOCUSED_COLOR, KEYBINDS, UNFOCUSED_COLOR};

/// Главный цикл обработки событий X11.
pub fn run(conn: &RustConnection, wm: &mut WmState) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let event = conn.wait_for_event()?;

        match event {
            x11rb::protocol::Event::ConfigureRequest(e) => configure_request(conn, e, wm)?,
            x11rb::protocol::Event::MapRequest(e) => map_request(conn, e, wm)?,
            x11rb::protocol::Event::UnmapNotify(e) => unmap_notify(conn, e, wm)?,
            x11rb::protocol::Event::KeyPress(e) => key_press(conn, e, wm)?,
            x11rb::protocol::Event::FocusIn(e) => focus_in(conn, e, wm)?,
            x11rb::protocol::Event::FocusOut(e) => focus_out(conn, e, wm)?,
            _ => {}
        }
    }
}

/// `ConfigureRequest`: клиент просит изменить геометрию.
pub fn configure_request(
    conn: &RustConnection,
    e: ConfigureRequestEvent,
    wm: &mut WmState,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("ConfigureRequest от {}", e.window);

    let managed = wm.is_managed(e.window);
    let aux = ConfigureWindowAux::from_configure_request(&e);
    conn.configure_window(e.window, &aux)?;

    if managed {
        wm.arrange(conn)?;
        conn.flush()?;
    }
    Ok(())
}

/// `MapRequest`: новое окно хочет показаться, или WM сам запросил `map_window`
/// при смене тега (тогда окно уже в `wm.mapping` и его не нужно регистрировать).
pub fn map_request(
    conn: &RustConnection,
    e: MapRequestEvent,
    wm: &mut WmState,
) -> Result<(), Box<dyn std::error::Error>> {
    let win = e.window;

    // Случай 1: WM сам запросил `map_window` при смене тега.
    // Окно уже зарегистрировано и настроено — просто переразложим тайлинг.
    if wm.mapping.remove(&win) {
        info!(
            "MapRequest для уже известного окна {} (возврат из тега)",
            win
        );
        wm.arrange(conn)?;
        if let Some(&last_win) = wm.visible_windows().last() {
            conn.set_input_focus(InputFocus::POINTER_ROOT, last_win, CURRENT_TIME)?;
            let aux = ChangeWindowAttributesAux::new().border_pixel(FOCUSED_COLOR);
            let _ = conn.change_window_attributes(last_win, &aux);
        }
        conn.flush()?;
        return Ok(());
    }

    // Случай 2: новое окно от клиента.
    info!("MapRequest от нового окна {}", win);
    wm.tags.add_window(win);

    // 1) Раскладываем тайлинг.
    wm.arrange(conn)?;

    // 2) Border + event_mask.
    conn.configure_window(win, &ConfigureWindowAux::new().border_width(BORDER_WIDTH))?;
    let aux = ChangeWindowAttributesAux::new()
        .event_mask(EventMask::STRUCTURE_NOTIFY | EventMask::FOCUS_CHANGE)
        .border_pixel(UNFOCUSED_COLOR);
    conn.change_window_attributes(win, &aux)?;

    // 3) Показываем окно.
    conn.map_window(win)?;

    // 4) Фокус — на последнее видимое окно.
    if let Some(&last_win) = wm.visible_windows().last() {
        conn.set_input_focus(InputFocus::POINTER_ROOT, last_win, CURRENT_TIME)?;
        let aux = ChangeWindowAttributesAux::new().border_pixel(FOCUSED_COLOR);
        let _ = conn.change_window_attributes(last_win, &aux);
    }

    conn.flush()?;
    info!(
        "Окно {} добавлено в тег {}. Видимых окон: {}",
        win,
        wm.tags.focused(),
        wm.visible_windows().len()
    );
    Ok(())
}

/// `UnmapNotify`: окно закрыто клиентом или временно скрыто WM'ом.
///
/// Различаем источник через `wm.unmapping`:
/// - если окно в `unmapping` — это WM его временно спрятал (смена тега),
///   из тегов **не удаляем**, просто убираем из `unmapping`.
/// - иначе — клиент сам закрылся, удаляем из тегов и пересчитываем тайлинг.
pub fn unmap_notify(
    conn: &RustConnection,
    e: UnmapNotifyEvent,
    wm: &mut WmState,
) -> Result<(), Box<dyn std::error::Error>> {
    if wm.unmapping.remove(&e.window) {
        // WM сам спрятал окно при смене тега — оно остаётся в `tags`.
        return Ok(());
    }

    if wm.tags.remove_window(e.window) {
        info!(
            "Окно {} вышло из под управления. Пересчитываем тайлинг.",
            e.window
        );

        wm.arrange(conn)?;

        if let Some(&last_win) = wm.visible_windows().last() {
            conn.set_input_focus(InputFocus::POINTER_ROOT, last_win, CURRENT_TIME)?;
        }
        conn.flush()?;
    }
    Ok(())
}

/// `KeyPress`: ищем подходящий keybind и выполняем действие.
pub fn key_press(
    conn: &RustConnection,
    e: KeyPressEvent,
    wm: &mut WmState,
) -> Result<(), Box<dyn std::error::Error>> {
    let clean_state = u16::from(e.state) & !(0x0002 | 0x0010);
    debug!("Кнопка-{} Модификации-{:?}", e.detail, e.state);

    for bind in KEYBINDS.iter() {
        let keycode = crate::handlers::keybinds::keysym_to_keycode(&conn, bind.keysym)
            .ok()
            .flatten();
        if keycode == Some(e.detail) && bind.mods == clean_state {
            info!("Нажата клавиша: {:?}", bind.action);
            action_for_window(conn, bind.action, wm)?;
            break;
        }
    }
    Ok(())
}

/// `FocusIn`: подсветить рамку цветом фокуса.
pub fn focus_in(
    conn: &RustConnection,
    e: x11rb::protocol::xproto::FocusInEvent,
    wm: &WmState,
) -> Result<(), Box<dyn std::error::Error>> {
    if wm.is_managed(e.event) {
        let aux = ChangeWindowAttributesAux::new().border_pixel(FOCUSED_COLOR);
        conn.change_window_attributes(e.event, &aux)?;
        conn.flush()?;
    }
    Ok(())
}

/// `FocusOut`: вернуть серый цвет неактивной рамки.
pub fn focus_out(
    conn: &RustConnection,
    e: x11rb::protocol::xproto::FocusOutEvent,
    wm: &WmState,
) -> Result<(), Box<dyn std::error::Error>> {
    if wm.is_managed(e.event) {
        let aux = ChangeWindowAttributesAux::new().border_pixel(UNFOCUSED_COLOR);
        conn.change_window_attributes(e.event, &aux)?;
        conn.flush()?;
    }
    Ok(())
}

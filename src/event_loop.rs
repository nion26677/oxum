use crate::{
    BORDER_WIDTH, FOCUSED_COLOR, KEYBINDS, SCREEN_SIZE_HEIGHT, SCREEN_SIZE_WIDTH, TILING, UNFOCUSED_COLOR, WORKSPACE, event_handler::action_for_window
};
use log::{debug, info};
use x11rb::{
    CURRENT_TIME,
    connection::Connection,
    protocol::xproto::{
        ChangeWindowAttributesAux, ConfigureRequestEvent, ConfigureWindowAux, ConnectionExt,
        EventMask, InputFocus, KeyPressEvent, MapRequestEvent, Screen, UnmapNotifyEvent,
    },
    rust_connection::RustConnection,
};
/// Создаёт конфигурацию для настройки окна
pub fn configure_windows(
    conn: &RustConnection,
    e: ConfigureRequestEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("ConfigureRequest от {}", e.window);

    let workspace = WORKSPACE.lock().unwrap();

    // Заполняем aux параметрами, предложенными композитором
    let aux = ConfigureWindowAux::from_configure_request(&e); 

    conn.configure_window(e.window, &aux)?;
    if workspace.windows.contains(&e.window) {
        TILING.arrange(
            &conn,
            *SCREEN_SIZE_WIDTH.lock().unwrap(),
            *SCREEN_SIZE_HEIGHT.lock().unwrap(),
            &workspace,
        )?;
        conn.flush()?;
    }
    Ok(())
}

pub fn map_request(
    conn: &RustConnection,
    e: MapRequestEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    let win = e.window;

    let mut workspace = WORKSPACE.lock().unwrap();

    if !workspace.windows.contains(&win) {
        workspace.windows.push(win);
    }

    conn.configure_window(win, &ConfigureWindowAux::new().border_width(BORDER_WIDTH))?;

    let aux = ChangeWindowAttributesAux::new()
        .event_mask(EventMask::STRUCTURE_NOTIFY | EventMask::FOCUS_CHANGE)
        .border_pixel(UNFOCUSED_COLOR);
    conn.change_window_attributes(win, &aux)?;

    conn.map_window(win)?;

    TILING.arrange(
        &conn,
        *SCREEN_SIZE_WIDTH.lock().unwrap(),
        *SCREEN_SIZE_HEIGHT.lock().unwrap(),
        &workspace,
    )?;

    conn.set_input_focus(InputFocus::POINTER_ROOT, win, CURRENT_TIME)?;
    conn.flush()?;
    Ok(())
}

pub fn window_remover(
    conn: &RustConnection,
    e: UnmapNotifyEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    let client_win = e.window;
    let mut workspace = WORKSPACE.lock().unwrap();

    if let Some(pos) = workspace.windows.iter().position(|&w| w == client_win) {
        workspace.windows.remove(pos);
        info!(
            "Окно {} вышло из под управления. Пересчитываем тайлинг.",
            client_win
        );

        TILING.arrange(
            &conn,
            *SCREEN_SIZE_WIDTH.lock().unwrap(),
            *SCREEN_SIZE_HEIGHT.lock().unwrap(),
            &workspace,
        )?;
        if let Some(&last_win) = workspace.windows.last() {
            conn.set_input_focus(InputFocus::POINTER_ROOT, last_win, CURRENT_TIME)?;
        }
        conn.flush()?;
    }
    Ok(())
}

pub fn key_request(
    conn: &RustConnection,
    e: KeyPressEvent,
    screen: &Screen,
) -> Result<(), Box<dyn std::error::Error>> {
    // Клавиши для игнорирования CapsLock | NumLock
    let clean_state = u16::from(e.state) & !(0x0002 | 0x0010);
    debug!("Кнопка-{} Модификации-{:?}", e.detail, e.state);

    if let Some(bind) = KEYBINDS
        .iter()
        .find(|b| b.keysym == e.detail && b.mods == clean_state)
    {
        action_for_window(conn, bind.action, screen)?;
    }
    Ok(())
}

// Создание рамки окну для окна в фокуса
pub fn focus_in(
    conn: &RustConnection,
    e: x11rb::protocol::xproto::FocusInEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    let workspace = WORKSPACE.lock().unwrap();
    if workspace.windows.contains(&e.event) {
        let aux = ChangeWindowAttributesAux::new().border_pixel(FOCUSED_COLOR);
        conn.change_window_attributes(e.event, &aux)?;
        conn.flush()?;
    }
    Ok(())
}

// Создание рамки окну без фокуса
pub fn focus_out(
    conn: &RustConnection,
    e: x11rb::protocol::xproto::FocusOutEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    let workspace = WORKSPACE.lock().unwrap();
    if workspace.windows.contains(&e.event) {
        let aux = ChangeWindowAttributesAux::new().border_pixel(UNFOCUSED_COLOR);
        conn.change_window_attributes(e.event, &aux)?;
        conn.flush()?;
    }
    Ok(())
}

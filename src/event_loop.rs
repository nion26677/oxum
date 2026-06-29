use crate::{
    KEYBINDS, SCREEN_SIZE_HEIGHT, SCREEN_SIZE_WIDTH, TILING, WORKSPACE,
    event_handler::action_for_window,
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
    let aux = ConfigureWindowAux::new();
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
    let client_win = e.window;

    let mut workspace = WORKSPACE.lock().unwrap();

    if !workspace.windows.contains(&client_win) {
        workspace.windows.push(client_win);
    }

    let aux = ChangeWindowAttributesAux::new()
        .event_mask(EventMask::STRUCTURE_NOTIFY | EventMask::FOCUS_CHANGE);
    conn.change_window_attributes(client_win, &aux)?;

    conn.map_window(client_win)?;

    TILING.arrange(
        &conn,
        *SCREEN_SIZE_WIDTH.lock().unwrap(),
        *SCREEN_SIZE_HEIGHT.lock().unwrap(),
        &workspace,
    )?;

    conn.set_input_focus(InputFocus::POINTER_ROOT, client_win, CURRENT_TIME)?;
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
    // Клавиши для игнорирования
    let clean_state = u16::from(e.state) & !(0x0002 | 0x0010);
    debug!("Кнопка-{} Модификации-{:?}", e.detail, e.state);

    if let Some(bind) = KEYBINDS
        .iter()
        .find(|b| b.button == e.detail && b.mods == clean_state)
    {
        action_for_window(&conn, bind.action, screen)?;
    }
    Ok(())
}

use std::sync::{LazyLock, Mutex};

use log::error;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt, GrabMode, ModMask};

mod event_handler;
mod event_loop;
mod keybinds;
mod tiling;

use crate::event_loop::{configure_windows, focus_in, focus_out, key_request, map_request, window_remover};
use crate::keybinds::Direction;
use crate::keybinds::{Keybind, Action::*, key_mods::*, keys};
use crate::tiling::{TilingType, Workspace};

// File configuration
include!("../config.rs");

// Окна на на столе
pub static WORKSPACE: LazyLock<Mutex<Workspace>> =
    LazyLock::new(|| Mutex::new(Workspace::default()));

pub static SCREEN_SIZE_WIDTH: LazyLock<Mutex<u16>> = LazyLock::new(|| Mutex::new(u16::default()));
pub static SCREEN_SIZE_HEIGHT: LazyLock<Mutex<u16>> = LazyLock::new(|| Mutex::new(u16::default()));

// Тип тайлинга
pub static TILING: TilingType = TilingType::Stack;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Логирование для отладки
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    let (conn, screen_sum) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_sum];

    // Получение размеров экрана
    *SCREEN_SIZE_WIDTH.lock()? = screen.width_in_pixels;
    *SCREEN_SIZE_HEIGHT.lock()? = screen.height_in_pixels;

    // Попытка захвата экрана для проверки запускается ли сессия
    let change = x11rb::protocol::xproto::ChangeWindowAttributesAux::default().event_mask(
        x11rb::protocol::xproto::EventMask::SUBSTRUCTURE_REDIRECT
            | x11rb::protocol::xproto::EventMask::SUBSTRUCTURE_NOTIFY,
    );

    let result = conn.change_window_attributes(screen.root, &change)?.check();

    // Проверка на запуск X11 сесcии
    if result.is_err() {
        error!("Ошибка запуска wm. Возможно уже запущенна другая X11 сессия");
        std::process::exit(1);
    }

    let keybinds = KEYBINDS;

    for bind in keybinds {
        conn.grab_key(
            true,
            screen.root,
            ModMask::from(bind.mods),
            bind.keysym,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
        )?;
    }

    conn.flush()?;

    loop {
        let event = conn.wait_for_event()?;

        match event {
            x11rb::protocol::Event::ConfigureRequest(e) => configure_windows(&conn, e)?,

            // Разрешение на отображение
            x11rb::protocol::Event::MapRequest(e) => map_request(&conn, e)?,

            x11rb::protocol::Event::UnmapNotify(e) => window_remover(&conn, e)?,

            // Обработка событий клавишь
            x11rb::protocol::Event::KeyPress(e) => key_request(&conn, e, screen)?,

            x11rb::protocol::Event::FocusIn(e) => focus_in(&conn, e)?,
            x11rb::protocol::Event::FocusOut(e) => focus_out(&conn, e)?,

            _ => {}
        }
    }
}

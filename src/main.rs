use x11rb::{connection::Connection};
use x11rb::protocol::xproto::{ConfigureWindowAux, ConnectionExt, GrabMode, ModMask} ;
mod keybinds;
use crate::keybinds::Action::Spawn;
use crate::keybinds::Keybind;


// File configuration
include!("../config.rs");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (conn, screen_sum) = x11rb::connect(None)?;
    // Настройки экрана
    let screen = &conn.setup().roots[screen_sum];

    // Попытка захвата экрана для проверки запускается ли сессия
    let change = x11rb::protocol::xproto::ChangeWindowAttributesAux::default().event_mask(
        x11rb::protocol::xproto::EventMask::SUBSTRUCTURE_REDIRECT
            | x11rb::protocol::xproto::EventMask::SUBSTRUCTURE_NOTIFY,
    );

    let result = conn.change_window_attributes(screen.root, &change)?.check();

    // Проверка на запуск X11 сесcии
    if result.is_err() {
        eprintln!("Не удалось захватить экран. Возможно запущен уже другой wm.");
        std::process::exit(1);
    }

    let keybinds = KEYBINDS;

    for bind in keybinds {
        conn.grab_key(
            true,
            screen.root,
            ModMask::from(bind.mods),
            bind.button,
            GrabMode::ASYNC,
            GrabMode::ASYNC
        )?;
    }

    conn.flush()?;

    loop {
        let event = conn.wait_for_event()?;

        match event {
            // Первичная настройка геометрии
            x11rb::protocol::Event::ConfigureRequest(e) => {
                let aux = ConfigureWindowAux::from_configure_request(&e);
                conn.configure_window(e.window, &aux)?;
            }
            // Разрешение на отображение
            x11rb::protocol::Event::MapRequest(e) => {
                conn.map_window(e.window)?;
                conn.flush()?;
            }
            // Обработка событий клавишь
            x11rb::protocol::Event::KeyPress(e) => {
                let clean_state = u16::from(e.state) & !(0x0002 | 0x0010);
                println!("Кнопка-{} Модификации-{:?}", e.detail, e.state);

                if let Some(bind) = KEYBINDS.iter().find(|b| b.button == e.detail && b.mods == clean_state) {
                    match bind.action {
                        Quit => {
                            println!("Закрытие оконного менеджера");
                            break Ok(());
                        }
                        Spawn(command) => {
                            #[allow(clippy::zombie_processes)]
                            std::process::Command::new("sh")
                                .arg("-c")
                                .arg(command)
                                .spawn()
                                .expect("Не удалось запустить команду");
                        }
                        _ => {}
                    }
                } 
            }
            _ => {}
        }
    }
}

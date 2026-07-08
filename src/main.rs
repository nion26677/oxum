use x11rb::connection::Connection;

use crate::core::client::WmState;

mod core;
mod handlers;
mod x11;

include!("../config.rs");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];

    WmState::grab_root_events(&conn, screen)?;

    let mut wm = WmState::new(screen);
    WmState::load_keybinds(&conn, screen, KEYBINDS)?;

    conn.flush()?;

    x11::events_loop::run(&conn, &mut wm)
}

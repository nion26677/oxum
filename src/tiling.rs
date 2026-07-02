use x11rb::{
    connection::Connection,
    protocol::xproto::{ConfigureWindowAux, ConnectionExt},
};

use crate::{BORDER_WIDTH, MASTER_RATIO};

/// Описание видов тайлингов
pub enum TilingType {
    Stack,
}

/// Структура хранящая рабочие столы и ширину master
pub struct Workspace {
    pub windows: Vec<u32>,
    pub master_ratio: f32,
}

/// `impl` для придания свойств тайлинга конкретному перечислению
impl TilingType {
    pub fn arrange(
        &self,
        conn: &impl Connection,
        size_width: u16,
        size_height: u16,
        workspace: &Workspace,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            TilingType::Stack => arrange_for_stack(conn, size_width, size_height, workspace),
        }
    }
}

/// Создаёт заранее прописаный по умолчанию конифгурацию области
impl Default for Workspace {
    fn default() -> Self {
        Self {
            windows: Vec::new(),
            master_ratio: MASTER_RATIO,
        }
    }
}

/// Тайлинг для типа Stack
fn arrange_for_stack(
    conn: &impl Connection,
    sw: u16,
    sh: u16,
    ws: &Workspace,
) -> Result<(), Box<dyn std::error::Error>> {
    let windows_count = ws.windows.len();

    // Игнорируем расчёт геометрии при осутствии окон
    if windows_count == 0 {
        return Ok(());
    }

    let sw = sw as u32;
    let sh = sh as u32;

    // Одно окно занимает весь экран.
    if windows_count == 1 {
        conn.configure_window(
            ws.windows[0],
            &ConfigureWindowAux::new()
                .x(0).y(0)
                .width(sw.saturating_sub(2 * BORDER_WIDTH))
                .height(sh.saturating_sub(2 * BORDER_WIDTH)),
        )?;
        return Ok(());
    }

    // Округляем значения вне диапозона для `master` окна
    let ratio = ws.master_ratio.clamp(0.05, 0.95);
    let master_w = ((sw as f32) * ratio) as u32;

    // Гарантируем хотя бы 1 пиксель стека, иначе стек схлопнется.
    let master_w = master_w.min(sw.saturating_sub(1));
    let stack_w = sw - master_w;

    // Высоту стека делим поровну между n - 1 окнами, без отступов.
    let stack_count = (windows_count - 1) as u32;
    let stack_h = sh / stack_count;

    for (i, &win) in ws.windows.iter().enumerate() {
        if i == 0 {
            // Мастер слева, на всю высоту экрана.
            conn.configure_window(
                win,
                &ConfigureWindowAux::new()
                    .x(0)
                    .y(0)
                    .width(master_w.saturating_sub(2 * BORDER_WIDTH))
                    .height(sh.saturating_sub(2 * BORDER_WIDTH)),
            )?;
        } else {
            // Окна стека справа, без отступов, плотно друг под другом.
            let stack_idx = (i - 1) as u32;
            let win_y = stack_idx * stack_h;
            // Последнее окно добиваем до низа экрана, чтобы избежать
            // щели из-за целочисленного деления.
            let win_h = if stack_idx + 1 == stack_count {
                sh - win_y
            } else {
                stack_h
            };

            conn.configure_window(
                win,
                &ConfigureWindowAux::new()
                    .x(master_w as i32)
                    .y(win_y as i32)
                    .width(stack_w.saturating_sub(2 * BORDER_WIDTH))
                    .height(win_h.saturating_sub(2 * BORDER_WIDTH)),
            )?;
        }
    }

    Ok(())
}

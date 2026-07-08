use x11rb::{
    connection::Connection,
    protocol::xproto::{ConfigureWindowAux, ConnectionExt},
};

use crate::{BORDER_WIDTH, MASTER_RATIO};

pub type Windows = Vec<u32>;

pub struct Stack {
    /// Соотношение ширины master-окна к общей ширине экрана (0.0..=1.0).
    pub master_ratio: f32,
}

impl Stack {
    /// Создать новый `Stack` с указанным `master_ratio` (0.0..=1.0).
    pub fn new(master_ratio: f32) -> Self {
        Self { master_ratio }
    }
}

impl Default for Stack {
    fn default() -> Self {
        // Берём значение из `config.rs` — это единственный источник правды.
        Self::new(MASTER_RATIO)
    }
}

pub trait Layout {
    /// Расположить `windows` по экрану размером `sw × sh`.
    fn arrange(
        &self,
        conn: &impl Connection,
        sw: u32,
        sh: u32,
        windows: &[u32],
    ) -> Result<(), Box<dyn std::error::Error>>;
}

/// Текущий режим тайлинга. Сейчас доступен только `Stack`,
/// но перечисление сделано расширяемым — чтобы добавить,
/// например, `Monocle` или `Floating`, достаточно реализовать `Layout`.
pub enum TilingType {
    Stack(Stack),
}

impl Default for TilingType {
    fn default() -> Self {
        Self::Stack(Stack::default())
    }
}

impl Layout for Stack {
    fn arrange(
        &self,
        conn: &impl Connection,
        sw: u32,
        sh: u32,
        windows: &[u32],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let count = windows.len();
        if count == 0 {
            return Ok(());
        }

        // Одно окно — растягиваем на весь экран.
        if count == 1 {
            conn.configure_window(
                windows[0],
                &ConfigureWindowAux::new()
                    .x(0)
                    .y(0)
                    .width(sw.saturating_sub(2 * BORDER_WIDTH))
                    .height(sh.saturating_sub(2 * BORDER_WIDTH)),
            )?;
            return Ok(());
        }

        // Master слева, остальные делят правую колонку поровну.
        let ratio = self.master_ratio.clamp(0.05, 0.95);
        let master_w = ((sw as f32) * ratio) as u32;
        let master_w = master_w.min(sw.saturating_sub(1));
        let stack_w = sw - master_w;
        let stack_count = (count - 1) as u32;
        let stack_h = sh / stack_count;

        conn.configure_window(
            windows[0],
            &ConfigureWindowAux::new()
                .x(0)
                .y(0)
                .width(master_w.saturating_sub(2 * BORDER_WIDTH))
                .height(sh.saturating_sub(2 * BORDER_WIDTH)),
        )?;

        for (idx, &win) in windows.iter().skip(1).enumerate() {
            let stack_idx = idx as u32;
            let win_y = stack_idx * stack_h;
            // Последнее окно забирает остаток высоты, чтобы не было щели внизу.
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

        Ok(())
    }
}

use std::collections::HashSet;

use log::info;
use x11rb::{
    protocol::xproto::{ChangeWindowAttributesAux, ConnectionExt, ModMask, Screen},
    rust_connection::RustConnection,
};

use crate::{
    NUM_TAGS,
    core::{layouts::TilingType, tags::TagSet},
    handlers::keybinds::{Keybind, keysym_to_keycode},
};

/// Состояние оконного менеджера.
///
/// Хранится как обычная структура (без `Mutex`/`LazyLock`),
/// поскольку event loop однопоточный. Передаётся в хендлеры как `&mut WmState`.
pub struct WmState {
    /// Геометрия экрана в пикселях.
    pub screen: (u32, u32),
    /// Какие окна в каких тегах и какие теги сейчас показаны.
    pub tags: TagSet,
    /// Текущий режим тайлинга. По умолчанию — `Stack` (см. `TilingType::default`).
    pub tag: TilingType,
    /// Окна, которые мы сейчас прячем по собственной инициативе
    /// (через `unmap_window` при смене тега). `unmap_notify` использует этот
    /// набор, чтобы отличить «WM спрятал» от «клиент закрылся».
    pub unmapping: HashSet<u32>,
    /// Окна, которые мы сейчас показываем по собственной инициативе
    /// (через `map_window` при смене тега). `map_request` использует этот
    /// набор, чтобы не дублировать border/event_mask и не рекурсить.
    pub mapping: HashSet<u32>,
}

impl WmState {
    pub fn new(screen: &Screen) -> Self {
        Self {
            screen: (
                u32::from(screen.width_in_pixels),
                u32::from(screen.height_in_pixels),
            ),
            tags: TagSet::new(NUM_TAGS),
            tag: TilingType::default(),
            unmapping: HashSet::new(),
            mapping: HashSet::new(),
        }
    }

    /// `(width, height)` экрана — удобный хелпер для хендлеров.
    #[inline]
    pub fn screen_size(&self) -> (u32, u32) {
        self.screen
    }

    /// Находится ли окно `win` под нашим управлением прямо сейчас.
    #[inline]
    pub fn is_managed(&self, win: u32) -> bool {
        self.tags.contains(win)
    }

    /// Список видимых сейчас окон в порядке, пригодном для тайлинга.
    pub fn visible_windows(&self) -> Vec<u32> {
        self.tags.visible_windows()
    }

    /// Множество управляемых окон, которые сейчас **показаны** (т.е. не в `unmapping`).
    /// Используется в `apply_tag_change` для определения, какие окна нужно
    /// показать через `map_window`.
    pub fn visible_managed(&self) -> std::collections::HashSet<u32> {
        let visible_tags = self.visible_windows().into_iter().collect::<HashSet<_>>();
        visible_tags
            .into_iter()
            .filter(|w| !self.unmapping.contains(w))
            .collect()
    }

    /// Расположить все видимые окна согласно текущему `tag` (layout).
    pub fn arrange(&self, conn: &RustConnection) -> Result<(), Box<dyn std::error::Error>> {
        use crate::core::layouts::Layout;
        let (sw, sh) = self.screen_size();
        let windows = self.visible_windows();
        match &self.tag {
            TilingType::Stack(layout) => layout.arrange(conn, sw, sh, &windows)?,
        }
        Ok(())
    }

    /// Подписаться на события root-окна: WM должен получать
    /// `MapRequest` / `ConfigureRequest` / `UnmapNotify` от всех клиентов.
    /// Без `SUBSTRUCTURE_REDIRECT` X-сервер будет доставлять их напрямую
    /// приложениям, и тайлинг работать не будет.
    pub fn grab_root_events(
        conn: &RustConnection,
        screen: &Screen,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let aux = ChangeWindowAttributesAux::default().event_mask(
            x11rb::protocol::xproto::EventMask::SUBSTRUCTURE_REDIRECT
                | x11rb::protocol::xproto::EventMask::SUBSTRUCTURE_NOTIFY,
        );
        conn.change_window_attributes(screen.root, &aux)?
            .check()
            .map_err(|e| {
                Box::<dyn std::error::Error>::from(format!(
                    "Не удалось запросить SUBSTRUCTURE_REDIRECT: {e}. \
                     Возможно, уже запущен другой WM/композитор."
                ))
            })?;
        Ok(())
    }

    /// Зарегистрировать в X-сервере все keybind'ы, описанные в `config.rs`.
    pub fn load_keybinds(
        conn: &RustConnection,
        screen: &Screen,
        keybinds: &[Keybind],
    ) -> Result<(), Box<dyn std::error::Error>> {
        for bind in keybinds {
            if let Some(keycode) = keysym_to_keycode(conn, bind.keysym)? {
                conn.grab_key(
                    true,
                    screen.root,
                    ModMask::from(bind.mods),
                    keycode,
                    x11rb::protocol::xproto::GrabMode::ASYNC,
                    x11rb::protocol::xproto::GrabMode::ASYNC,
                )?;
            } else {
                info!(
                    "Не удалось найти KeyCode для KeySym {} — бинд проигнорирован",
                    bind.keysym
                );
            }
        }
        Ok(())
    }
}

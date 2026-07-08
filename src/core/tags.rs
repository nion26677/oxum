//! Модель тегов (dwm-style).
//!
//! Каждый тег — набор окон. Окно может одновременно находиться в нескольких
//! тегах. Какие теги «активны» (показаны на экране) задаёт битовая маска
//! [`TagSet::active`].
//!
//! Семантика близка к dwm: окно не «принадлежит» одному workspace,
//! оно отмечено набором тегов, а пользователь переключает видимость тегов.

use std::collections::HashSet;

/// Один тег — просто множество id окон.
#[derive(Debug, Default, Clone)]
pub struct Tag {
    windows: HashSet<u32>,
}

impl Tag {
    pub fn add(&mut self, win: u32) {
        self.windows.insert(win);
    }

    /// Удалить окно из тега. Возвращает `true`, если оно там было.
    pub fn remove(&mut self, win: u32) -> bool {
        self.windows.remove(&win)
    }

    pub fn contains(&self, win: u32) -> bool {
        self.windows.contains(&win)
    }
}

/// Набор тегов фиксированного размера `n` + битовая маска активных.
///
/// Реализация хранения активных тегов — простой `u32` битсет.
/// Максимум тегов: 32 (с запасом для будущего расширения).
#[derive(Debug, Clone)]
pub struct TagSet {
    tags: Vec<Tag>,
    /// Бит `i` установлен ⇔ тег `i` показан на экране.
    active: u32,
    /// Тег, в который попадают новые окна (в dwm — последний использованный).
    focused: usize,
}

impl TagSet {
    pub fn new(n: usize) -> Self {
        Self {
            tags: (0..n).map(|_| Tag::default()).collect(),
            active: 1, // по умолчанию показан только тег 0
            focused: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.tags.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    /// Активен ли тег `i`.
    pub fn is_active(&self, i: usize) -> bool {
        i < 32 && (self.active & (1 << i)) != 0
    }

    /// Текущий «фокусный» тег — в него попадают новые окна.
    pub fn focused(&self) -> usize {
        self.focused
    }

    pub fn set_focused(&mut self, i: usize) {
        if i < self.tags.len() {
            self.focused = i;
        }
    }

    /// Показать только тег `i` (поведение `view` в dwm).
    pub fn view(&mut self, i: usize) {
        if i < self.tags.len() {
            self.active = 1 << i;
        }
    }

    /// Показать все теги разом.
    pub fn view_all(&mut self) {
        self.active = if self.tags.len() >= 32 {
            u32::MAX
        } else {
            (1u32 << self.tags.len()) - 1
        };
    }

    /// Переключить видимость тега `i` (поведение `toggletag` в dwm).
    pub fn toggle(&mut self, i: usize) {
        if i < 32 && i < self.tags.len() {
            self.active ^= 1 << i;
        }
    }

    /// Добавить окно в фокусный тег.
    pub fn add_window(&mut self, win: u32) {
        if let Some(tag) = self.tags.get_mut(self.focused) {
            tag.add(win);
        }
    }

    /// Убрать окно из всех тегов. Возвращает `true`, если оно где-то было.
    pub fn remove_window(&mut self, win: u32) -> bool {
        let mut found = false;
        for tag in &mut self.tags {
            if tag.remove(win) {
                found = true;
            }
        }
        found
    }

    /// Переместить окно в тег `to` (очистив остальные его теги).
    /// Если окна нигде не было — оно просто появится в `to`.
    pub fn move_to(&mut self, win: u32, to: usize) {
        if to >= self.tags.len() {
            return;
        }
        for tag in &mut self.tags {
            tag.remove(win);
        }
        self.tags[to].add(win);
        self.focused = to;
    }

    /// Находится ли окно в каком-либо теге.
    pub fn contains(&self, win: u32) -> bool {
        self.tags.iter().any(|t| t.contains(win))
    }

    /// Список **всех** управляемых окон (из любых тегов), в стабильном порядке.
    ///
    /// Нужен хендлерам, чтобы скрыть окна, не входящие в активные теги.
    pub fn all_managed(&self) -> Vec<u32> {
        let mut out: Vec<u32> = self
            .tags
            .iter()
            .flat_map(|tag| tag.windows.iter().copied())
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Список видимых сейчас окон, в стабильном порядке.
    ///
    /// Используется в `arrange` и `cycle_focus`. Чтобы порядок был
    /// предсказуемым (master + stack), сортируем по `(tag_index, window_id)`.
    /// Это близко к dwm, где порядок окон внутри тега задаётся стеком фокуса.
    pub fn visible_windows(&self) -> Vec<u32> {
        let mut out: Vec<u32> = self
            .tags
            .iter()
            .enumerate()
            .filter(|(i, _)| self.is_active(*i))
            .flat_map(|(_, tag)| tag.windows.iter().copied())
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Id последнего видимого окна (для восстановления фокуса).
    pub fn last_visible(&self) -> Option<u32> {
        self.visible_windows().last().copied()
    }
}

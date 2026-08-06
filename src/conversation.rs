#[derive(Debug)]
pub enum Message {
    Sued(String),
    User(String),
}

/// How far `[PgUp PgDn]` jump, in rows.
///
/// ⚠ Deliberately NOT the real viewport height. A true page needs the popover's
/// inner height, which is only known inside the render — the same measurement
/// gap that pushed the scroll anchor to the bottom. On a transcript that is one
/// or two screens long, "a big jump" is all this key has to be, so a constant
/// buys the behaviour without plumbing render geometry back into key handling.
/// Tune it by eye once the popover draws.
pub const PAGE_ROWS: u16 = 10;

#[derive(Debug, Default, Copy, Clone)]
pub struct HistoryView {
    from_bottom: u16,
}

impl HistoryView {
    pub fn new() -> Self {
        Self::default()
    }

    /// Rows scrolled back from the newest end of the thread. `0` is the view
    /// F1 opens on. Deliberately unbounded above — see `handle_up`.
    pub fn rows_from_bottom(&self) -> u16 {
        self.from_bottom
    }

    pub fn handle_up(&mut self) {
        self.from_bottom = self.from_bottom.saturating_add(1);
    }

    pub fn handle_down(&mut self) {
        self.from_bottom = self.from_bottom.saturating_sub(1);
    }

    pub fn handle_page_up(&mut self) {
        self.from_bottom = self.from_bottom.saturating_add(PAGE_ROWS);
    }

    pub fn handle_page_down(&mut self) {
        // self.selected = (self.selected() + 1).min(self.len.saturating_sub(1));
        self.from_bottom = self.from_bottom.saturating_sub(PAGE_ROWS);
    }
}

#[derive(Debug)]
pub enum Overlay {
    Transcript(HistoryView), // this variant OWNS a HistoryView
    ConfirmLeave(ConfirmChoice),
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum ConfirmChoice {
    Leave,
    #[default]
    Stay,
}

impl ConfirmChoice {
    pub fn toggle(&mut self) {
        match self {
            ConfirmChoice::Leave => *self = ConfirmChoice::Stay,
            ConfirmChoice::Stay => *self = ConfirmChoice::Leave,
        }
    }
}

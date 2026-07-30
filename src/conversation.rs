#[allow(dead_code)] // remove after complete the task
#[derive(Debug)]
pub enum Message {
    Sued(String),
    User(String),
}

#[derive(Debug)]
pub struct HistoryView {
    selected: usize,
    len: usize,
}

impl HistoryView {
    pub fn opened_on_last(len: usize) -> Self {
        Self {
            selected: len.saturating_sub(1),
            len,
        }
    }

    pub fn handle_up(&mut self) {
        self.selected = self.selected().saturating_sub(1)
    }

    pub fn handle_down(&mut self) {
        self.selected = (self.selected() + 1).min(self.len.saturating_sub(1));
    }

    pub fn jump_to_first(&mut self) {
        self.selected = 0
    }
    pub fn jump_to_last(&mut self) {
        self.selected = self.len.saturating_sub(1)
    }

    pub fn selected(&self) -> usize {
        self.selected
    }
}

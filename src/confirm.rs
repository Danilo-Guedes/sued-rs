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

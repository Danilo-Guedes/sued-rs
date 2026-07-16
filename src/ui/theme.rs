use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Sangue,
    Ambar,
    Fosforo,
}

impl Theme {
    pub const ALL: [Theme; 3] = [Theme::Sangue, Theme::Ambar, Theme::Fosforo];
}

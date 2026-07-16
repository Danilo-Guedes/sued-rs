use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    PtBr,
    EnUs,
    EsEs,
}

impl Language {
    pub const ALL: [Language; 3] = [Language::PtBr, Language::EnUs, Language::EsEs];
}

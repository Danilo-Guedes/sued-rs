//! `sued.json` — SueD's preferences file.
//!
//! Both the file and every key in it are optional. With no `sued.json` on disk
//! SueD runs on its defaults, so you only need to write down the settings you
//! actually want to change:
//!
//! ```json
//! {
//!   "theme": "sangue",
//!   "audio_volume": 80,
//!   "animations": true,
//!   "language": "ptbr"
//! }
//! ```
//!
//! - **`theme`** — `"sangue"` (default), `"ambar"`, or `"fosforo"`.
//! - **`audio_volume`** — `0`–`100`. `0` is silence.
//! - **`animations`** — `true` (default) lets the horror flicker, flash, shake effects; `false` holds it
//!   steady, which also helps if flickering bothers your eyes.
//!
//! A missing file is normal and silent. A file that *exists* but is malformed,
//! or that names a key SueD doesn't recognise, is a hard error — SueD would
//! rather stop and say so than ignore your settings and leave you wondering
//! why nothing changed.
//!
//! The incantations SueD types and the insults it throws back aren't set here;
//! they belong to the oracle, not to you.

// Volume is a 0–100 PERCENT here on purpose. kira speaks decibels
// (`Decibels(0.0)` = full, `Decibels(-60.0)` = silence), so the percent→dB
// mapping belongs at the audio edge, not in the config file. Passing a raw
// fraction to kira compiles and is wrong: `.volume(0.5)` means +0.5 dB ≈ 106%.

use std::{fs, path::Path};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{app::ConfigOption, language::Language, ui::theme::Theme};

const MAX_ALLOWED_VOLUME: u8 = 100;
const VOLUME_STEP: u8 = 10;

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Configuration {
    theme: Theme,
    audio_volume: u8,
    animations: bool,
    language: Language,
}

impl Configuration {
    pub fn from_json(conf_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(conf_str)
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        match fs::read_to_string(path) {
            Ok(text) => {
                Self::from_json(&text).with_context(|| format!("parsing {}", path.display()))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
        }
    }

    pub fn to_json(self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self)
    }

    pub fn handle_right_click(&mut self, selected_config: ConfigOption) {
        match selected_config {
            ConfigOption::Theme => {
                let theme_options_size = Theme::ALL.len();
                let new_opt_idx = (self.theme_index() + 1) % theme_options_size;
                self.theme = Theme::ALL[new_opt_idx];
            }
            ConfigOption::Animations => {
                self.animations = !self.animations();
            }
            ConfigOption::Volume => {
                let new_vol = self
                    .audio_volume
                    .saturating_add(VOLUME_STEP)
                    .min(MAX_ALLOWED_VOLUME);
                self.audio_volume = new_vol;
            }
            ConfigOption::Language => {
                let language_options_size = Language::ALL.len();
                let new_language_idx = (self.language_index() + 1) % language_options_size;
                self.language = Language::ALL[new_language_idx]
            }
        }
    }

    pub fn handle_left_click(&mut self, selected_config: ConfigOption) {
        match selected_config {
            ConfigOption::Theme => {
                let theme_options_size = Theme::ALL.len();
                let new_opt_idx =
                    (self.theme_index() + theme_options_size - 1) % theme_options_size;
                self.theme = Theme::ALL[new_opt_idx];
            }
            ConfigOption::Animations => {
                self.animations = !self.animations();
            }
            ConfigOption::Volume => {
                let new_vol = self.audio_volume.saturating_sub(VOLUME_STEP);
                self.audio_volume = new_vol;
            }
            ConfigOption::Language => {
                let language_options_size = Language::ALL.len();
                let new_language_idx =
                    (self.language_index() + language_options_size - 1) * language_options_size;
                self.language = Language::ALL[new_language_idx]
            }
        }
    }

    //GETTERS

    pub fn theme(&self) -> Theme {
        self.theme
    }

    pub fn theme_index(&self) -> usize {
        self.theme() as usize
    }

    pub fn audio_volume(&self) -> u8 {
        self.audio_volume
    }

    pub fn animations(&self) -> bool {
        self.animations
    }

    pub fn language(&self) -> Language {
        self.language
    }

    pub fn language_index(&self) -> usize {
        self.language() as usize
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            theme: Theme::Sangue,
            audio_volume: 80,
            animations: true,
            language: Language::PtBr,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_todays_hardcoded_behaviour() {
        let config = Configuration::default();

        assert_eq!(config.theme, Theme::Sangue);
        assert_eq!(config.audio_volume, 80);
        assert_eq!(config.language, Language::PtBr);
        assert!(config.animations);
    }

    #[test]
    fn an_empty_json_object_yields_every_default() {
        // This is what `#[serde(default)]` on the struct buys: `{}` is a valid,
        // complete config. Without it, serde rejects every missing key.
        let config = Configuration::from_json("{}").expect("`{}` must be a valid config");

        assert_eq!(config, Configuration::default());
    }

    #[test]
    fn a_partial_config_defaults_only_the_missing_fields() {
        let config = Configuration::from_json(r#"{ "theme": "ambar" }"#)
            .expect("a config naming one field must parse");

        assert_eq!(
            config.theme,
            Theme::Ambar,
            "the named field must be honoured"
        );
        assert_eq!(config.audio_volume, 80, "an unnamed field must fall back");
        assert!(config.animations, "an unnamed field must fall back");
    }

    #[test]
    fn a_full_config_parses_every_field() {
        let json = r#"{ "theme": "fosforo", "audio_volume": 40, "animations": false, "language": "ptbr" }"#;

        let config = Configuration::from_json(json).expect("a complete config must parse");

        assert_eq!(config.theme, Theme::Fosforo);
        assert_eq!(config.audio_volume, 40);
        assert!(!config.animations);
        assert_eq!(config.language, Language::PtBr)
    }

    #[test]
    fn every_theme_parses_from_its_unaccented_lowercase_name() {
        // The wire format is unaccented ASCII; `Âmbar`/`Fósforo` are UI labels,
        // which is a rendering concern and none of serde's business.
        for (name, expected) in [
            ("sangue", Theme::Sangue),
            ("ambar", Theme::Ambar),
            ("fosforo", Theme::Fosforo),
        ] {
            let json = format!(r#"{{ "theme": "{name}" }}"#);

            let config = Configuration::from_json(&json)
                .unwrap_or_else(|e| panic!("theme {name:?} must parse, got: {e}"));

            assert_eq!(config.theme, expected, "theme {name:?} parsed wrong");
        }
    }

    #[test]
    fn an_unknown_theme_is_rejected() {
        let result = Configuration::from_json(r#"{ "theme": "roxo" }"#);

        assert!(
            result.is_err(),
            "a theme we don't have must fail loudly, not silently pick one"
        );
    }

    #[test]
    fn an_unknown_key_is_rejected() {
        // Serde IGNORES unknown fields unless told otherwise, so this typo
        // parses happily and `audio_volume` stays 80 — the user's edit
        // vanishes with no message. Same silent failure we refused elsewhere.
        let result = Configuration::from_json(r#"{ "audio_volumee": 40 }"#);

        assert!(
            result.is_err(),
            "a misspelled key must fail loudly, not leave the user wondering why nothing changed"
        );
    }

    #[test]
    fn malformed_json_is_rejected() {
        let result = Configuration::from_json("{ this is not json");

        assert!(
            result.is_err(),
            "a broken file must error, never quietly become the default"
        );
    }

    #[test]
    fn a_config_survives_a_json_round_trip() {
        let original = Configuration {
            theme: Theme::Fosforo,
            audio_volume: 25,
            animations: false,
            language: Language::default(),
        };

        let json = original.to_json().expect("a config must serialize");
        let parsed = Configuration::from_json(&json).expect("our own output must parse back");

        assert_eq!(parsed, original, "a round trip must not lose anything");
    }

    #[test]
    fn load_falls_back_to_defaults_when_the_file_is_missing() {
        // No file is the NORMAL case — a fresh clone has no sued.json and must
        // still run. This path is never created, so there's nothing to clean up.
        let missing = std::path::Path::new("/sued-rs/no/such/dir/sued.json");
        assert!(
            !missing.exists(),
            "this test needs a path that truly doesn't exist"
        );

        let config = Configuration::load(missing).expect("a missing file must NOT be an error");

        assert_eq!(config, Configuration::default());
    }
}

//! User preferences, loaded from `sued.json` via `serde` + `serde_json` (M5).
//!
//! **Config is preferences, not content.** The decoy/denial pools deliberately
//! do NOT live here — they're app content baked into the binary (and keyed by
//! language once i18n lands), so a user can't empty a pool and break the prank.
//! What's here is only what someone would reasonably want to tune: theme,
//! volume, flicker.
//!
//! `Default` holds today's hardcoded behaviour, so the app runs identically with
//! no `sued.json` on disk at all.
//
// TODO(Danilo, M5): implement to the API the tests below pin —
//   * `Config { theme: Theme, audio_volume: u8, flicker: bool }`
//     with `#[serde(default)]` on the STRUCT (not per-field), so any missing
//     key falls back to `Default`.
//   * `impl Default for Config` = Sangue / 100 / true.
//   * `enum Theme { Sangue, Ambar, Fosforo }`, `#[serde(rename_all = "lowercase")]`,
//     `#[default] Sangue`.
//   * `from_json` / `to_json` — pure, where the logic lives.
//   * `load` / `save` — thin filesystem edge. `load` forgives ONLY `NotFound`.
//
// Volume is a 0–100 PERCENT here on purpose. kira speaks decibels
// (`Decibels(0.0)` = full, `Decibels(-60.0)` = silence), so the percent→dB
// mapping belongs at the audio edge, not in the config file. Passing a raw
// fraction to kira compiles and is wrong: `.volume(0.5)` means +0.5 dB ≈ 106%.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_todays_hardcoded_behaviour() {
        let config = Config::default();

        assert_eq!(config.theme, Theme::Sangue);
        assert_eq!(config.audio_volume, 100);
        assert!(config.flicker);
    }

    #[test]
    fn an_empty_json_object_yields_every_default() {
        // This is what `#[serde(default)]` on the struct buys: `{}` is a valid,
        // complete config. Without it, serde rejects every missing key.
        let config = Config::from_json("{}").expect("`{}` must be a valid config");

        assert_eq!(config, Config::default());
    }

    #[test]
    fn a_partial_config_defaults_only_the_missing_fields() {
        let config = Config::from_json(r#"{ "theme": "ambar" }"#)
            .expect("a config naming one field must parse");

        assert_eq!(
            config.theme,
            Theme::Ambar,
            "the named field must be honoured"
        );
        assert_eq!(config.audio_volume, 100, "an unnamed field must fall back");
        assert!(config.flicker, "an unnamed field must fall back");
    }

    #[test]
    fn a_full_config_parses_every_field() {
        let json = r#"{ "theme": "fosforo", "audio_volume": 40, "flicker": false }"#;

        let config = Config::from_json(json).expect("a complete config must parse");

        assert_eq!(config.theme, Theme::Fosforo);
        assert_eq!(config.audio_volume, 40);
        assert!(!config.flicker);
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

            let config = Config::from_json(&json)
                .unwrap_or_else(|e| panic!("theme {name:?} must parse, got: {e}"));

            assert_eq!(config.theme, expected, "theme {name:?} parsed wrong");
        }
    }

    #[test]
    fn an_unknown_theme_is_rejected() {
        let result = Config::from_json(r#"{ "theme": "roxo" }"#);

        assert!(
            result.is_err(),
            "a theme we don't have must fail loudly, not silently pick one"
        );
    }

    #[test]
    fn malformed_json_is_rejected() {
        let result = Config::from_json("{ this is not json");

        assert!(
            result.is_err(),
            "a broken file must error, never quietly become the default"
        );
    }

    #[test]
    fn a_config_survives_a_json_round_trip() {
        let original = Config {
            theme: Theme::Fosforo,
            audio_volume: 25,
            flicker: false,
        };

        let json = original.to_json().expect("a config must serialize");
        let parsed = Config::from_json(&json).expect("our own output must parse back");

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

        let config = Config::load(missing).expect("a missing file must NOT be an error");

        assert_eq!(config, Config::default());
    }
}

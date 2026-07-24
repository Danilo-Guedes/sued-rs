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

    /// The on-screen label for this language, distinct from the lowercase serde
    /// wire format (`ptbr`/`enus`/`eses`).
    pub fn label(&self) -> &'static str {
        match self {
            Language::PtBr => "PT-BR",
            Language::EnUs => "EN-US",
            Language::EsEs => "ES-ES",
        }
    }

    /// The oracle's words in this language — the text counterpart of
    /// `Theme::palette()`: three literal tables, one per language.
    pub fn translation(&self) -> Translation {
        todo!("Danilo: three literal tables, one per language")
    }
}

/// Everything the oracle says in one language. Like `Palette`, this travels by
/// value and is looked up fresh each time — flipping `idioma` retranslates on
/// the next read, no caching, no invalidation.
#[derive(Debug, Copy, Clone)]
pub struct Translation {
    /// Fake questions the decoy "types itself" from during hidden input.
    pub decoys: &'static [&'static str],
    /// Taunts for a question asked without a staged answer.
    pub denials: &'static [&'static str],
    /// The opening line on a fresh ask screen.
    pub welcome_line: &'static str,
}

/// Map a random roll in `0.0..=1.0` onto one entry of a non-empty pool.
/// Pure — the caller supplies the roll (`rand::random()` at the edge), tests
/// supply explicit ones.
pub fn pick<'a>(pool: &[&'a str], roll: f32) -> &'a str {
    todo!("Danilo: multiply, floor, clamp — mind the roll == 1.0 edge")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── pick: the roll → entry mapping ───────────────────────────────────────
    // The contract mirrors `laugh_interval`: multiply the roll across the pool
    // and floor into an index. `rand::random::<f32>()` yields `0.0..1.0`, but
    // the clamp at exactly 1.0 is pinned anyway — an inclusive roll from a
    // future caller must never index out of bounds (the `%`-vs-`*` crash of
    // M5 was this same off-by-the-edge family).

    const POOL: [&str; 4] = ["primeiro", "segundo", "terceiro", "quarto"];

    #[test]
    fn pick_with_roll_zero_takes_the_first_entry() {
        assert_eq!(pick(&POOL, 0.0), "primeiro");
    }

    #[test]
    fn pick_maps_the_roll_linearly_across_the_pool() {
        // 0.25 × 4 = 1.0 and 0.5 × 4 = 2.0 — exact in f32, so these pins are
        // deterministic: the mapping is floor(roll × len), nothing fancier.
        assert_eq!(pick(&POOL, 0.25), "segundo");
        assert_eq!(pick(&POOL, 0.5), "terceiro");
        assert_eq!(pick(&POOL, 0.75), "quarto");
    }

    #[test]
    fn pick_with_a_roll_just_under_one_takes_the_last_entry() {
        assert_eq!(pick(&POOL, 0.99), "quarto");
    }

    #[test]
    fn pick_with_roll_exactly_one_clamps_to_the_last_entry() {
        // 1.0 × 4 = index 4 — one past the end. The clamp is the whole test.
        assert_eq!(pick(&POOL, 1.0), "quarto");
    }

    #[test]
    fn pick_from_a_single_entry_pool_always_returns_it() {
        let lonely = ["único"];
        assert_eq!(pick(&lonely, 0.0), "único");
        assert_eq!(pick(&lonely, 0.5), "único");
        assert_eq!(pick(&lonely, 1.0), "único");
    }

    // ── translation tables: the tripwires ────────────────────────────────────
    // These don't test logic — they guard the literal tables against the
    // drift class theme day surfaced twice (colors migrating between themes).
    // A failure names the language and the offending line.

    #[test]
    fn every_language_has_a_nonempty_decoy_pool() {
        for lang in Language::ALL {
            assert!(
                !lang.translation().decoys.is_empty(),
                "{lang:?} has no decoys — pick() would have nothing to draw from"
            );
        }
    }

    #[test]
    fn every_language_has_a_nonempty_denial_pool() {
        for lang in Language::ALL {
            assert!(
                !lang.translation().denials.is_empty(),
                "{lang:?} has no denials — an open question would have no taunt"
            );
        }
    }

    #[test]
    fn every_language_has_a_welcome_line() {
        for lang in Language::ALL {
            assert!(
                !lang.translation().welcome_line.is_empty(),
                "{lang:?} has an empty welcome line"
            );
        }
    }

    #[test]
    fn no_decoy_line_is_shared_between_languages() {
        for (i, a) in Language::ALL.iter().enumerate() {
            for b in &Language::ALL[i + 1..] {
                for line in a.translation().decoys {
                    assert!(
                        !b.translation().decoys.contains(line),
                        "decoy {line:?} appears in both {a:?} and {b:?} — copy-paste drift"
                    );
                }
            }
        }
    }

    #[test]
    fn no_denial_line_is_shared_between_languages() {
        for (i, a) in Language::ALL.iter().enumerate() {
            for b in &Language::ALL[i + 1..] {
                for line in a.translation().denials {
                    assert!(
                        !b.translation().denials.contains(line),
                        "denial {line:?} appears in both {a:?} and {b:?} — copy-paste drift"
                    );
                }
            }
        }
    }

    #[test]
    fn the_welcome_line_differs_per_language() {
        for (i, a) in Language::ALL.iter().enumerate() {
            for b in &Language::ALL[i + 1..] {
                assert_ne!(
                    a.translation().welcome_line,
                    b.translation().welcome_line,
                    "{a:?} and {b:?} share a welcome line — copy-paste drift"
                );
            }
        }
    }

    #[test]
    fn every_decoy_is_long_enough_to_paint_a_question() {
        // A decoy shorter than the secret answer exhausts mid-prank: the fake
        // question freezes on screen while the operator is still typing. 20
        // chars comfortably outlasts typical secret answers and still reads
        // as a real question mid-crawl.
        const MIN_DECOY_CHARS: usize = 20;

        for lang in Language::ALL {
            for decoy in lang.translation().decoys {
                assert!(
                    decoy.chars().count() >= MIN_DECOY_CHARS,
                    "{lang:?} decoy {decoy:?} is under {MIN_DECOY_CHARS} chars"
                );
            }
        }
    }
}

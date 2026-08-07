//! Keystroke fixtures shared by the `app` and `ui::screens` test modules.
//!
//! ⚠⚠ **WHY THIS MODULE EXISTS — a real bug, not tidiness.** G17 split refusals
//! in two on the LENGTH of the question: `<= SHORT_QUESTION_CHARS` earns the
//! rebuke, anything longer earns a pooled denial. Every test that had hand-rolled
//! its own short question silently changed meaning the day that landed —
//! `the_ask_screen_draws_a_denial` and `the_taunt_is_visible_once_sued_refuses`
//! both typed `"oi"` (2 chars) and quietly began exercising the rebuke path while
//! **still passing**, leaving the denial render path with no coverage at all and
//! nothing to say so.
//!
//! The rule itself is real domain behaviour and tests should be able to state it.
//! What failed was relying on the test author to *remember* it. So: one named
//! fixture per outcome, nobody writes a question inline, and
//! `the_fixtures_actually_straddle_the_threshold` turns "someone retuned the
//! constant" from a scattered silent drift into one loud failure that names the
//! cause.
//!
//! `screens.rs` could not reach `app::tests`' constants, which is exactly why it
//! hand-rolled its own — hence a module both can see rather than a `pub(crate)`
//! on a `#[cfg(test)] mod tests`.

use crate::core::engine::KeyPress;

/// Long enough to reach the denial pool, and kept that way by
/// `the_fixtures_actually_straddle_the_threshold`.
pub(crate) const DENIED_QUESTION: &str = "will the oracle answer me tonight?";

/// Short enough to earn the rebuke. One of the greetings the feature was
/// designed around.
pub(crate) const REBUKED_QUESTION: &str = "hello there";

/// A denial-length question in Portuguese, for the tests that also flip
/// `idioma` — the point there is *which language* SueD refuses in, so the
/// question has to be PT **and** past the threshold.
pub(crate) const DENIED_QUESTION_PT: &str = "Eae Sued, o quê você sabe sobre mim?";

/// One keystroke per character of `text`.
pub(crate) fn typing(text: &str) -> Vec<KeyPress> {
    text.chars().map(KeyPress::Char).collect()
}

/// Reach the ask screen and submit `question` with NO hidden answer staged, so
/// the engine answers `Denied` and SueD refuses — as a denial or a rebuke
/// depending on how long `question` is.
///
/// Prefer [`ask_and_be_denied`] or [`ask_and_be_rebuked`]: passing a literal here
/// is how the threshold got re-derived at five call sites in the first place.
pub(crate) fn ask_openly(question: &str) -> Vec<KeyPress> {
    let mut keys = vec![
        KeyPress::Enter, // Intro → Menu
        KeyPress::Enter, // Menu → Asking
    ];
    keys.extend(typing(question));
    keys.push(KeyPress::Enter); // → Denied
    keys
}

/// Ask something SueD deigns to weigh, and be refused from the pool.
pub(crate) fn ask_and_be_denied() -> Vec<KeyPress> {
    ask_openly(DENIED_QUESTION)
}

/// Ask something too short to be a question, and be rebuked for it.
pub(crate) fn ask_and_be_rebuked() -> Vec<KeyPress> {
    ask_openly(REBUKED_QUESTION)
}

/// The key that raises the story popover on About.
///
/// `?` and deliberately **not** `F1`: `F1` already means "transcript" on Ask,
/// which is one nav-strip step away, and one key meaning two things across
/// adjacent screens is a collision rather than a per-screen binding.
pub(crate) const STORY_KEY: KeyPress = KeyPress::Char('?');

/// Reach the About screen from a fresh app. Menu order is
/// Perguntar · Informações · Sobre · Configurações · Sair, so Sobre is two
/// steps down — encoded once here so nobody counts `Down`s at a call site.
pub(crate) fn reach_about() -> Vec<KeyPress> {
    vec![
        KeyPress::Enter, // Intro → Menu
        KeyPress::Down,  // → Informações
        KeyPress::Down,  // → Sobre
        KeyPress::Enter, // Menu → About
    ]
}

/// Reach About and raise the story popover, which opens on its first line.
pub(crate) fn open_the_story() -> Vec<KeyPress> {
    let mut keys = reach_about();
    keys.push(STORY_KEY);
    keys
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::SHORT_QUESTION_CHARS;

    #[test]
    fn the_fixtures_actually_straddle_the_threshold() {
        // ⚠ THE GUARD THAT WOULD HAVE CAUGHT G17's SILENT DRIFT. Today
        // `DENIED_QUESTION` clears the bound by luck of how it was worded — retune
        // `SHORT_QUESTION_CHARS` upward and every "denial" fixture in the suite
        // starts producing rebukes instead. The tests that assert on the pool
        // would fail confusingly; the ones that only `draw()` or check
        // `is_pondering()` would keep passing while covering the wrong branch.
        //
        // This is the one place that failure becomes a single, named error.
        assert!(
            DENIED_QUESTION.chars().count() > SHORT_QUESTION_CHARS,
            "DENIED_QUESTION ({} chars) must stay LONGER than SHORT_QUESTION_CHARS \
             ({SHORT_QUESTION_CHARS}), or every test that believes it covers a \
             denial is quietly covering a rebuke",
            DENIED_QUESTION.chars().count()
        );
        assert!(
            DENIED_QUESTION_PT.chars().count() > SHORT_QUESTION_CHARS,
            "DENIED_QUESTION_PT ({} chars) must clear the threshold too — the \
             language tests refuse in PT, and a rebuke would refuse in PT just as \
             happily while testing the wrong branch",
            DENIED_QUESTION_PT.chars().count()
        );
        assert!(
            REBUKED_QUESTION.chars().count() <= SHORT_QUESTION_CHARS,
            "REBUKED_QUESTION ({} chars) must stay within SHORT_QUESTION_CHARS \
             ({SHORT_QUESTION_CHARS}), or the rebuke tests stop reaching a rebuke",
            REBUKED_QUESTION.chars().count()
        );
    }
}

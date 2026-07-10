//! Terror effects (M4): flicker, screen-shake, color-flash, char-by-char reveal.
//! Driven by `Engine` state changes; kept out of the pure core.
//!
//! First effect: the char-by-char "SUED FALA" reveal. The key idea is that the
//! animation is a **pure function of elapsed time** — given how long ago the
//! reveal started, we derive how many characters should be on screen. No
//! per-frame counter (those drift and couple to the frame rate).

use std::time::Duration;

/// Milliseconds of elapsed time per revealed character (~18 cps — ominous crawl).
/// Tunable: larger = slower. The tests derive their timings from this constant,
/// so retuning the speed here won't break the spec.
pub const REVEAL_MS_PER_CHAR: u64 = 55;

/// How many characters of the answer should be visible after `elapsed` time has
/// passed since the reveal began, clamped to `total`.
///
/// Pure and total: no I/O, no clock, no randomness — you hand it an elapsed
/// `Duration` and it tells you the visible-char count. That is what makes it
/// unit-testable; the only impure bit (reading the real clock via
/// `Instant::elapsed()`) stays out at the render boundary.
///
/// TODO(Danilo, M4): implement me until the tests below go green.
/// Hint: visible chars = `elapsed / REVEAL_MS_PER_CHAR`, then clamp to `total`.
fn typewriter_len(elapsed: Duration, total: usize) -> usize {
    let visible_chars = elapsed.as_millis() as u64 / REVEAL_MS_PER_CHAR;
    visible_chars.min(total as u64) as usize
}

fn typewriter_slice(text: &str, duration: Duration) -> String {
    let total_boundary = text.chars().count();
    let n_to_be_revealed = typewriter_len(duration, total_boundary);
    let revealed_text: String = text.chars().take(n_to_be_revealed).collect();
    revealed_text
}

pub fn typewriter_reveal(text: &str, elapsed: Duration) -> String {
    let mut visible = typewriter_slice(text, elapsed);
    let still_typing = visible.chars().count() < text.chars().count();
    if still_typing && cursor_on(elapsed) {
        visible.push(CURSOR_GLYPH);
    }
    visible
}

const CURSOR_BLINK_MS: u64 = 500;

const CURSOR_GLYPH: char = '█';

fn cursor_on(elapsed: Duration) -> bool {
    (elapsed.as_millis() as u64 / CURSOR_BLINK_MS).is_multiple_of(2)
}

#[cfg(test)]
mod tests {
    use super::{CURSOR_BLINK_MS, REVEAL_MS_PER_CHAR, cursor_on, typewriter_len, typewriter_slice};
    use std::time::Duration;

    /// Elapsed time expressed as "n characters' worth" of reveal intervals.
    /// Deriving from the constant keeps the spec correct if we retune the speed.
    fn after_chars(n: u64) -> Duration {
        Duration::from_millis(n * REVEAL_MS_PER_CHAR)
    }

    /// Elapsed time expressed as "n blink phases' worth", derived from the
    /// constant so the spec survives retuning the blink speed.
    fn after_phases(n: u64) -> Duration {
        Duration::from_millis(n * CURSOR_BLINK_MS)
    }

    #[test]
    fn zero_elapsed_reveals_nothing() {
        assert_eq!(typewriter_len(Duration::ZERO, 10), 0);
    }

    #[test]
    fn reveals_one_char_per_interval() {
        // Exactly 5 intervals in → 5 whole characters have landed.
        assert_eq!(typewriter_len(after_chars(5), 10), 5);
    }

    #[test]
    fn floors_partial_intervals() {
        // 2.5 intervals in → only the 2 *completed* chars show (floor, not round).
        let two_and_a_half = Duration::from_millis(REVEAL_MS_PER_CHAR * 5 / 2);
        assert_eq!(typewriter_len(two_and_a_half, 10), 2);
    }

    #[test]
    fn clamps_to_total_when_time_overflows() {
        // Long after the reveal finished, we never exceed the answer length.
        assert_eq!(typewriter_len(after_chars(1000), 3), 3);
    }

    #[test]
    fn empty_answer_is_always_zero() {
        // Nothing to reveal, no matter how much time passes.
        assert_eq!(typewriter_len(after_chars(1000), 0), 0);
    }

    // ── typewriter_slice: the char-safe reveal helper both branches will share ──
    // Same clock as `typewriter_len`, but hands back the actual visible prefix so
    // `ask.rs` stops duplicating the `.chars().take(n).collect()` dance.

    #[test]
    fn slice_reveals_nothing_at_zero_elapsed() {
        assert_eq!(typewriter_slice("abcdef", Duration::ZERO), "");
    }

    #[test]
    fn slice_reveals_the_first_k_chars() {
        // 3 intervals in → the first 3 characters, in order.
        assert_eq!(typewriter_slice("abcdef", after_chars(3)), "abc");
    }

    #[test]
    fn slice_reveals_the_whole_string_once_time_overflows() {
        // Long after the crawl finished, we get the full text — never more.
        assert_eq!(typewriter_slice("abc", after_chars(1000)), "abc");
    }

    #[test]
    fn slice_of_empty_text_is_empty() {
        assert_eq!(typewriter_slice("", after_chars(1000)), "");
    }

    #[test]
    fn slice_counts_and_cuts_in_chars_not_bytes() {
        // Regression: 'É' is two UTF-8 bytes, so a byte slice `&text[..1]` would
        // panic mid-character. Revealing one char must yield "É", never a panic —
        // and a later boundary must stay char-aligned.
        assert_eq!(typewriter_slice("É42", after_chars(1)), "É");
        assert_eq!(typewriter_slice("É42", after_chars(2)), "É4");
    }

    // ── cursor_on: the blink phase, shared by the reveal/input/logs cursors ─────
    // Pure like typewriter_len: hand it elapsed time, get back whether the cursor
    // is currently lit. A *phase* is one on OR off stretch (CURSOR_BLINK_MS long);
    // a full blink cycle is two phases. Lit on even phases, dark on odd.

    #[test]
    fn cursor_starts_visible() {
        // At the very start of the first phase the cursor is lit.
        assert!(cursor_on(Duration::ZERO));
    }

    #[test]
    fn cursor_stays_on_through_the_first_phase() {
        // Anywhere inside the first phase (before one full CURSOR_BLINK_MS) → on.
        let mid_first_phase = Duration::from_millis(CURSOR_BLINK_MS / 2);
        assert!(cursor_on(mid_first_phase));
    }

    #[test]
    fn cursor_turns_off_in_the_second_phase() {
        // One whole phase in, the cursor blinks off...
        assert!(!cursor_on(after_phases(1)));
        // ...and stays off for the rest of that phase.
        let mid_second_phase = Duration::from_millis(CURSOR_BLINK_MS + CURSOR_BLINK_MS / 2);
        assert!(!cursor_on(mid_second_phase));
    }

    #[test]
    fn cursor_comes_back_on_after_a_full_cycle() {
        // Two phases = one full blink cycle → lit again.
        assert!(cursor_on(after_phases(2)));
    }

    #[test]
    fn cursor_keeps_alternating() {
        // Even phases on, odd phases off — the blink never desyncs over time.
        assert!(!cursor_on(after_phases(3)));
        assert!(cursor_on(after_phases(10)));
        assert!(!cursor_on(after_phases(11)));
    }
}

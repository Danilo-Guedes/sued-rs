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
pub fn typewriter_len(elapsed: Duration, total: usize) -> usize {
    let _ = (elapsed, total);
    todo!("M4: derive visible-char count from elapsed / REVEAL_MS_PER_CHAR, clamped to total")
}

#[cfg(test)]
mod tests {
    use super::{typewriter_len, REVEAL_MS_PER_CHAR};
    use std::time::Duration;

    /// Elapsed time expressed as "n characters' worth" of reveal intervals.
    /// Deriving from the constant keeps the spec correct if we retune the speed.
    fn after_chars(n: u64) -> Duration {
        Duration::from_millis(n * REVEAL_MS_PER_CHAR)
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
}

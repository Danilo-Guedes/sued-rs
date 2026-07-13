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
const REVEAL_MS_PER_CHAR: u64 = 55;

const CURSOR_BLINK_MS: u64 = 400;

pub const CURSOR_CHAR: char = '█';

const FLASH_MS: u64 = 400;

// FLICKER CONSTANTS

const MAX_INTENSITY: u64 = 255;

const FLICKER_CHANCE: f32 = 0.06;

const MIN_FLICKER_VALUE: u8 = 160;

// SHAKE CONSTANTS

const SHAKE_MS: u64 = 700;

const SHAKE_MAX_CELLS: i16 = 2;

/// How many characters of the answer should be visible after `elapsed` time has
/// passed since the reveal began, clamped to `total`.
///
/// Pure and total: no I/O, no clock, no randomness — you hand it an elapsed
/// `Duration` and it tells you the visible-char count. That is what makes it
/// unit-testable; the only impure bit (reading the real clock via
/// `Instant::elapsed()`) stays out at the render boundary.
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
        visible.push(CURSOR_CHAR);
    }
    visible
}

pub fn cursor_on(elapsed: Duration) -> bool {
    (elapsed.as_millis() as u64 / CURSOR_BLINK_MS).is_multiple_of(2)
}

/// returns a intensity number from 0 to MAX_INTENSITY [255]
/// using FLASH_MS [400] as base divider
/// it'll be used to create flash ui effects
pub fn flash_intensity(elapsed: Duration) -> u8 {
    let elapsed_ms = elapsed.as_millis() as u64;

    if elapsed_ms >= FLASH_MS {
        return 0;
    }

    let faded = elapsed_ms * MAX_INTENSITY / FLASH_MS;

    (MAX_INTENSITY - faded) as u8
}

///a method that accept a rand f32 as arg values and map to a u8 in a scale of [MIN_FLICKER_VALUE(60)..256]
/// using the FLICKER_CHANGE(0.12) as a yearly return logic
/// where roll > FLICKER_CHANCE return u8::MAX (256)
/// only percentages bellow FLICKER_CHANGE will actually dim you text
/// use this returned value as a dim scale color to simulate flickering
pub fn flicker_intensity(roll: f32) -> u8 {
    if roll >= FLICKER_CHANCE {
        return u8::MAX;
    }

    // how far UP from the floor toward full brightness is this roll?
    let brightness_fraction = roll / FLICKER_CHANCE;

    // Room between the deepest dip and full brightness (255 − 60 = 195).
    let range_above_floor = u8::MAX as f32 - MIN_FLICKER_VALUE as f32;

    // Start at the floor, climb `brightness_fraction` of the way up that range.
    (MIN_FLICKER_VALUE as f32 + brightness_fraction * range_above_floor) as u8
}

pub fn shake_offset(elapsed: Duration, roll_x: f32, roll_y: f32) -> (i16, i16) {
    let elapesed_in_ms = elapsed.as_millis() as u64;
    if elapesed_in_ms >= SHAKE_MS {
        return (0, 0);
    }

    //how much has passed
    let faded = elapesed_in_ms * SHAKE_MAX_CELLS as u64 / SHAKE_MS;
    // how much is left to hit max_cell
    let left = SHAKE_MAX_CELLS as u64 - faded;

    // `roll * 2 - 1` maps the [0,1) roll to a signed [-1,+1] direction/strength,
    // then `* left` scales it into the current [-left, +left] cell range.
    let x_offset = (roll_x * 2.0 - 1.0) * left as f32;
    let y_offset = (roll_y * 2.0 - 1.0) * left as f32;

    (x_offset as i16, y_offset as i16)
}

#[cfg(test)]
mod tests {
    use super::{
        CURSOR_BLINK_MS, CURSOR_CHAR, FLASH_MS, FLICKER_CHANCE, MIN_FLICKER_VALUE,
        REVEAL_MS_PER_CHAR, SHAKE_MAX_CELLS, SHAKE_MS, cursor_on, flash_intensity,
        flicker_intensity, shake_offset, typewriter_len, typewriter_reveal, typewriter_slice,
    };
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

    // ── typewriter_reveal: the slice + a blinking cursor while the crawl runs ────
    // Ties the two clocks together. Expressed against `typewriter_slice` so the
    // assertions survive retuning either speed — we pin the *rule* (slice, plus a
    // cursor iff still-typing AND on an on-phase), not hard-coded prefixes.

    #[test]
    fn reveal_shows_a_lone_cursor_at_the_very_start() {
        // elapsed 0: nothing sliced yet, but the crawl is underway → a lone cursor
        // blinks (design choice: the block shows from the start, chars stream past).
        assert_eq!(
            typewriter_reveal("abc", Duration::ZERO),
            CURSOR_CHAR.to_string()
        );
    }

    #[test]
    fn reveal_drops_the_cursor_once_fully_revealed() {
        // The invariant that matters most: a finished answer must NOT keep a
        // cursor blinking at its tail.
        assert_eq!(typewriter_reveal("abc", after_chars(1000)), "abc");
    }

    #[test]
    fn reveal_of_empty_text_is_empty_and_uncursored() {
        assert_eq!(typewriter_reveal("", after_chars(1000)), "");
    }

    #[test]
    fn reveal_appends_the_cursor_mid_crawl_on_an_on_phase() {
        // 2 chars in (phase 0 → lit) and still typing → slice + the cursor glyph.
        let text = "abcdef";
        let elapsed = after_chars(2);
        assert!(
            typewriter_slice(text, elapsed).chars().count() < text.chars().count(),
            "fixture must be mid-crawl for this to mean anything"
        );
        let expected = format!("{}{CURSOR_CHAR}", typewriter_slice(text, elapsed));
        assert_eq!(typewriter_reveal(text, elapsed), expected);
    }

    #[test]
    fn reveal_hides_the_cursor_mid_crawl_on_an_off_phase() {
        // One blink phase in the cursor is dark, so even mid-crawl the reveal is
        // *just* the slice — no glyph. The long text keeps us still-typing then.
        let text = "abcdefghijklmnopqrst";
        let elapsed = after_phases(1);
        assert!(
            typewriter_slice(text, elapsed).chars().count() < text.chars().count(),
            "fixture must be mid-crawl for the off-phase check to be meaningful"
        );
        assert_eq!(
            typewriter_reveal(text, elapsed),
            typewriter_slice(text, elapsed)
        );
    }

    /// Elapsed time expressed as the fraction `num/den` of one flash lifetime,
    /// derived from the constant so the spec survives retuning the flash speed.
    fn flash_fraction(num: u64, den: u64) -> Duration {
        Duration::from_millis(FLASH_MS * num / den)
    }

    #[test]
    fn flash_peaks_at_the_instant_of_reveal() {
        // elapsed 0 = the reveal *just* fired → fully red. (This is the same ZERO
        // an ungated `None` would pass in — hence the render-boundary note above.)
        assert_eq!(flash_intensity(Duration::ZERO), 255);
    }

    #[test]
    fn flash_is_dark_once_its_lifetime_elapses() {
        // Exactly one FLASH_MS in, the flash has fully faded.
        assert_eq!(flash_intensity(Duration::from_millis(FLASH_MS)), 0);
    }

    #[test]
    fn flash_stays_dark_long_after() {
        // Well past the lifetime it never wraps or underflows back to bright.
        assert_eq!(flash_intensity(Duration::from_millis(FLASH_MS * 10)), 0);
    }

    #[test]
    fn flash_is_partway_between_peak_and_dark_mid_fade() {
        // Halfway through the lifetime it's genuinely fading: dimmer than the peak
        // but not yet out. We pin the *rule* (strictly between), not the exact
        // byte — integer division lands it on 128, not the ~127 you'd eyeball.
        let mid = flash_intensity(flash_fraction(1, 2));
        assert!(
            mid > 0 && mid < 255,
            "mid-fade intensity was {mid}, want 0 < x < 255"
        );
    }

    #[test]
    fn flash_fades_monotonically() {
        // Never brightens as time moves forward. Non-increasing (NOT strictly
        // decreasing): integer division makes the curve plateau for a millisecond
        // or two between steps, which is fine.
        let samples: Vec<u8> = (0..=4)
            .map(|k| flash_intensity(flash_fraction(k, 4)))
            .collect();
        for pair in samples.windows(2) {
            assert!(
                pair[0] >= pair[1],
                "flash brightened over time: {samples:?}"
            );
        }
    }

    #[test]
    fn flicker_is_full_brightness_at_or_above_the_chance() {
        // The common case: the vast majority of rolls leave the demon fully lit.
        assert_eq!(flicker_intensity(FLICKER_CHANCE), u8::MAX);
        assert_eq!(flicker_intensity(0.5), u8::MAX);
        assert_eq!(flicker_intensity(0.999), u8::MAX);
    }

    #[test]
    fn flicker_hits_the_floor_at_roll_zero() {
        // The deepest possible dip is the floor — a flicker never goes darker.
        assert_eq!(flicker_intensity(0.0), MIN_FLICKER_VALUE);
    }

    #[test]
    fn flicker_dim_band_sits_between_floor_and_full() {
        // A roll inside the dim band is a partial dip: dimmer than full, no darker
        // than the floor. Mid-band roll so it's safely interior, not on an edge.
        let dim = flicker_intensity(FLICKER_CHANCE * 0.5);
        assert!(
            dim > MIN_FLICKER_VALUE && dim < u8::MAX,
            "dim-band intensity was {dim}, want {MIN_FLICKER_VALUE} < x < {}",
            u8::MAX
        );
    }

    #[test]
    fn flicker_intensity_never_decreases_as_the_roll_rises() {
        // Brighter roll → brighter (or equal) demon: no inversions across the dim
        // band and on into full brightness. Non-increasing would be a bug.
        let rolls = [
            0.0,
            FLICKER_CHANCE * 0.25,
            FLICKER_CHANCE * 0.5,
            FLICKER_CHANCE * 0.75,
            FLICKER_CHANCE,
            0.5,
            0.99,
        ];
        let intensities: Vec<u8> = rolls.iter().map(|&r| flicker_intensity(r)).collect();
        for pair in intensities.windows(2) {
            assert!(
                pair[0] <= pair[1],
                "flicker intensity dropped as the roll rose: {intensities:?}"
            );
        }
    }

    // ── shake_offset: the reveal jolt — flash's decay ⊗ flicker's randomness ────
    // Pure like the rest: `elapsed` drives the decaying amplitude, and the two
    // rolls (from `rand` at the edge) place us inside `[-amp, +amp]` per axis. We
    // pin the *rules* — center = still, peak = full, settles to nothing, decays,
    // bounded, axes independent — not frame-exact offsets.

    /// Elapsed time as the fraction `num/den` of one shake lifetime, derived from
    /// the constant so the spec survives retuning the shake speed.
    fn shake_fraction(num: u64, den: u64) -> Duration {
        Duration::from_millis(SHAKE_MS * num / den)
    }

    #[test]
    fn shake_is_centered_for_the_neutral_roll() {
        // A roll of 0.5 sits dead-center of [-amp, +amp] (`0.5 * 2 - 1 == 0`), so
        // that axis never moves — even at the very peak of the shake.
        assert_eq!(shake_offset(Duration::ZERO, 0.5, 0.5), (0, 0));
    }

    #[test]
    fn shake_reaches_full_amplitude_at_the_instant_of_reveal() {
        // elapsed 0 = peak amplitude. The extreme rolls hit the corners of the
        // jolt: 0.0 → the full negative throw, 1.0 → the full positive throw.
        assert_eq!(
            shake_offset(Duration::ZERO, 0.0, 0.0),
            (-SHAKE_MAX_CELLS, -SHAKE_MAX_CELLS)
        );
        assert_eq!(
            shake_offset(Duration::ZERO, 1.0, 1.0),
            (SHAKE_MAX_CELLS, SHAKE_MAX_CELLS)
        );
    }

    #[test]
    fn shake_settles_to_nothing_once_its_lifetime_elapses() {
        // Exactly one SHAKE_MS in, the jolt is spent — dead still for ANY roll...
        assert_eq!(
            shake_offset(Duration::from_millis(SHAKE_MS), 0.0, 1.0),
            (0, 0)
        );
        // ...and long after it never wraps or underflows back to life. That guard
        // is the flash lesson again — the one bug the happy-path tests can't see.
        assert_eq!(shake_offset(shake_fraction(10, 1), 0.0, 1.0), (0, 0));
    }

    #[test]
    fn shake_amplitude_decays_from_its_peak() {
        // The same extreme roll, later in the window → a strictly smaller throw.
        // This is the whole point: the shake calms instead of rattling forever.
        let peak = shake_offset(Duration::ZERO, 1.0, 1.0).0;
        let midway = shake_offset(shake_fraction(1, 2), 1.0, 1.0).0;
        assert!(
            midway.abs() < peak.abs(),
            "midway throw {midway} was not smaller than the peak {peak}"
        );
    }

    #[test]
    fn shake_never_throws_further_than_the_max() {
        // Across the whole roll range and the whole lifetime, neither axis ever
        // exceeds SHAKE_MAX_CELLS in magnitude — the shifted Rect stays sane.
        let rolls = [0.0, 0.25, 0.5, 0.75, 1.0];
        for num in 0..=4 {
            for &rx in &rolls {
                for &ry in &rolls {
                    let (dx, dy) = shake_offset(shake_fraction(num, 4), rx, ry);
                    assert!(
                        dx.abs() <= SHAKE_MAX_CELLS && dy.abs() <= SHAKE_MAX_CELLS,
                        "offset ({dx},{dy}) exceeded max {SHAKE_MAX_CELLS}"
                    );
                }
            }
        }
    }

    #[test]
    fn shake_decays_monotonically() {
        // Hold the roll at the positive extreme and walk time forward: the throw
        // never grows. Non-increasing (integer truncation makes it plateau).
        let throws: Vec<i16> = (0..=4)
            .map(|k| shake_offset(shake_fraction(k, 4), 1.0, 0.5).0)
            .collect();
        for pair in throws.windows(2) {
            assert!(pair[0] >= pair[1], "shake grew over time: {throws:?}");
        }
    }

    #[test]
    fn shake_axes_are_independent() {
        // roll_x drives dx and only dx; roll_y drives dy and only dy. A neutral
        // roll on one axis keeps it still while the other throws to full.
        assert_eq!(shake_offset(Duration::ZERO, 1.0, 0.5), (SHAKE_MAX_CELLS, 0));
        assert_eq!(shake_offset(Duration::ZERO, 0.5, 1.0), (0, SHAKE_MAX_CELLS));
    }
}

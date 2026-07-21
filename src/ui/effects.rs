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

pub fn reveal_is_complete(text: &str, elapsed: Duration) -> bool {
    let total_char = text.chars().count();
    let visible_chars = (elapsed.as_millis() as u64 / REVEAL_MS_PER_CHAR) as usize;

    visible_chars >= total_char
}

/// How hot the reply flash burns `elapsed` after SueD answered, as an RGB red
/// channel: `MAX_INTENSITY` at the instant of the reply, fading linearly to `0`
/// once `FLASH_MS` has passed and staying there.
///
/// `0` is the effect's **rest value**, which the render side draws as
/// `Color::Reset` — so a flash that is over and a flash that never started are
/// the same frame, and the caller needs no special case.
///
/// Pure like the rest of this module: you hand it an elapsed `Duration`, the
/// real clock (`Instant::elapsed()`) stays out at the render boundary.
///
/// `enable_animations = false` returns that rest value immediately — the
/// photosensitivity half of the accessibility gate (see `Configuration::animations`).
/// Note it returns rest rather than asking the caller to skip drawing: "effects
/// off" must still produce a complete frame.
pub fn flash_intensity(elapsed: Duration, enable_animations: bool) -> u8 {
    let elapsed_ms = elapsed.as_millis() as u64;

    if !enable_animations || (elapsed_ms >= FLASH_MS) {
        return 0;
    }

    let faded = elapsed_ms * MAX_INTENSITY / FLASH_MS;

    (MAX_INTENSITY - faded) as u8
}

/// How bright the demon burns this frame, as an RGB red channel, decided by a
/// random `roll` in `[0.0, 1.0)` that the caller supplies.
///
/// Only rolls *below* `FLICKER_CHANCE` dim anything — so about 6% of frames dip
/// and every other frame comes back `u8::MAX`, full brightness. Inside that band
/// the value climbs from `MIN_FLICKER_VALUE` (the deepest dip the demon ever
/// takes) up toward full, which makes a roll of `0.0` the darkest possible frame
/// and a roll just under the chance barely perceptible.
///
/// The randomness itself lives out at the render edge (`rand::random()`), which
/// is exactly what keeps this testable: the tests feed explicit rolls.
///
/// `animations_enabled = false` returns **full brightness**. Worth pausing on:
/// this rest value sits at the opposite end of the range from `flash_intensity`'s
/// and `shake_offset`'s, because "no flicker" means an *undimmed* demon, not a
/// dark one. The dip is the effect; being lit is the resting state.
pub fn flicker_intensity(roll: f32, animations_enabled: bool) -> u8 {
    if !animations_enabled || (roll >= FLICKER_CHANCE) {
        return u8::MAX;
    }

    // how far UP from the floor toward full brightness is this roll?
    let brightness_fraction = roll / FLICKER_CHANCE;

    // Room between the deepest dip and full brightness (255 − 160 = 95).
    let range_above_floor = u8::MAX as f32 - MIN_FLICKER_VALUE as f32;

    // Start at the floor, climb `brightness_fraction` of the way up that range.
    (MIN_FLICKER_VALUE as f32 + brightness_fraction * range_above_floor) as u8
}

/// How far to jolt the demon's `Rect` this frame, in `(x, y)` terminal cells.
///
/// This one is the other two effects multiplied together: `flash_intensity`'s
/// decaying amplitude — full `SHAKE_MAX_CELLS` at the instant of the reply,
/// settling to nothing once `SHAKE_MS` has passed — times `flicker_intensity`'s
/// randomness, where `roll_x`/`roll_y` in `[0.0, 1.0)` place each axis somewhere
/// inside the current `[-amp, +amp]` range. The axes are independent, so a
/// neutral `0.5` on one of them holds that axis still while the other throws.
///
/// The `>= SHAKE_MS` guard is load-bearing, not an optimisation: without it the
/// amplitude subtraction underflows once the window has passed and the jolt
/// comes back to life instead of dying.
///
/// `enable_animations = false` returns `(0, 0)` — the motion-sickness half of
/// the accessibility gate (see `Configuration::animations`). The render side
/// still offsets and intersects the `Rect`; it just offsets it by nothing.
pub fn shake_offset(
    elapsed: Duration,
    roll_x: f32,
    roll_y: f32,
    enable_animations: bool,
) -> (i16, i16) {
    let elapesed_in_ms = elapsed.as_millis() as u64;
    if !enable_animations || (elapesed_in_ms >= SHAKE_MS) {
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
        flicker_intensity, reveal_is_complete, shake_offset, typewriter_len, typewriter_reveal,
        typewriter_slice,
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
        assert_eq!(flash_intensity(Duration::ZERO, true), 255);
    }

    #[test]
    fn flash_is_dark_once_its_lifetime_elapses() {
        // Exactly one FLASH_MS in, the flash has fully faded.
        assert_eq!(flash_intensity(Duration::from_millis(FLASH_MS), true), 0);
    }

    #[test]
    fn flash_stays_dark_long_after() {
        // Well past the lifetime it never wraps or underflows back to bright.
        assert_eq!(
            flash_intensity(Duration::from_millis(FLASH_MS * 10), true),
            0
        );
    }

    #[test]
    fn flash_is_partway_between_peak_and_dark_mid_fade() {
        // Halfway through the lifetime it's genuinely fading: dimmer than the peak
        // but not yet out. We pin the *rule* (strictly between), not the exact
        // byte — integer division lands it on 128, not the ~127 you'd eyeball.
        let mid = flash_intensity(flash_fraction(1, 2), true);
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
            .map(|k| flash_intensity(flash_fraction(k, 4), true))
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
        assert_eq!(flicker_intensity(FLICKER_CHANCE, true), u8::MAX);
        assert_eq!(flicker_intensity(0.5, true), u8::MAX);
        assert_eq!(flicker_intensity(0.999, true), u8::MAX);
    }

    #[test]
    fn flicker_hits_the_floor_at_roll_zero() {
        // The deepest possible dip is the floor — a flicker never goes darker.
        assert_eq!(flicker_intensity(0.0, true), MIN_FLICKER_VALUE);
    }

    #[test]
    fn flicker_dim_band_sits_between_floor_and_full() {
        // A roll inside the dim band is a partial dip: dimmer than full, no darker
        // than the floor. Mid-band roll so it's safely interior, not on an edge.
        let dim = flicker_intensity(FLICKER_CHANCE * 0.5, true);
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
        let intensities: Vec<u8> = rolls.iter().map(|&r| flicker_intensity(r, true)).collect();
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
        assert_eq!(shake_offset(Duration::ZERO, 0.5, 0.5, true), (0, 0));
    }

    #[test]
    fn shake_reaches_full_amplitude_at_the_instant_of_reveal() {
        // elapsed 0 = peak amplitude. The extreme rolls hit the corners of the
        // jolt: 0.0 → the full negative throw, 1.0 → the full positive throw.
        assert_eq!(
            shake_offset(Duration::ZERO, 0.0, 0.0, true),
            (-SHAKE_MAX_CELLS, -SHAKE_MAX_CELLS)
        );
        assert_eq!(
            shake_offset(Duration::ZERO, 1.0, 1.0, true),
            (SHAKE_MAX_CELLS, SHAKE_MAX_CELLS)
        );
    }

    #[test]
    fn shake_settles_to_nothing_once_its_lifetime_elapses() {
        // Exactly one SHAKE_MS in, the jolt is spent — dead still for ANY roll...
        assert_eq!(
            shake_offset(Duration::from_millis(SHAKE_MS), 0.0, 1.0, true),
            (0, 0)
        );
        // ...and long after it never wraps or underflows back to life. That guard
        // is the flash lesson again — the one bug the happy-path tests can't see.
        assert_eq!(shake_offset(shake_fraction(10, 1), 0.0, 1.0, true), (0, 0));
    }

    #[test]
    fn shake_amplitude_decays_from_its_peak() {
        // The same extreme roll, later in the window → a strictly smaller throw.
        // This is the whole point: the shake calms instead of rattling forever.
        let peak = shake_offset(Duration::ZERO, 1.0, 1.0, true).0;
        let midway = shake_offset(shake_fraction(1, 2), 1.0, 1.0, true).0;
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
                    let (dx, dy) = shake_offset(shake_fraction(num, 4), rx, ry, true);
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
            .map(|k| shake_offset(shake_fraction(k, 4), 1.0, 0.5, true).0)
            .collect();
        for pair in throws.windows(2) {
            assert!(pair[0] >= pair[1], "shake grew over time: {throws:?}");
        }
    }

    #[test]
    fn shake_axes_are_independent() {
        // roll_x drives dx and only dx; roll_y drives dy and only dy. A neutral
        // roll on one axis keeps it still while the other throws to full.
        assert_eq!(
            shake_offset(Duration::ZERO, 1.0, 0.5, true),
            (SHAKE_MAX_CELLS, 0)
        );
        assert_eq!(
            shake_offset(Duration::ZERO, 0.5, 1.0, true),
            (0, SHAKE_MAX_CELLS)
        );
    }

    // ── the `animations` gate: accessibility, not a feature switch ──────────────
    // `animations = false` is SueD's `prefers-reduced-motion`: it must silence the
    // three effects that can genuinely hurt someone — flicker and flash
    // (photosensitivity) and shake (motion sickness). It must NOT touch the
    // typewriter or the cursors; those are benign text reveal, so they have no gate
    // and no test here.
    //
    // The rule every test below pins: **off = the effect's REST value, not a
    // skipped render.** Each fn already knows its own rest state — flicker rests at
    // FULL brightness, flash at 0 (`Color::Reset`), shake at (0, 0) — so the caller
    // never has to know what "no effect" looks like, and a `false` frame is still a
    // complete frame.

    #[test]
    fn flicker_stays_fully_lit_when_animations_are_off() {
        // Rest for flicker is NOT the floor — it's full brightness. A roll deep in
        // the dim band, which would normally dip hard, must leave the demon lit.
        assert_eq!(flicker_intensity(0.0, false), u8::MAX);
        assert_eq!(flicker_intensity(FLICKER_CHANCE * 0.5, false), u8::MAX);
    }

    #[test]
    fn flicker_never_dims_for_any_roll_when_animations_are_off() {
        // The whole point of the gate: no roll, anywhere in the range, can produce
        // a single dark frame. One dim frame in a thousand is still a flash.
        let rolls = [0.0, FLICKER_CHANCE * 0.25, FLICKER_CHANCE * 0.75, 0.5, 0.99];
        for roll in rolls {
            assert_eq!(
                flicker_intensity(roll, false),
                u8::MAX,
                "roll {roll} dimmed the screen with animations off"
            );
        }
    }

    #[test]
    fn flash_is_dark_at_its_peak_when_animations_are_off() {
        // Duration::ZERO is the instant of the reply — the brightest frame the
        // flash ever produces. Gated off, that peak must render as rest (0).
        assert_eq!(flash_intensity(Duration::ZERO, false), 0);
    }

    #[test]
    fn flash_stays_dark_across_its_whole_lifetime_when_animations_are_off() {
        // Not just the peak: every frame of the fade is 0, so there is no window
        // in which the colour moves at all.
        for k in 0..=4 {
            let lit = flash_intensity(flash_fraction(k, 4), false);
            assert_eq!(lit, 0, "flash lit up at {k}/4 through its lifetime");
        }
    }

    #[test]
    fn shake_is_still_at_its_peak_when_animations_are_off() {
        // Duration::ZERO with the extreme rolls is the hardest possible throw.
        // Gated off it must be dead centre, so the Rect never moves.
        assert_eq!(shake_offset(Duration::ZERO, 0.0, 0.0, false), (0, 0));
        assert_eq!(shake_offset(Duration::ZERO, 1.0, 1.0, false), (0, 0));
    }

    #[test]
    fn shake_never_moves_for_any_roll_or_time_when_animations_are_off() {
        // Sweep both rolls across the whole window: not one frame of motion.
        let rolls = [0.0, 0.25, 0.5, 0.75, 1.0];
        for num in 0..=4 {
            for rx in rolls {
                for ry in rolls {
                    assert_eq!(
                        shake_offset(shake_fraction(num, 4), rx, ry, false),
                        (0, 0),
                        "shake moved at {num}/4 with rolls ({rx}, {ry})"
                    );
                }
            }
        }
    }

    // ── reveal_is_complete: the crawl's own finish line (G8) ───────────────────
    // The conversation flow needs to know *when SueD stopped talking*, because
    // that is the moment the input unlocks and the next question can begin. It's
    // the same clock the typewriter already runs on, asked a yes/no question, so
    // the unlock can never drift out of sync with what's on screen.
    //
    // Deliberately phrased over `text` rather than a char count: the caller has
    // the reply string in hand, and passing a length invites the byte-vs-char
    // mistake this module has already been bitten by once.

    #[test]
    fn a_reply_is_not_complete_before_it_starts() {
        // elapsed 0 — SueD has said nothing yet, so the input must stay locked.
        assert!(!reveal_is_complete("abc", Duration::ZERO));
    }

    #[test]
    fn a_reply_is_not_complete_mid_crawl() {
        // 3 of 6 characters in: still talking, still locked.
        assert!(!reveal_is_complete("abcdef", after_chars(3)));
    }

    #[test]
    fn a_reply_is_complete_when_its_last_char_lands() {
        // Exactly the moment the final character appears — not a tick later.
        assert!(reveal_is_complete("abc", after_chars(3)));
    }

    #[test]
    fn a_reply_stays_complete_long_afterwards() {
        // The user may sit and stare before typing again; the door stays open.
        assert!(reveal_is_complete("abc", after_chars(1000)));
    }

    #[test]
    fn an_empty_reply_is_complete_immediately() {
        // Degenerate but reachable: nothing to crawl means nothing to wait for.
        assert!(reveal_is_complete("", Duration::ZERO));
    }

    #[test]
    fn completion_counts_chars_not_bytes() {
        // 'É' is two UTF-8 bytes, so a `text.len()`-based implementation would
        // think this reply is one character longer than it is and hold the input
        // locked for an extra interval after SueD visibly stopped typing.
        assert!(!reveal_is_complete("É42", after_chars(2)));
        assert!(reveal_is_complete("É42", after_chars(3)));
    }

    #[test]
    fn a_complete_reply_is_exactly_the_fully_revealed_text() {
        // The invariant that ties the unlock to the screen: the instant this
        // returns true, the typewriter must already be showing the whole reply
        // with no trailing cursor. If it could go true early, the input would
        // reopen while SueD was still mid-sentence.
        let text = "abcdef";
        let done = after_chars(text.chars().count() as u64);

        assert!(reveal_is_complete(text, done));
        assert_eq!(typewriter_reveal(text, done), text);
    }

    #[test]
    fn animations_on_is_exactly_todays_behaviour() {
        // The gate must be additive: `true` changes nothing that shipped in M4.
        // (Every other test in this module passes `true` and still pins the old
        // values — this one just states the contract out loud.)
        assert_eq!(flicker_intensity(0.0, true), MIN_FLICKER_VALUE);
        assert_eq!(flash_intensity(Duration::ZERO, true), 255);
        assert_eq!(
            shake_offset(Duration::ZERO, 1.0, 1.0, true),
            (SHAKE_MAX_CELLS, SHAKE_MAX_CELLS)
        );
    }
}

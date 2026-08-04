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

// THINKING PAUSE CONSTANTS

const MAX_THINKING_MS: u64 = 6_000;
const MIN_THINKING_MS: u64 = 3_000;

// 3 DOTS ANIMATION
const DOT_CYCLE_MS: u64 = 400;
const DOTS_WIDTH: u64 = 3;

// PULSE INTENSITY

const PULSE_INTENSITY_MAX: u64 = 255;

const PULSE_INTENSITY_MIN: u64 = 160;

const PULSE_INTENSITY_WAVE_TIME: u64 = 1_200;

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

/// True once the typewriter crawl has fully revealed `text`.
///
/// Deliberately derived from the SAME clock and rate (`REVEAL_MS_PER_CHAR`) as
/// `typewriter_reveal`, so "the input has unlocked" can never drift from "the
/// last char is on screen". Takes the text itself rather than a pre-counted
/// length so the char-vs-byte counting stays in one place.
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

pub fn thinking_duration(roll: f32) -> Duration {
    let span = (MAX_THINKING_MS - MIN_THINKING_MS) as f32;

    let millis = MIN_THINKING_MS + (roll.clamp(0.0, 1.0) * span) as u64;

    Duration::from_millis(millis)
}

pub fn is_thinking(since_asked: Duration, thinking_for: Duration) -> bool {
    thinking_for > since_asked
}

pub fn reveal_elapsed(since_asked: Duration, thinking_for: Duration) -> Duration {
    since_asked.saturating_sub(thinking_for)
}

pub fn thinking_dots(elapsed: Duration) -> usize {
    (elapsed.as_millis() as u64 / DOT_CYCLE_MS % DOTS_WIDTH + 1) as usize
}

/// How brightly the spell glows this frame, as the `u8` handed to
/// `Palette::glow` — a triangle wave breathing between `PULSE_INTENSITY_MIN`
/// and `PULSE_INTENSITY_MAX` once every `PULSE_INTENSITY_WAVE_TIME`.
///
/// This replaced the spell's typewriter crawl (G18). The crawl now belongs to
/// the *answer* alone, so letters-arriving-one-by-one means one thing only:
/// SueD is answering. The spell became atmosphere instead of a second, slower
/// answer.
///
/// ⚠ **The floor is not decoration.** `glow(0)` is black on every theme — see
/// `theme::glow_at_zero_intensity_is_black_for_every_theme` — so a wave that
/// reached zero would make the spell *vanish* at every trough. On a `palette.bg`
/// of (7,4,6) that reads as a crash, not as breathing. The wave is therefore
/// mapped INTO `[MIN, MAX]` rather than clipped into it: the whole triangle is
/// squeezed to fit the range, instead of a full-height triangle having its
/// bottom sliced off (which would flatten every trough into a plateau).
///
/// ⚠ **The peak reaching `PULSE_INTENSITY_MAX` is load-bearing too.** Every
/// palette's `peak` tuple *is* its `accent`, so `glow(255)` returns the exact
/// colour the spell already wears. Peak any lower and this "animation tweak"
/// would quietly restyle the ponder, undoing the settled accent-spell-vs-white-
/// reply decision at `ask.rs:116-120`.
///
/// `animations_enabled = false` returns **full brightness**, the same rest value
/// `flicker_intensity` uses and for the same reason: "no pulse" means an
/// undimmed spell, not a dark one. Returning `0` here would freeze the
/// accessibility user at the trough — the accessibility gate causing the very
/// bug the floor exists to prevent.
pub fn pulse_intensity(elapsed: Duration, animations_enabled: bool) -> u8 {
    if !animations_enabled {
        return u8::MAX;
    }

    let elapsed_ms = elapsed.as_millis() as u64;
    let half_wave = PULSE_INTENSITY_WAVE_TIME / 2;

    // WHERE INSIDE THE CURRENT BREATH ARE WE? `%` is what makes this repeat.
    // `/` would answer a different question — "how many breaths have finished" —
    // and that is a counter that only ever grows.
    let phase = elapsed_ms % PULSE_INTENSITY_WAVE_TIME;

    // Fold the phase back on itself so the second half descends: 0 → half → 0.
    // Without the fold the shape is a sawtooth that snaps from full back to the
    // floor in a single frame, which reads as a flash rather than a breath.
    let distance_from_trough = if phase < half_wave {
        phase
    } else {
        PULSE_INTENSITY_WAVE_TIME - phase
    };

    let range_above_floor = PULSE_INTENSITY_MAX - PULSE_INTENSITY_MIN;

    // ⚠ MULTIPLY BEFORE DIVIDING. `distance / half_wave` is integer division, so
    // on its own it is only ever 0 or 1 and every brightness in between is lost.
    let lit = PULSE_INTENSITY_MIN + distance_from_trough * range_above_floor / half_wave;

    // `lit` cannot exceed MAX while the wave time is even, but the `.min` keeps
    // that true if it is ever retuned to an odd number — and `as u8` truncates
    // silently rather than saturating, so one wrapped value would be a BLACK
    // frame rather than an obviously wrong one.
    lit.min(PULSE_INTENSITY_MAX) as u8
}

#[cfg(test)]
mod tests {
    use super::{
        CURSOR_BLINK_MS, CURSOR_CHAR, DOT_CYCLE_MS, FLASH_MS, FLICKER_CHANCE, MAX_THINKING_MS,
        MIN_FLICKER_VALUE, MIN_THINKING_MS, REVEAL_MS_PER_CHAR, SHAKE_MAX_CELLS, SHAKE_MS,
        cursor_on, flash_intensity, flicker_intensity, is_thinking, pulse_intensity,
        reveal_elapsed, reveal_is_complete, shake_offset, thinking_dots, thinking_duration,
        typewriter_len, typewriter_reveal, typewriter_slice,
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

    // ── G13 · the thinking pause ─────────────────────────────────────────────
    //
    // Between Enter and the first revealed character the oracle PONDERS. An
    // instantaneous answer reads robotic and breaks the seance, so a randomized
    // 3-6s beat is inserted in front of the crawl.
    //
    // ⚠ THE SAME-CLOCK RULE (the G8 lesson, and the whole reason `reveal_elapsed`
    // exists as a named function instead of an inline `saturating_sub`): FIVE
    // call sites read this clock — `typewriter_reveal`, `flash_intensity`,
    // `shake_offset`, and `reveal_is_complete` in BOTH `ask.rs` and `app.rs`.
    // Every one of them must see the SHIFTED clock. If a single site keeps
    // reading the raw elapsed, the input unlocks (or the flash fires) while
    // SueD is still thinking, and the bug will only show up in front of an
    // audience.

    /// The duration a roll of `r` buys, expressed from the constants so the spec
    /// survives retuning the range.
    fn thinking_ms(fraction: f64) -> Duration {
        let span = (MAX_THINKING_MS - MIN_THINKING_MS) as f64;
        Duration::from_millis(MIN_THINKING_MS + (span * fraction) as u64)
    }

    #[test]
    fn the_shortest_ponder_is_the_floor() {
        assert_eq!(
            thinking_duration(0.0),
            Duration::from_millis(MIN_THINKING_MS)
        );
    }

    #[test]
    fn the_longest_ponder_is_the_ceiling() {
        // `rand::random::<f32>()` yields 0.0..1.0, so exactly 1.0 never arrives
        // from today's caller — pinned anyway, because an inclusive roll from a
        // future caller must not overshoot the range. Same reasoning as `pick`.
        assert_eq!(
            thinking_duration(1.0),
            Duration::from_millis(MAX_THINKING_MS)
        );
    }

    #[test]
    fn a_midpoint_roll_lands_midway_through_the_range() {
        // The lerp: floor + roll x span. Half a roll buys half the span, NOT
        // half the ceiling — 4.5s, not 3s.
        assert_eq!(thinking_duration(0.5), thinking_ms(0.5));
    }

    #[test]
    fn the_oracle_is_thinking_the_instant_it_is_asked() {
        assert!(is_thinking(Duration::ZERO, thinking_ms(0.5)));
    }

    #[test]
    fn the_oracle_is_still_thinking_one_millisecond_before_time() {
        let ponder = thinking_ms(0.5);
        let almost = ponder - Duration::from_millis(1);
        assert!(is_thinking(almost, ponder));
    }

    #[test]
    fn the_ponder_ends_exactly_when_it_runs_out() {
        // THE BOUNDARY, and the one worth getting right: `<` not `<=`. Off by one
        // here and the pause either never ends or ends a tick early.
        let ponder = thinking_ms(0.5);
        assert!(!is_thinking(ponder, ponder));
    }

    #[test]
    fn the_oracle_stops_thinking_once_it_speaks() {
        let ponder = thinking_ms(0.5);
        assert!(!is_thinking(ponder + Duration::from_secs(10), ponder));
    }

    #[test]
    fn nothing_is_revealed_while_the_oracle_ponders() {
        // The reveal clock must read ZERO for the whole pause — not a small
        // number, ZERO — so `typewriter_len` yields no characters and the box
        // stays empty until SueD actually speaks.
        let ponder = thinking_ms(0.5);
        assert_eq!(reveal_elapsed(Duration::ZERO, ponder), Duration::ZERO);
        assert_eq!(
            reveal_elapsed(ponder - Duration::from_millis(1), ponder),
            Duration::ZERO
        );
        assert_eq!(reveal_elapsed(ponder, ponder), Duration::ZERO);
    }

    #[test]
    fn the_reveal_clock_starts_the_moment_the_ponder_ends() {
        let ponder = thinking_ms(0.5);
        assert_eq!(
            reveal_elapsed(ponder + after_chars(3), ponder),
            after_chars(3)
        );
    }

    #[test]
    fn the_input_stays_locked_for_the_whole_ponder() {
        // THE CONTRACT PIN — this is the same-clock rule as an executable claim,
        // and it is the one that would have caught the G8 bug. A short secret
        // finishes its crawl in ~165ms, far less than the 3s floor, so WITHOUT
        // the shift this passes trivially at t=0 and fails everywhere after.
        let secret = "42";
        let ponder = thinking_ms(0.5);

        for fraction in [0.0, 0.25, 0.5, 0.75, 0.99] {
            let since_asked = ponder.mul_f64(fraction);
            assert!(
                !reveal_is_complete(secret, reveal_elapsed(since_asked, ponder)),
                "input unlocked {fraction} of the way through the ponder — \
                 a consumer is reading the raw clock instead of the shifted one"
            );
        }
    }

    #[test]
    fn the_reveal_still_completes_after_the_ponder() {
        // The other half of the pin: the pause must DELAY the unlock, not
        // prevent it. Without this, "never unlock" would pass the test above.
        let secret = "42";
        let ponder = thinking_ms(0.5);
        let long_after = ponder + after_chars(100);

        assert!(reveal_is_complete(
            secret,
            reveal_elapsed(long_after, ponder)
        ));
    }

    // ── G13 · the waiting dots ───────────────────────────────────────────────
    //
    // Once the incantation has finished typing there is still 0.5-3.5s of ponder
    // left (the spell length is fixed, the pause is rolled), so the line would
    // otherwise sit frozen. Trailing dots cycle 1 -> 2 -> 3 to keep it reading as
    // pending. Same family as `cursor_on`: a pure step function of elapsed, with
    // the randomness and the real clock both left outside.
    //
    // ⚠ It never returns 0. A zero state would drop the trailing mark for a beat
    // and the text would visibly jump; the dots are there to say "still working",
    // and an empty frame says the opposite.

    /// Elapsed expressed as "n dot cycles' worth", derived from the constant so
    /// the spec survives retuning the cycle speed.
    fn after_dot_cycles(n: u64) -> Duration {
        Duration::from_millis(n * DOT_CYCLE_MS)
    }

    #[test]
    fn the_wait_opens_with_a_single_dot() {
        assert_eq!(thinking_dots(Duration::ZERO), 1);
    }

    #[test]
    fn each_cycle_adds_a_dot() {
        assert_eq!(thinking_dots(after_dot_cycles(1)), 2);
        assert_eq!(thinking_dots(after_dot_cycles(2)), 3);
    }

    #[test]
    fn the_dots_wrap_back_to_one_after_three() {
        assert_eq!(
            thinking_dots(after_dot_cycles(3)),
            1,
            "the fourth cycle restarts the run — not a fourth dot, not an empty frame"
        );
        assert_eq!(thinking_dots(after_dot_cycles(4)), 2);
    }

    #[test]
    fn a_dot_holds_for_its_whole_cycle() {
        // A step function, not a continuous one: mid-cycle must look identical to
        // the start of it, or the dots stutter between frames instead of ticking.
        let mid = after_dot_cycles(1) + Duration::from_millis(DOT_CYCLE_MS / 2);
        assert_eq!(thinking_dots(mid), thinking_dots(after_dot_cycles(1)));
    }

    #[test]
    fn the_dot_count_never_leaves_one_through_three() {
        // THE CONTRACT PIN — the cases above only sample the first few cycles.
        // A wrong modulus (or an off-by-one on the `+ 1`) shows up here as a 0 or
        // a 4, at some cycle nobody thought to write a case for. Sweeps well past
        // MAX_THINKING_MS so no real ponder can reach an untested value.
        for tenth_second in 0..=100 {
            let elapsed = Duration::from_millis(tenth_second * 100);
            let dots = thinking_dots(elapsed);
            assert!(
                (1..=3).contains(&dots),
                "at {elapsed:?} the dot count was {dots} — outside 1..=3"
            );
        }
    }

    // ── G18 · the spell pulses instead of typing ─────────────────────────────
    //
    // ⚠⚠ THESE ARE WRITTEN TO SURVIVE TUNING, AND THAT IS DELIBERATE. Both of
    // G18's numbers — the cadence and the trough floor — are explicitly "tune it
    // by eye" values. Pin a brightness at a millisecond and every visual
    // adjustment turns the suite red, which is the exact trap `ui/screens.rs`
    // already warns about for layout: *"pinning them would make every visual
    // tweak a failure"*.
    //
    // So nothing below asserts a specific value at a specific time. What they
    // pin is SHAPE — the four things that would be bugs at ANY cadence:
    // never black · reaches full · ramps rather than blinks · slower than the
    // cursor. Retune freely; these should stay green.

    /// A trough dark enough to read as "the spell disappeared" rather than "the
    /// spell dimmed". ⚠ This is a floor on the FLOOR, not the floor itself — the
    /// real const is expected to sit far above it (`MIN_FLICKER_VALUE` is 160,
    /// and G18 is the same family of hazard). It exists only so the test can
    /// fail loudly on a naive `0..=255` ramp without dictating the tuning.
    const LEGIBLE_TROUGH: u8 = 32;

    /// Long enough to hold several cycles at any cadence a person would call
    /// "slow", short enough that the sweep stays cheap.
    const PULSE_SWEEP_MS: u64 = 6_000;

    /// Sample the pulse densely across `PULSE_SWEEP_MS`.
    fn pulse_sweep(animations_enabled: bool) -> Vec<u8> {
        (0..PULSE_SWEEP_MS)
            .step_by(10)
            .map(|ms| pulse_intensity(Duration::from_millis(ms), animations_enabled))
            .collect()
    }

    #[test]
    fn animations_off_holds_the_spell_at_full_brightness() {
        // ⚠ THE REST VALUE IS FULL, NOT ZERO — the same call `flicker_intensity`
        // makes, and for the same reason: "no pulse" means an UNDIMMED spell,
        // not a dark one. Returning 0 here would leave the accessibility user
        // staring at a spell frozen at the trough, which on `palette.bg` is an
        // empty box — the accessibility gate would have caused the very bug the
        // floor exists to prevent.
        for value in pulse_sweep(false) {
            assert_eq!(
                value,
                u8::MAX,
                "animations off must hold the spell lit, not dim it"
            );
        }
    }

    #[test]
    fn the_pulse_never_falls_to_black() {
        // ⚠⚠ THE HARD RULE OF G18. `glow(0)` is black on every theme —
        // `theme.rs::glow_at_zero_intensity_is_black_for_every_theme` pins it —
        // so a naive `0..=255` ramp makes the spell VANISH at the trough. On a
        // `palette.bg` of (7,4,6) that is an empty box mid-ponder, which reads
        // as a crash rather than as breathing.
        let trough = pulse_sweep(true)
            .into_iter()
            .min()
            .expect("the sweep must not be empty");

        assert_ne!(trough, 0, "the spell must never go fully black");
        assert!(
            trough >= LEGIBLE_TROUGH,
            "the trough reached {trough}, close enough to black to read as the \
             spell disappearing rather than dimming"
        );
    }

    #[test]
    fn the_pulse_peaks_at_the_colour_the_spell_already_has() {
        // `glow(255)` returns `peak` unscaled, and every palette's `peak` tuple
        // IS its `accent`. So full intensity is exactly the colour the spell
        // wears today, and peaking below it would quietly RESTYLE the ponder
        // instead of animating it — undoing the settled accent-spell-vs-white-
        // reply decision at `ask.rs:116-120` as a side effect of an animation
        // tweak.
        let peak = pulse_sweep(true)
            .into_iter()
            .max()
            .expect("the sweep must not be empty");

        assert_eq!(
            peak,
            u8::MAX,
            "the pulse must reach full brightness, or the spell never wears its \
             own accent colour"
        );
    }

    #[test]
    fn the_pulse_ramps_instead_of_blinking() {
        // He said "kinda slow blink" but the word that decides the shape is
        // "glow". A square wave visits exactly TWO brightnesses; a ramp passes
        // through the range. Counting distinct values tells the two apart
        // without knowing the cadence.
        //
        // 📌 This is also what stops a CONSTANT function passing the whole
        // section — every other test here is satisfied by `|_, _| u8::MAX`.
        let mut visited = pulse_sweep(true);
        visited.sort_unstable();
        visited.dedup();

        assert!(
            visited.len() >= 16,
            "the pulse visited only {} distinct brightnesses across {PULSE_SWEEP_MS}ms — \
             that is a blink, not a glow",
            visited.len()
        );
    }

    #[test]
    fn the_pulse_comes_back_down_and_rises_again() {
        // ⚠⚠ ADDED 2026-08-04, AND IT IS A HOLE IN THIS SECTION RATHER THAN NEW
        // SCOPE. The first implementation exposed it: a ONE-SHOT RAMP — climb to
        // full and stay there forever — satisfies every other test here.
        // `ramps_instead_of_blinking` sees plenty of distinct brightnesses on
        // the way up, and a monotone function has no direction changes at all,
        // so the cadence test reads it as "slower than the whole sweep" and
        // waves it through. Both would have gone green on a spell that brightens
        // once and then just sits there.
        //
        // Breathing is the part that REPEATS. Two peaks and two troughs is the
        // smallest evidence that it does.
        let mut runs = pulse_sweep(true);
        runs.dedup(); // collapse plateaus, so a flat peak still reads as one turn

        let (peaks, troughs) = runs.windows(3).fold((0, 0), |(up, down), window| {
            let (before, here, after) = (window[0], window[1], window[2]);
            if here > before && here > after {
                (up + 1, down)
            } else if here < before && here < after {
                (up, down + 1)
            } else {
                (up, down)
            }
        });

        assert!(
            peaks >= 2 && troughs >= 2,
            "across {PULSE_SWEEP_MS}ms the pulse turned around {peaks} time(s) at the top \
             and {troughs} at the bottom — a spell that brightens once and stays lit is \
             not breathing"
        );
    }

    #[test]
    fn the_pulse_glows_rather_than_snapping() {
        // The other shape `ramps_instead_of_blinking` cannot rule out: a
        // SAWTOOTH climbs gently and then snaps back to the floor in a single
        // frame — visiting exactly as many distinct brightnesses on the way up
        // as a triangle does, so the distinct-value count says nothing. On
        // screen that is a flash-and-fade, not a glow.
        //
        // Stated as a fraction of the pulse's OWN range so it survives retuning
        // the floor, the peak and the cadence together.
        let samples = pulse_sweep(true);
        let trough = *samples.iter().min().expect("the sweep must not be empty");
        let peak = *samples.iter().max().expect("the sweep must not be empty");
        let range = peak.saturating_sub(trough) as u32;

        let biggest_step = samples
            .windows(2)
            .map(|window| window[0].abs_diff(window[1]) as u32)
            .max()
            .expect("the sweep must not be empty");

        assert!(
            biggest_step * 4 <= range,
            "one frame moved the spell by {biggest_step} of its {range}-wide range — \
             that is a snap, not a glow"
        );
    }

    #[test]
    fn the_pulse_breathes_slower_than_the_cursor_blinks() {
        // "Slow" is his word and the spec calls it the point: at the cursor's
        // own cadence this would read as a SECOND CURSOR sitting beside the real
        // one rather than as the spell breathing — and the two would be on
        // screen together, which is what makes the collision visible.
        //
        // A full cycle has exactly two direction changes, so counting local
        // extremes bounds the period without this test needing to know the
        // cadence const. ⚠ Deliberately a LOOSE bound — it fails a pulse that is
        // cursor-fast, not one that is merely faster than you last left it.
        let samples = pulse_sweep(true);
        let reversals = samples
            .windows(3)
            .filter(|window| {
                let (before, here, after) = (window[0], window[1], window[2]);
                (here > before && here > after) || (here < before && here < after)
            })
            .count() as u64;

        let cycles = reversals / 2;

        // No turns at all means the pulse is slower than the whole sweep, which
        // is certainly slow enough — `checked_div` gives `None` on the zero and
        // the fallback says exactly that.
        let period_ms = PULSE_SWEEP_MS.checked_div(cycles).unwrap_or(PULSE_SWEEP_MS);

        assert!(
            period_ms > CURSOR_BLINK_MS,
            "the pulse cycles about every {period_ms}ms against the cursor's \
             {CURSOR_BLINK_MS}ms — that reads as a second cursor, not as breathing"
        );
    }
}

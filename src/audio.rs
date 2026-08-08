//! Audio via `kira` — a looping dread ambience + one-shot scare stings.
//!
//! Feature-gated: with `--features audio` this drives real sound; without it,
//! `Audio` is a **no-op stub with the same API**, so `main` never needs a
//! `#[cfg]`. `--no-sound` (passed at runtime) also forces the silent path even
//! in an audio build. kira owns its own realtime audio thread, so nothing here
//! spawns threads — we just hand it sounds to play.

pub const RANDOM_AUDIO_MIN_SECONDS: u64 = 40;
pub const RANDOM_AUDIO_MAX_SECONDS: u64 = 90;

#[cfg(feature = "audio")]
const SILENCE_DB: f32 = -60.0; // mirrors kira's Decibels::SILENCE

pub const MAX_ALLOWED_VOLUME: u8 = 100;

#[cfg(feature = "audio")]
use std::io::Cursor;
use std::time::Duration;

#[cfg(feature = "audio")]
use kira::{
    AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Tween,
    sound::static_sound::StaticSoundData,
};

/// A one-shot sound. **Not** feature-gated — the pure logic decides *which*
/// sound to play without ever depending on kira, so it stays testable.
///
/// Two different things fire these, and the split matters:
///
/// - **State-triggered** ([`JumpScare`](Self::JumpScare), [`Thunder`](Self::Thunder)):
///   `App` queues one when the session changes state; `main` drains it each tick.
///   They land on a specific beat and must never fire at random.
/// - **Timer-driven** (everything in [`ALL_RANDOM_CUES`](Self::ALL_RANDOM_CUES)):
///   `main`'s own clock fires these on a [`random_audio_interval`] to keep the
///   room feeling inhabited. Nothing in `App` knows about them.
///
/// [`next_cue`] walks only the second group, so the two can never cross.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioCue {
    /// SUED replies — the jump-scare sting (`assets/jump_scare.ogg`).
    JumpScare,
    /// The decoy buffer is running out of characters (`assets/thunder.ogg`).
    Thunder,
    /// Demonic laughter (`assets/laugh.ogg`).
    Laugh,
    /// Muttered summoning (`assets/incantation_1.ogg`).
    Incantation1,
    /// Muttered summoning, second take (`assets/incantation_2.ogg`).
    Incantation2,
    /// A distant shriek (`assets/scream.ogg`).
    Scream,
    /// A single toll (`assets/bell.ogg`).
    Bell,
}

impl AudioCue {
    /// The timer-driven rotation, in play order. Deliberately *not* the
    /// declaration order of the enum: the two incantations sit apart so a lap
    /// never plays them back to back, which is the one pairing that sounds like
    /// a repeat rather than two different sounds.
    ///
    /// Adding a variant here puts it in the rotation; adding one to the enum
    /// alone leaves it state-triggered. `the_rotation_never_fires_a_state_triggered_cue`
    /// is what catches a new cue landing in the wrong group.
    // In the silent build the only caller is `next_cue`, which is itself only
    // reached from tests — `dead_code` doesn't count test usage, so it fires.
    #[cfg_attr(not(feature = "audio"), allow(dead_code))]
    const ALL_RANDOM_CUES: [AudioCue; 5] = [
        AudioCue::Laugh,
        AudioCue::Incantation1,
        AudioCue::Scream,
        AudioCue::Incantation2,
        AudioCue::Bell,
    ];
}

/// How long to wait before the next timer-driven cue, from a `roll` in `[0, 1)`.
///
/// Linear across `RANDOM_AUDIO_MIN_SECONDS..=RANDOM_AUDIO_MAX_SECONDS`: `0.0` is
/// the floor, `1.0` the ceiling. Pure so it's testable with no sound card.
pub fn random_audio_interval(roll: f32) -> Duration {
    let span = RANDOM_AUDIO_MAX_SECONDS - RANDOM_AUDIO_MIN_SECONDS;
    Duration::from_secs(RANDOM_AUDIO_MIN_SECONDS + (roll * span as f32) as u64)
}

/// Advances `cursor` one step around [`AudioCue::ALL_RANDOM_CUES`] and returns
/// the cue it was pointing at.
///
/// **Round-robin, not a random draw**, which is a deliberate reversal. Drawing
/// independently from five cues clumps badly in a session that only fires a
/// dozen of them: the same sting lands three times running while another never
/// shows up at all. Walking the list guarantees every cue is heard once per lap.
/// The *interval* stays random ([`random_audio_interval`]), so the rotation
/// still never sounds metronomic.
///
/// The caller owns the cursor, which keeps this pure and kira-free — same tier
/// as [`random_audio_interval`], so it compiles and is tested in both the audio
/// and the silent build. Indexing goes through `%` as well as the advance, so a
/// cursor seeded out of range (a random start, say) wraps instead of panicking.
// Ungated so the rotation is specified once and tested in both builds — but in
// the silent build nothing in `main` reaches it (the stub `play_next_random_cue`
// is a no-op), so outside `cfg(test)` it is genuinely unreachable there.
#[cfg_attr(not(feature = "audio"), allow(dead_code))]
pub fn next_cue(cursor: &mut usize) -> AudioCue {
    let cues = AudioCue::ALL_RANDOM_CUES;

    let cue = cues[*cursor % cues.len()];
    *cursor = (*cursor + 1) % cues.len();

    cue
}

/// Converts the config's `0`–`100` volume **percent** into the **decibels** kira
/// speaks, which is the whole reason this function exists: the two units look
/// interchangeable and aren't. `Decibels(0.0)` is *unchanged*, not silent — so
/// handing kira a raw `0.5` compiles happily and means **+0.5 dB ≈ 106%**, a
/// slight boost rather than half volume.
///
/// So `100` maps to `0.0` (unity gain, the asset at its mastered level) and
/// every step down is negative, at `20 · log10` of the amplitude ratio: `50` is
/// about `-6` dB, `10` about `-20` dB.
///
/// Two inputs get special handling:
///
/// - **`0`** returns [`SILENCE_DB`] rather than going through the logarithm,
///   because `log10(0)` is `-∞` and would poison everything downstream.
/// - **Anything above [`MAX_ALLOWED_VOLUME`]** is clamped to it. `percent` is a
///   `u8`, so `101..=255` are representable, and they would map to *positive*
///   dB — amplifying the signal past the level it was mastered at, which clips.
///   `Configuration` already stops the slider at 100, but that guarantee lives
///   in another module and this one refuses to depend on it.
///
/// Pure and kira-free on purpose (like [`random_audio_interval`]): it's arithmetic, so
/// it compiles and is tested in both the audio and the silent build, with no
/// sound card anywhere. Only the caller wraps the result in `Decibels(..)`.
#[cfg(feature = "audio")]
pub fn volume_db(percent: u8) -> f32 {
    if percent == 0 {
        return SILENCE_DB;
    }

    let capped_percent_as_f32 = percent.min(MAX_ALLOWED_VOLUME) as f32;

    // The `100.0` is the *definition of percent*, not the volume ceiling — they
    // happen to be the same number today for unrelated reasons. Don't replace it
    // with `MAX_ALLOWED_VOLUME`: if a boost mode ever raises the ceiling to 150,
    // this must stay 100, or 150% would silently mean unity gain again.
    let ratio = capped_percent_as_f32 / 100.0;

    // 20·log10, not 10·log10, because we're scaling AMPLITUDE and acoustic power
    // goes as amplitude² — squaring inside the log comes out as the factor of 2
    // (10 for "deci", × 2 for the square). It's the definition of the unit, not
    // a tunable, which is why it stays a literal instead of becoming a constant.
    20.0 * ratio.log10()
}

// ── Silent build: no `audio` feature (or `--no-sound`) ──────────────────────
// Same surface as the real thing, every method a no-op. This is what keeps the
// crate building with no ALSA headers and `main` free of `#[cfg]`.
#[cfg(not(feature = "audio"))]
pub struct Audio;

#[cfg(not(feature = "audio"))]
impl Audio {
    pub fn new(_enabled: bool) -> anyhow::Result<Self> {
        Ok(Audio)
    }

    pub fn start_background_ambience(&mut self) {}

    pub fn play(&mut self, _cue: AudioCue) {}

    pub fn set_volume(&mut self, _percent: u8) {}

    pub fn play_next_random_cue(&mut self) {}
}

#[cfg(feature = "audio")]
struct Player {
    manager: AudioManager,
    ambience_sound: StaticSoundData,
    jump_scare_sound: StaticSoundData,
    thunder_sound: StaticSoundData,
    laugh_sound: StaticSoundData,
    incantation_1: StaticSoundData,
    incantation_2: StaticSoundData,
    scream: StaticSoundData,
    bell: StaticSoundData,
}

#[cfg(feature = "audio")]
pub struct Audio {
    player: Option<Player>,
    /// Cursor into [`AudioCue::ALL_RANDOM_CUES`], owned here but advanced by the
    /// pure [`next_cue`] so the rotation itself stays testable without a device.
    next_random_cue_index: usize,
}

#[cfg(feature = "audio")]
impl Audio {
    pub fn new(audio_enabled: bool) -> anyhow::Result<Self> {
        if !audio_enabled {
            return Ok(Audio {
                player: None,
                next_random_cue_index: 0,
            });
        }

        let audio_manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;

        let ambience_sound =
            StaticSoundData::from_cursor(Cursor::new(include_bytes!("../assets/ambience.ogg")))?;
        let laugh_sound =
            StaticSoundData::from_cursor(Cursor::new(include_bytes!("../assets/laugh.ogg")))?;
        let jump_scare_sound =
            StaticSoundData::from_cursor(Cursor::new(include_bytes!("../assets/jump_scare.ogg")))?;

        let thunder_sound =
            StaticSoundData::from_cursor(Cursor::new(include_bytes!("../assets/thunder.ogg")))?;

        let incantation_1 = StaticSoundData::from_cursor(Cursor::new(include_bytes!(
            "../assets/incantation_1.ogg"
        )))?;

        let incantation_2 = StaticSoundData::from_cursor(Cursor::new(include_bytes!(
            "../assets/incantation_2.ogg"
        )))?;

        let scream =
            StaticSoundData::from_cursor(Cursor::new(include_bytes!("../assets/scream.ogg")))?;

        let bell = StaticSoundData::from_cursor(Cursor::new(include_bytes!("../assets/bell.ogg")))?;

        let player = Player {
            manager: audio_manager,
            ambience_sound,
            laugh_sound,
            jump_scare_sound,
            thunder_sound,
            incantation_1,
            incantation_2,
            scream,
            bell,
        };

        Ok(Audio {
            player: Some(player),
            next_random_cue_index: 0,
        })
    }

    pub fn start_background_ambience(&mut self) {
        let Some(player) = &mut self.player else {
            return;
        };

        let sound = player.ambience_sound.clone().loop_region(..);
        let _ = player.manager.play(sound);
    }

    pub fn play(&mut self, audio_cue: AudioCue) {
        let Some(player) = &mut self.player else {
            return;
        };

        match audio_cue {
            AudioCue::JumpScare => {
                let _ = player.manager.play(player.jump_scare_sound.clone());
            }
            AudioCue::Thunder => {
                let _ = player.manager.play(player.thunder_sound.clone());
            }
            AudioCue::Laugh => {
                let _ = player.manager.play(player.laugh_sound.clone());
            }

            AudioCue::Incantation1 => {
                let _ = player.manager.play(player.incantation_1.clone());
            }

            AudioCue::Incantation2 => {
                let _ = player.manager.play(player.incantation_2.clone());
            }

            AudioCue::Scream => {
                let _ = player.manager.play(player.scream.clone());
            }

            AudioCue::Bell => {
                let _ = player.manager.play(player.bell.clone());
            }
        }
    }

    /// Fires the next cue in the rotation. Named `random_cue` rather than
    /// `ambience` on purpose — [`start_background_ambience`](Self::start_background_ambience)
    /// is the looping dread bed; these are one-shot stings layered over it.
    pub fn play_next_random_cue(&mut self) {
        let cue = next_cue(&mut self.next_random_cue_index);
        self.play(cue);
    }

    pub fn set_volume(&mut self, percent: u8) {
        let Some(player) = &mut self.player else {
            return;
        };

        player
            .manager
            .main_track()
            .set_volume(Decibels(volume_db(percent)), Tween::default());
    }
}

#[cfg(test)]
mod cadence_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn the_interval_sits_at_the_floor_for_roll_zero() {
        assert_eq!(
            random_audio_interval(0.0),
            Duration::from_secs(RANDOM_AUDIO_MIN_SECONDS)
        );
    }

    #[test]
    fn the_interval_reaches_the_ceiling_for_roll_one() {
        assert_eq!(
            random_audio_interval(1.0),
            Duration::from_secs(RANDOM_AUDIO_MAX_SECONDS)
        );
    }

    #[test]
    fn the_interval_lands_midway_for_a_half_roll() {
        assert_eq!(
            random_audio_interval(0.5),
            Duration::from_secs(
                RANDOM_AUDIO_MIN_SECONDS
                    + (RANDOM_AUDIO_MAX_SECONDS - RANDOM_AUDIO_MIN_SECONDS) / 2
            )
        );
    }

    #[test]
    fn the_interval_stays_within_bounds_across_the_rng_range() {
        // `rand::random::<f32>()` yields [0.0, 1.0). Sweep it: every result must
        // land inside the floor..=ceiling window. The bounds are read from the
        // constants rather than written as literals so that retuning the range
        // (40–120 → 40–90, and whatever comes next) can never quietly leave this
        // asserting a window WIDER than the real one, which passes while checking
        // nothing.
        for i in 0..=100 {
            let roll = i as f32 / 100.0;
            let secs = random_audio_interval(roll).as_secs();
            assert!(
                (RANDOM_AUDIO_MIN_SECONDS..=RANDOM_AUDIO_MAX_SECONDS).contains(&secs),
                "roll {roll} produced {secs}s, outside {RANDOM_AUDIO_MIN_SECONDS}..={RANDOM_AUDIO_MAX_SECONDS}"
            );
        }
    }
}

// ── next_cue: the timer-driven rotation ───────────────────────────────────────
// Ungated like the cadence, and for the same reason: the cursor walk is arithmetic
// over a const array, so it's tested in BOTH builds with no sound card. This is
// the whole point of `next_cue` taking `&mut usize` instead of living on `Audio`
// — from the outside, `play_next_random_cue` swallows its choice into kira and no
// test can see which cue came out.
#[cfg(test)]
mod rotation_tests {
    use super::*;

    /// Walks `n` steps from a fresh cursor.
    fn walk(n: usize) -> Vec<AudioCue> {
        let mut cursor = 0;
        (0..n).map(|_| next_cue(&mut cursor)).collect()
    }

    #[test]
    fn a_full_lap_plays_every_cue_in_the_rotation_exactly_once() {
        // The property that motivated round-robin in the first place. A random
        // draw over five cues clumps: play-testing showed the same sting three
        // times running while others never appeared. One lap, one of each.
        let lap = walk(AudioCue::ALL_RANDOM_CUES.len());

        for cue in AudioCue::ALL_RANDOM_CUES {
            let plays = lap.iter().filter(|&&played| played == cue).count();
            assert_eq!(
                plays, 1,
                "{cue:?} played {plays}× in one lap {lap:?}, want 1"
            );
        }
    }

    #[test]
    fn a_lap_walks_the_rotation_in_its_declared_order() {
        // Stronger than "one of each", and NOT redundant with it: advancing the
        // cursor by 2 instead of 1 still visits all five and still wraps after
        // five steps (gcd(2, 5) == 1), so the coverage test above passes while
        // the running order silently becomes Laugh, Scream, Bell, Incantation1,
        // Incantation2 — the two incantations back to back, which is precisely
        // what `ALL_RANDOM_CUES` is ordered to avoid. Only pinning the sequence
        // catches that.
        assert_eq!(
            walk(AudioCue::ALL_RANDOM_CUES.len()),
            AudioCue::ALL_RANDOM_CUES.to_vec(),
            "the rotation drifted from the order declared in ALL_RANDOM_CUES"
        );
    }

    #[test]
    fn the_lap_wraps_instead_of_running_off_the_end() {
        // The step after the last cue is the first cue again — and the second lap
        // must be identical to the first, not merely non-panicking. An `idx + 1`
        // that wrapped to 1 instead of 0 would still be in range and still play
        // sounds forever, silently starving the first cue after lap one.
        let two_laps = walk(AudioCue::ALL_RANDOM_CUES.len() * 2);
        let (first, second) = two_laps.split_at(AudioCue::ALL_RANDOM_CUES.len());

        assert_eq!(first, second, "the second lap diverged from the first");
    }

    #[test]
    fn the_rotation_never_fires_a_state_triggered_cue() {
        // The invariant that keeps the two trigger sources disjoint. `JumpScare`
        // has to land exactly when SUED answers and `Thunder` when the decoy
        // buffer runs dry; either one going off on the ambience timer wrecks the
        // beat the whole prank is built on. Swept over several laps so a cue that
        // only surfaces late still gets caught.
        for cue in walk(AudioCue::ALL_RANDOM_CUES.len() * 3) {
            assert!(
                !matches!(cue, AudioCue::JumpScare | AudioCue::Thunder),
                "{cue:?} is state-triggered but turned up in the timer rotation"
            );
        }
    }

    #[test]
    fn a_cursor_seeded_past_the_end_wraps_instead_of_panicking() {
        // `next_cue` takes whatever `usize` the caller hands it, and the obvious
        // next feature — starting each session at a random offset so the order
        // isn't identical every run — is exactly what would hand it one past the
        // end. Indexing goes through `%` so that's a wrap, not an out-of-bounds
        // panic mid-session.
        let mut cursor = 999;
        let cue = next_cue(&mut cursor);

        assert!(
            AudioCue::ALL_RANDOM_CUES.contains(&cue),
            "an out-of-range cursor produced {cue:?}, which is not in the rotation"
        );
        assert!(
            cursor < AudioCue::ALL_RANDOM_CUES.len(),
            "the cursor stayed out of range at {cursor}"
        );
    }
}

// ── volume_db: the percent → decibels seam ─────────────────────────────────────
// Ungated on purpose, exactly like `random_audio_interval`: it's arithmetic, it touches
// no kira type, so it compiles and is tested in BOTH the audio and silent builds
// and needs no sound card. The kira edge wraps the result in `Decibels(..)`.
//
// The mapping is the textbook one — `20 * log10(percent / 100)` — because the
// slider genuinely means "percent of amplitude". The one thing it cannot do is
// take `log10(0)` (that's -infinity), so `0` returns `SILENCE_DB`, a floor that
// mirrors kira's own `Decibels::SILENCE` (-60.0) without depending on the type.
//
// Assertions use a tolerance rather than `==`: these are f32 and the expected
// values are irrational-ish, so pinning exact bits would be testing the FPU.
#[cfg(all(test, feature = "audio"))]
mod volume_tests {
    use super::{SILENCE_DB, volume_db};

    /// Close enough for decibels — a hundredth of a dB is far below audible.
    fn approx(actual: f32, expected: f32) -> bool {
        (actual - expected).abs() < 0.01
    }

    /// The slider's real stops: 0, 10, 20 … 100, per `VOLUME_STEP` in `config.rs`.
    const SLIDER_STOPS: [u8; 11] = [0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100];

    #[test]
    fn full_volume_is_unity_gain() {
        // 100% must be 0 dB — *unchanged*, not "loud". This is the one that
        // catches the classic kira misreading: `Decibels(0.5)` is +0.5 dB, a
        // slight BOOST, not half volume.
        let db = volume_db(100);
        assert!(approx(db, 0.0), "100% mapped to {db} dB, want 0.0");
    }

    #[test]
    fn zero_percent_drops_to_the_silence_floor() {
        // The special case that has to exist: log10(0) is -infinity, which would
        // poison every downstream calculation. It returns the floor instead.
        let db = volume_db(0);
        assert!(
            approx(db, SILENCE_DB),
            "0% mapped to {db} dB, want the {SILENCE_DB} floor"
        );
        assert!(db.is_finite(), "0% produced {db} — a non-finite dB value");
    }

    #[test]
    fn half_volume_is_about_six_db_down() {
        // Halving the amplitude is ~-6.02 dB. If this comes out as -50 or -30,
        // the curve is a dB-space lerp, not the amplitude conversion we chose.
        let db = volume_db(50);
        assert!(approx(db, -6.02), "50% mapped to {db} dB, want ~-6.02");
    }

    #[test]
    fn a_tenth_of_the_volume_is_about_twenty_db_down() {
        // Every factor-of-10 drop in amplitude is another -20 dB — the property
        // that makes this curve the standard one.
        let db = volume_db(10);
        assert!(approx(db, -20.0), "10% mapped to {db} dB, want ~-20.0");
    }

    #[test]
    fn volume_never_amplifies() {
        // Nothing on the slider may exceed unity gain. Positive dB would boost
        // the signal past the mastered level of the asset and clip it.
        //
        // Swept across the WHOLE `u8`, not just the slider stops: `percent` can
        // represent 101..=255, and the clamp that turns those away lives here,
        // not in `Configuration`. A future `--volume 200` must be turned down,
        // and the failure mode — audible distortion — is one no other test sees.
        for percent in 0..=u8::MAX {
            let db = volume_db(percent);
            assert!(db <= 0.0, "{percent}% boosted the signal to {db} dB");
        }
    }

    #[test]
    fn above_full_volume_is_pinned_to_unity_gain() {
        // Not merely "doesn't amplify" — an out-of-range percent must land on
        // exactly the same volume as 100%, so overshooting reads as "full",
        // never as some other level.
        let full = volume_db(100);
        for percent in [101, 150, u8::MAX] {
            let db = volume_db(percent);
            assert!(
                approx(db, full),
                "{percent}% mapped to {db} dB, want the {full} dB of full volume"
            );
        }
    }

    #[test]
    fn volume_never_sinks_below_the_silence_floor() {
        // The floor is a floor: no stop may land under it, so the render of a
        // volume change can never ask kira for something quieter than silence.
        for percent in SLIDER_STOPS {
            let db = volume_db(percent);
            assert!(
                db >= SILENCE_DB,
                "{percent}% mapped to {db} dB, below the {SILENCE_DB} floor"
            );
        }
    }

    #[test]
    fn louder_percent_is_never_quieter() {
        // Strictly increasing across the stops. `[←]` must always get quieter
        // and `[→]` louder — an inversion anywhere makes the slider feel broken.
        let curve: Vec<f32> = SLIDER_STOPS.iter().map(|&p| volume_db(p)).collect();
        for pair in curve.windows(2) {
            assert!(pair[0] < pair[1], "the volume curve inverted: {curve:?}");
        }
    }

    #[test]
    fn every_slider_stop_is_audibly_distinct() {
        // Each keypress must *do* something. A mapping that rounds or clamps
        // could technically stay monotonic while several stops sound identical;
        // 0.5 dB apart is the loosest bound that still guarantees a real step.
        let curve: Vec<f32> = SLIDER_STOPS.iter().map(|&p| volume_db(p)).collect();
        for pair in curve.windows(2) {
            assert!(
                pair[1] - pair[0] >= 0.5,
                "two neighbouring stops are the same volume: {curve:?}"
            );
        }
    }
}

// Only meaningful in an audio build: the stub `Audio` is unconditionally silent
// and has no `player` to inspect. There is deliberately no `new(true)` test —
// that one needs a real sound card, which CI doesn't have.
#[cfg(all(test, feature = "audio"))]
mod tests {
    use super::*;

    #[test]
    fn a_disabled_audio_holds_no_player() {
        let audio = Audio::new(false).expect("a silent Audio must build on a box with no sound");

        assert!(
            audio.player.is_none(),
            "--no-sound must not open the audio device at all"
        );
    }

    #[test]
    fn a_silent_audio_stays_quiet_instead_of_panicking() {
        let mut audio = Audio::new(false).unwrap();

        audio.start_background_ambience();
        audio.play(AudioCue::JumpScare);
        audio.play(AudioCue::Laugh);
        audio.play_next_random_cue();
        // `set_volume` reaches for `main_track()` on the manager — the one call
        // here that would touch a device that was never opened. `--no-sound` has
        // to swallow it like the rest.
        audio.set_volume(50);
    }
}

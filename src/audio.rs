//! Audio via `kira` — a looping dread ambience + one-shot scare stings.
//!
//! Feature-gated: with `--features audio` this drives real sound; without it,
//! `Audio` is a **no-op stub with the same API**, so `main` never needs a
//! `#[cfg]`. `--no-sound` (passed at runtime) also forces the silent path even
//! in an audio build. kira owns its own realtime audio thread, so nothing here
//! spawns threads — we just hand it sounds to play.

pub const LAUGH_MIN_SECS: u64 = 40;
pub const LAUGH_MAX_SECS: u64 = 120;
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

/// A one-shot sound triggered by a state change. `App` queues one; `main` drains
/// it each tick and plays it. **Not** feature-gated — the pure app logic decides
/// *which* sound to play without ever depending on kira, so it stays testable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioCue {
    /// SUED replies — the jump-scare sting (`assets/jump_scare.ogg`).
    JumpScare,
    /// random demonic laughter (`assets/laugh.ogg`) running in background.
    Laugh,
}

pub fn laugh_interval(roll: f32) -> Duration {
    let span = LAUGH_MAX_SECS - LAUGH_MIN_SECS;
    Duration::from_secs(LAUGH_MIN_SECS + (roll * span as f32) as u64)
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
/// Pure and kira-free on purpose (like [`laugh_interval`]): it's arithmetic, so
/// it compiles and is tested in both the audio and the silent build, with no
/// sound card anywhere. Only the caller wraps the result in `Decibels(..)`.
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

    pub fn start_ambience(&mut self) {}

    pub fn play(&mut self, _cue: AudioCue) {}

    pub fn set_volume(&mut self, _percent: u8) {}
}

#[cfg(feature = "audio")]
struct Player {
    manager: AudioManager,
    ambience_sound: StaticSoundData,
    laugh_sound: StaticSoundData,
    jump_scare_sound: StaticSoundData,
}

#[cfg(feature = "audio")]
pub struct Audio {
    player: Option<Player>,
}

#[cfg(feature = "audio")]
impl Audio {
    pub fn new(audio_enabled: bool) -> anyhow::Result<Self> {
        if !audio_enabled {
            return Ok(Audio { player: None });
        }

        let audio_manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;

        let ambience_sound =
            StaticSoundData::from_cursor(Cursor::new(include_bytes!("../assets/ambience.ogg")))?;
        let laugh_sound =
            StaticSoundData::from_cursor(Cursor::new(include_bytes!("../assets/laugh.ogg")))?;
        let jump_scare_sound =
            StaticSoundData::from_cursor(Cursor::new(include_bytes!("../assets/jump_scare.ogg")))?;

        let player = Player {
            manager: audio_manager,
            ambience_sound,
            laugh_sound,
            jump_scare_sound,
        };

        Ok(Audio {
            player: Some(player),
        })
    }

    pub fn start_ambience(&mut self) {
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
            AudioCue::Laugh => {
                let _ = player.manager.play(player.laugh_sound.clone());
            }
            AudioCue::JumpScare => {
                let _ = player.manager.play(player.jump_scare_sound.clone());
            }
        }
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
    fn laugh_interval_sits_at_the_floor_for_roll_zero() {
        assert_eq!(laugh_interval(0.0), Duration::from_secs(40));
    }

    #[test]
    fn laugh_interval_reaches_the_ceiling_for_roll_one() {
        assert_eq!(laugh_interval(1.0), Duration::from_secs(120));
    }

    #[test]
    fn laugh_interval_lands_midway_for_a_half_roll() {
        assert_eq!(laugh_interval(0.5), Duration::from_secs(80));
    }

    #[test]
    fn laugh_interval_stays_within_bounds_across_the_rng_range() {
        // `rand::random::<f32>()` yields [0.0, 1.0). Sweep it: every result must
        // land in [40, 120]s — never quieter than the floor, never longer than the ceiling.
        for i in 0..=100 {
            let roll = i as f32 / 100.0;
            let secs = laugh_interval(roll).as_secs();
            assert!(
                (40..=120).contains(&secs),
                "roll {roll} produced {secs}s, outside 40..=120"
            );
        }
    }
}

// ── volume_db: the percent → decibels seam ─────────────────────────────────────
// Ungated on purpose, exactly like `laugh_interval`: it's arithmetic, it touches
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
#[cfg(test)]
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

        audio.start_ambience();
        audio.play(AudioCue::JumpScare);
        audio.play(AudioCue::Laugh);
        // `set_volume` reaches for `main_track()` on the manager — the one call
        // here that would touch a device that was never opened. `--no-sound` has
        // to swallow it like the rest.
        audio.set_volume(50);
    }
}

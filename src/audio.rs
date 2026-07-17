//! Audio via `kira` — a looping dread ambience + one-shot scare stings.
//!
//! Feature-gated: with `--features audio` this drives real sound; without it,
//! `Audio` is a **no-op stub with the same API**, so `main` never needs a
//! `#[cfg]`. `--no-sound` (passed at runtime) also forces the silent path even
//! in an audio build. kira owns its own realtime audio thread, so nothing here
//! spawns threads — we just hand it sounds to play.

pub const LAUGH_MIN_SECS: u64 = 40;
pub const LAUGH_MAX_SECS: u64 = 120;

#[cfg(feature = "audio")]
use std::io::Cursor;
use std::time::Duration;

#[cfg(feature = "audio")]
use kira::{
    AudioManager, AudioManagerSettings, DefaultBackend, sound::static_sound::StaticSoundData,
};

/// A one-shot sound triggered by a state change. `App` queues one; `main` drains
/// it each tick and plays it. **Not** feature-gated — the pure app logic decides
/// *which* sound to play without ever depending on kira, so it stays testable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioCue {
    /// SUED denies the uninitiated — the jump-scare sting (`assets/jump_scare.ogg`).
    JumpScare,
    /// SUED reveals the answer — demonic laughter (`assets/laugh.ogg`).
    Laugh,
}

pub fn laugh_interval(roll: f32) -> Duration {
    let span = LAUGH_MAX_SECS - LAUGH_MIN_SECS;
    Duration::from_secs(LAUGH_MIN_SECS + (roll * span as f32) as u64)
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
    }
}

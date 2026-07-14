//! Audio via `kira` — a looping dread ambience + one-shot scare stings.
//!
//! Feature-gated: with `--features audio` this drives real sound; without it,
//! `Audio` is a **no-op stub with the same API**, so `main` never needs a
//! `#[cfg]`. `--no-sound` (passed at runtime) also forces the silent path even
//! in an audio build. kira owns its own realtime audio thread, so nothing here
//! spawns threads — we just hand it sounds to play.

#[cfg(feature = "audio")]
use kira::{AudioManager, sound::static_sound::StaticSoundData};

/// A one-shot sound triggered by a state change. `App` queues one; `main` drains
/// it each tick and plays it. **Not** feature-gated — the pure app logic decides
/// *which* sound to play without ever depending on kira, so it stays testable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioCue {
    /// SUED reveals the answer — the jump-scare sting (`assets/jump_scare.ogg`).
    JumpScare,
    /// SUED denies the uninitiated — demonic laughter (`assets/laugh.ogg`).
    Laugh,
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

// ── Real build: `--features audio` ──────────────────────────────────────────
// TODO(Danilo, M3): wire kira in here (the bodies are `todo!()` for you). You
// read the 0.12 docs — the shape we agreed:
//   * fields: a `kira::AudioManager` + the three loaded `StaticSoundData`
//     (ambience looped, sting, laugh) — load from `assets/*.ogg`.
//     `include_bytes!` them or read at startup; your call.
//   * new(enabled): if `!enabled`, still return a valid *silent* `Audio`
//     (skip building the manager) so `--no-sound` works in an audio build;
//     otherwise create the manager and load the sounds.
//   * start_ambience: play the ambience `StaticSoundData` with a loop region.
//   * play(cue): `match cue { Sting => play sting, Mock => play laugh }` — a
//     one-shot each (no loop). Keep the manager handle alive on `self`.
#[cfg(feature = "audio")]
pub struct Audio {
    manager: AudioManager,
    ambience_sound: StaticSoundData,
    laugh_sound: StaticSoundData,
    jump_scare_sound: StaticSoundData,
}

#[cfg(feature = "audio")]
impl Audio {
    pub fn new(_enabled: bool) -> anyhow::Result<Self> {
        use kira::{
            AudioManager, AudioManagerSettings, DefaultBackend,
            sound::static_sound::StaticSoundData,
        };

        let audio_manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;

        let ambience_sound = StaticSoundData::from_file("assets/ambience.ogg")?;
        let laugh_sound = StaticSoundData::from_file("assets/laugh.ogg")?;
        let jump_scare_sound = StaticSoundData::from_file("assets/jump_scare.ogg")?;

        Ok(Audio {
            manager: audio_manager,
            ambience_sound,
            laugh_sound,
            jump_scare_sound,
        })
    }

    pub fn start_ambience(&mut self) {
        let sound = self.ambience_sound.clone().loop_region(..);
        let _ = self.manager.play(sound);
    }

    pub fn play(&mut self, audio_cue: AudioCue) {
        match audio_cue {
            AudioCue::Laugh => {
                let _ = self.manager.play(self.laugh_sound.clone());
            }
            AudioCue::JumpScare => {
                let _ = self.manager.play(self.jump_scare_sound.clone());
            }
        }
    }
}

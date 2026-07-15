//! Audio via `kira` — a looping dread ambience + one-shot scare stings.
//!
//! Feature-gated: with `--features audio` this drives real sound; without it,
//! `Audio` is a **no-op stub with the same API**, so `main` never needs a
//! `#[cfg]`. `--no-sound` (passed at runtime) also forces the silent path even
//! in an audio build. kira owns its own realtime audio thread, so nothing here
//! spawns threads — we just hand it sounds to play.

#[cfg(feature = "audio")]
use std::io::Cursor;

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

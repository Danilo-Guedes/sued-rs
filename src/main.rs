//! sued-rs — a horror-themed terminal recreation of the SueD prank oracle.
//!
//! Terminal lifecycle (RAII guard) + the tick loop live here; the ratatui draw
//! code lives in [`ui::screens`]. The pure prank logic is in [`core::engine`]
//! and stays untouched.
#![allow(dead_code)]

mod app;
mod audio;
mod cli;
mod config;
mod constants;
mod core;
mod language;
mod ui;

use std::io::{Stdout, stdout};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::app::{App, AppFlow};
use crate::audio::{Audio, AudioCue, laugh_interval};
use crate::cli::Args;
use crate::config::Configuration;
use crate::core::engine::KeyPress;

/// How long each tick waits for input before redrawing. ~50ms ≈ 20 fps — smooth
/// enough for the animations, cheap enough to idle on.
const TICK: Duration = Duration::from_millis(50);

/// Owns the terminal's "loud" state: raw mode + the alternate screen. Acquired
/// on `new`, released on `Drop` — so the terminal is always restored, even on a
/// panic or an early `?` return.
struct TerminalGuard;

impl TerminalGuard {
    fn new() -> std::io::Result<Self> {
        enable_raw_mode()?; // keystrokes reach us raw — no line buffering / echo
        execute!(stdout(), EnterAlternateScreen)?; // switch to a fresh screen we own
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Best-effort teardown, reverse order, and it must NOT panic.
        let _ = execute!(stdout(), LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let config_path = match args.config {
        Some(path) => path,
        None => default_config_path()?,
    };

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all::<&Path>(parent)
            .with_context(|| format!("while creating {}", parent.display()))?;
    }

    let parsed_config = Configuration::load(config_path.as_path())
        .with_context(|| format!("while trying to read {}", config_path.display()))?;

    if !config_path.exists() {
        parsed_config
            .save(&config_path)
            .with_context(|| format!("saving in {}", config_path.display()))?;
    }

    let _guard = TerminalGuard::new()?; // declared first → dropped LAST (cleans up after the terminal)
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app_state = App::new(parsed_config);

    // Sound is on only when compiled with `--features audio` AND not `--no-sound`.
    // The no-op `Audio` ignores the flag; the real one goes silent when it's false.
    let mut audio = Audio::new(!args.no_sound)?;

    run(&mut terminal, &mut app_state, &mut audio, &config_path)
}

/// The tick loop: redraw every frame, only `read()` when there's actually input.
/// Blocking on `read()` would freeze any animation between keystrokes.
fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app_state: &mut App,
    audio: &mut Audio,
    config_file_path: &Path,
) -> Result<()> {
    audio.start_ambience(); // the dread bed loops for the whole session

    let mut next_laugh_at = Instant::now() + laugh_interval(rand::random());

    loop {
        // 1. DRAW — ratatui diffs against the last frame and writes only what changed.
        terminal.draw(|frame| ui::screens::render(frame, app_state))?;

        // 2. QUEUE THE LAUGH AUDIO EFFECT
        let now = Instant::now();
        if now >= next_laugh_at {
            audio.play(AudioCue::Laugh);
            next_laugh_at = now + laugh_interval(rand::random());
        }

        // 3. POLL — wait up to `TICK` for an event. Returns false on timeout (no input).
        if event::poll(TICK)? {
            // 3. READ — only now, knowing an event is waiting, so this won't block.
            if let Event::Key(key) = event::read()? {
                // Windows fires Press AND Release; only act on Press, or every key doubles.
                if key.kind == KeyEventKind::Press {
                    let flow = translate_key(app_state, key);

                    if let Some(config) = app_state.take_pending_save() {
                        config
                            .save(config_file_path)
                            .with_context(|| format!("saving {}", config_file_path.display()))?;
                    }

                    if let Some(cue) = app_state.take_cue() {
                        audio.play(cue);
                    }
                    if flow == AppFlow::Quit {
                        return Ok(());
                    }
                }
            }
        }
    }
}

/// Translate a crossterm key into an engine `Key` and drive the engine.
/// Returns `AppFlow::Quit` on the exit keys so the loop can break cleanly — note we
/// never `process::exit`; we return, so `TerminalGuard`'s `Drop` always runs.
fn translate_key(app_state: &mut App, key: KeyEvent) -> AppFlow {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => AppFlow::Quit,
        KeyCode::Backspace => app_state.handle_key(KeyPress::Backspace),
        KeyCode::Esc => app_state.handle_key(KeyPress::Esc),
        KeyCode::Enter => app_state.handle_key(KeyPress::Enter),
        KeyCode::Down => app_state.handle_key(KeyPress::Down),
        KeyCode::Up => app_state.handle_key(KeyPress::Up),
        KeyCode::Char(ch) => app_state.handle_key(KeyPress::Char(ch)),
        KeyCode::F(5) => app_state.handle_key(KeyPress::F5),
        KeyCode::Left => app_state.handle_key(KeyPress::Left),
        KeyCode::Right => app_state.handle_key(KeyPress::Right),
        _ => AppFlow::Stay,
    }
}

fn default_config_path() -> Result<PathBuf> {
    let mut path = dirs::config_dir().context("could not determine the config directory")?;
    path.push("sued-rs");
    path.push("sued.config.json");
    Ok(path)
}

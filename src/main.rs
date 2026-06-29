//! sued-rs — a horror-themed terminal recreation of the SueD prank oracle.
//!
//! **M2 scaffold.** Terminal lifecycle (RAII guard) + the tick loop live here;
//! the ratatui draw code lives in [`ui::app`]. The pure prank logic is in
//! [`core::engine`] and stays untouched. See `../plan/PLAN.md` §D (M2).
#![allow(dead_code)]

mod cli;
mod config;
mod core;
mod ui;

#[cfg(feature = "audio")]
mod audio;

use std::io::{Stdout, stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::core::engine::{DECOY_STRING, Engine, Key};

/// How long each tick waits for input before redrawing. ~50ms ≈ 20 fps — smooth
/// enough for the animations coming in M3/M4, cheap enough to idle on.
const TICK: Duration = Duration::from_millis(50);

/// Owns the terminal's "loud" state: raw mode + the alternate screen. Acquired
/// on `new`, released on `Drop` — so the terminal is always restored, even on a
/// panic or an early `?` return. (Same RAII idea as M1's guard, now owning two
/// resources instead of one.)
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

/// What the loop should do after handling a key.
#[derive(PartialEq)]
enum LoopFlow {
    Continue,
    Quit,
}

fn main() -> Result<()> {
    let _guard = TerminalGuard::new()?; // declared first → dropped LAST (cleans up after the terminal)
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut engine = Engine::new(DECOY_STRING);

    run(&mut terminal, &mut engine)
}

/// The tick loop: redraw every frame, only `read()` when there's actually input.
/// Blocking on `read()` (M1) would freeze any animation between keystrokes.
fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, engine: &mut Engine) -> Result<()> {
    loop {
        // 1. DRAW — ratatui diffs against the last frame and writes only what changed.
        terminal.draw(|frame| ui::app::render(frame, engine))?;

        // 2. POLL — wait up to `TICK` for an event. Returns false on timeout (no input).
        if event::poll(TICK)? {
            // 3. READ — only now, knowing an event is waiting, so this won't block.
            if let Event::Key(key) = event::read()? {
                // Windows fires Press AND Release; only act on Press, or every key doubles.
                if key.kind == KeyEventKind::Press && handle_key(engine, key) == LoopFlow::Quit {
                    return Ok(());
                }
            }
        }
    }
}

/// Translate a crossterm key into an engine `Key` and drive the engine.
/// Returns `LoopFlow::Quit` on the exit keys so the loop can break cleanly — note we
/// never `process::exit`; we return, so `TerminalGuard`'s `Drop` always runs.
fn handle_key(engine: &mut Engine, key: KeyEvent) -> LoopFlow {
    match key.code {
        KeyCode::Esc => return LoopFlow::Quit,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return LoopFlow::Quit,
        KeyCode::Char(ch) => {
            engine.handle_key(Key::Char(ch));
        }
        KeyCode::Enter => {
            engine.handle_key(Key::Enter);
        }
        KeyCode::Backspace => {
            engine.handle_key(Key::Backspace);
        }
        // Ignore anything else. (No `println!` here — it would corrupt the TUI.)
        _ => {}
    }
    LoopFlow::Continue
}

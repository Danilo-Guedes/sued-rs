//! sued-rs — a horror-themed terminal recreation of the SueD prank oracle.
//!
//! This is the M0 scaffold. The app proper begins at **M1**: implement the pure
//! prank `Engine` in [`core::engine`]. See `../plan/PLAN.md` for the milestone plan
//! and the working agreement.
//!
#![allow(dead_code)]

mod cli;
mod config;
mod core;
mod ui;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, read};
use crossterm::terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode};
use crossterm::{cursor::MoveTo, execute, style::Print};
use std::io::{Write, stdout};

#[cfg(feature = "audio")]
mod audio;

use anyhow::Result;

use crate::core::engine::{DECOY_STRING, Engine, Key, Mode, StateChange};

struct RawModeGuard; // a "zero-size" marker that owns the raw-mode state

impl RawModeGuard {
    fn new() -> std::io::Result<Self> {
        /* enable raw mode, return Ok(Self) */
        match enable_raw_mode() {
            Ok(_) => Ok(Self),
            Err(err) => Err(err),
        }
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        /* disable raw mode — best-effort, must NOT panic */

        match disable_raw_mode() {
            Ok(_) => {}
            Err(err) => {
                eprintln!("{}", err)
            }
        };
    }
}

fn main() -> Result<()> {
    let _raw_guard = RawModeGuard::new()?;

    let mut engine = Engine::new(DECOY_STRING);

    render(&engine)?;

    loop {
        match crossterm::event::read()? {
            // BLOCKS until an event
            Event::Key(key) => {
                /* translate → drive engine → maybe break */

                match key {
                    KeyEvent {
                        code,
                        modifiers,
                        kind: _,
                        state: _,
                    } => match code {
                        KeyCode::Backspace => {
                            engine.handle_key(Key::Backspace);
                        }
                        KeyCode::Enter => {
                            engine.handle_key(Key::Enter);
                        }
                        KeyCode::Esc => {
                            // get out of the loop
                            return Ok(());
                        }
                        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(());
                        }
                        KeyCode::Char(ch) => {
                            engine.handle_key(Key::Char(ch));
                        }
                        other_key => {
                            println!("key still not handled  got={}", other_key)
                        }
                    },
                }
            }
            _ => {} // ignore resize/mouse/etc.
        }

        render(&engine)?;
    }
}

fn render(engine: &Engine) -> Result<()> {
    let mut out = stdout();

    // Wipe the screen and move the cursor home — we redraw the whole frame each key.
    execute!(out, Clear(ClearType::All), MoveTo(0, 0))?;

    let mode_tag = match engine.get_mode() {
        Mode::Normal => "NORMAL",
        Mode::Hidden => "HIDDEN",
    };
    execute!(out, Print(format!("☠  SueD — o oráculo  [{mode_tag}]\r\n")))?;
    execute!(out, Print("──────────────────────────────\r\n"))?;

    // What the audience sees being "typed":
    execute!(out, Print(format!("{}\r\n", engine.get_visible_buffer())))?;

    // After Enter, the oracle "responds":
    if let Some(answer) = engine.get_revealed() {
        execute!(
            out,
            Print(format!("\r\n>>> O ORÁCULO RESPONDE:\r\n    {answer}\r\n"))
        )?;
    }

    execute!(
        out,
        Print("\r\n(';' alterna modo · Enter revela · Esc sai)\r\n")
    )?;
    out.flush()?;
    Ok(())
}

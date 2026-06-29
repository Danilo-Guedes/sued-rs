//! ratatui draw code — reads `Engine` state each frame and renders it.
//!
//! This is the **M2 scaffold**: a single bordered panel showing the live
//! `visible_buffer`, just enough to prove the `Engine → ratatui` pipe works.
//! Growing this is yours (improvement: immediate-mode rendering, `Layout`/`Rect`,
//! widgets, the menu + marquee). See `../../plan/PLAN.md` §D (M2).

use ratatui::Frame;
use ratatui::widgets::{Block, Paragraph};

use crate::core::engine::Engine;

/// Draw one frame. Called every tick by the run loop in `main.rs`.
///
/// `render` only ever *reads* the engine — it never mutates the prank logic.
/// That read-only split is the habit to keep as this grows.
pub fn render(frame: &mut Frame, engine: &Engine) {
    // TODO(M2 — yours):
    //   * split `frame.area()` with `Layout` into title / oracle panel / input area
    //   * show `engine.get_revealed()` as the oracle's answer after Enter
    //   * build the menu screen (`Resposta / Informações / Sair`) + red marquee
    //   * style it spooky (red on black, borders, etc.)
    // For now: one bordered box with the live "typed" text inside.
    let panel = Block::bordered().title(" ☠  SueD — o oráculo  ☠ ");
    let body = Paragraph::new(engine.get_visible_buffer()).block(panel);

    frame.render_widget(body, frame.area());
}

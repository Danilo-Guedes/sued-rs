//! ratatui draw code — reads `Engine` state each frame and renders it.
//!
//! This is the **M2 scaffold**: a single bordered panel showing the live
//! `visible_buffer`, just enough to prove the `Engine → ratatui` pipe works.
//! Growing this is yours (improvement: immediate-mode rendering, `Layout`/`Rect`,
//! widgets, the menu + marquee). See `../../plan/PLAN.md` §D (M2).

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Stylize;
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

    let [title_bar, sued_art, sued_says, sued_logs, input, status] = Layout::vertical([
        Constraint::Length(2), // title bar,
        Constraint::Fill(2),   // sued_art
        Constraint::Fill(2),   // sued_says
        Constraint::Fill(3),   // sued_logs
        Constraint::Length(4), // input box
        Constraint::Length(3), // status bar
    ])
    .areas(frame.area());

let [title_bar_left, title_bar_right] =
    Layout::horizontal([Constraint::Fill(1), Constraint::Length(22)]).areas(title_bar);

    frame.render_widget(
        Paragraph::new(" ☠  SueD — o oráculo  ☠ ").red().bold(),
        // .style(Style::new().red().rapid_blink()),
        title_bar_left,
    );

    frame.render_widget(
        Paragraph::new("sessão #666  * online")
            .blue()
            .right_aligned(),
        title_bar_right,
    );
    frame.render_widget(Block::bordered().title("sued_art"), sued_art);

    let speak = centered(sued_says, Constraint::Length(50), Constraint::Length(6));

    frame.render_widget(Block::bordered().title("sued_says"), sued_says);

    frame.render_widget(Block::bordered().title("sued_says"), speak);

    frame.render_widget(
        Paragraph::new("Pergunte-me o que deseja saber, humano ...").block(Block::new()),
        speak,
    );

    frame.render_widget(Block::bordered().title("sued_logs"), sued_logs);

    let typed = Paragraph::new(engine.get_visible_buffer())
        .block(Block::bordered().title("input").on_light_red());

    frame.render_widget(typed, input);

    frame.render_widget(Block::bordered().title("status_bar").on_red(), status);
}

fn centered(area: Rect, width: Constraint, height: Constraint) -> Rect {
    let [a] = Layout::horizontal([width]).flex(Flex::Center).areas(area);
    let [a] = Layout::vertical([height]).flex(Flex::Center).areas(a);
    a
}

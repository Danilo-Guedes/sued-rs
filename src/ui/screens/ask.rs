//! 03 · MODO PERGUNTA.

use std::time::{Duration, Instant};

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};

use super::common::{colorfull_bordered_block, create_centered_rect, render_nav_strip};
use crate::core::engine::Engine;
use crate::ui::effects::{CURSOR_CHAR, cursor_on, flash_intensity, typewriter_reveal};
use crate::ui::screens::common::{
    DEMON_ART, DEMON_ART_HEIGHT, DEMON_ART_WIDTH, NavTab, create_screen_block,
};

pub(super) fn render(
    frame: &mut Frame,
    engine: &Engine,
    replied_at: Option<Instant>,
    denied_message: Option<&'static str>,
    started_at: &Instant,
) {
    let layout = create_screen_block(frame);

    let [
        nav_layout,
        sued_art_top_layout,
        sued_says_layout,
        sued_logs_layout,
        input_layout,
        status_layout,
    ] = Layout::vertical([
        Constraint::Length(4), // nav strip
        Constraint::Fill(3),   // sued_art
        Constraint::Fill(2),   // sued_says
        Constraint::Fill(3),   // sued_logs
        Constraint::Length(3), // input box
        Constraint::Length(2), // status bar
    ])
    .areas(layout);

    render_nav_strip(frame, nav_layout, NavTab::Ask);

    let [_, center_art_rect, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
    ])
    .areas(sued_art_top_layout);

    // demon ASCII art will fill this area next (no border, per design)
    frame.render_widget(
        Paragraph::new(DEMON_ART).red(),
        create_centered_rect(
            center_art_rect,
            Constraint::Length(DEMON_ART_WIDTH),
            Constraint::Length(DEMON_ART_HEIGHT),
        ),
    );

    let speak_layout = create_centered_rect(
        sued_says_layout,
        Constraint::Length(60),
        Constraint::Fill(1),
    );

    let elapsed_duration = match replied_at {
        Some(instant) => instant.elapsed(),
        None => Duration::ZERO,
    };

    let final_sued_words = match engine.revealed() {
        Some(answer) => Text::from(typewriter_reveal(answer, elapsed_duration)),
        None => {
            match denied_message {
                Some(denied_str) => Text::from(typewriter_reveal(denied_str, elapsed_duration)),
                None => {
                    Text::from(vec![
                        Line::from("Pergunte-me o que deseja saber, humano..."),
                        Line::from(""), // blank row for breathing space
                        Line::from(vec![
                            Span::raw("— elogie-me antes da pergunta, e ").dim(),
                            Span::raw("talvez").red(),
                            Span::raw(" eu responda.").dim(),
                        ]),
                    ])
                }
            }
        }
    };

    let flash_effect = replied_at.map_or(0, |t| flash_intensity(t.elapsed()));

    let flash_bg = if flash_effect > 0 {
        Color::Rgb(flash_effect, 0, 0)
    } else {
        Color::Reset
    };

    let speak_widget = Paragraph::new(final_sued_words)
        .block(
            colorfull_bordered_block(None)
                .bg(flash_bg)
                .title(" SUED FALA ")
                .padding(Padding::new(2, 2, 1, 1)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(speak_widget, speak_layout);

    let default_logs_text = Text::from(vec![
        Line::from(vec![
            Span::raw(">").red(),
            Span::raw(" "),
            Span::raw("conexão com o além estabelecida.").dim(),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(">").red(),
            Span::raw(" "),
            Span::raw("aguardando oferenda do mortal_").dim(),
        ]),
    ]);

    frame.render_widget(
        Paragraph::new(default_logs_text).block(Block::new().padding(Padding::new(4, 2, 0, 0))),
        sued_logs_layout,
    );

    let rendered_cursor = if replied_at.is_none() && cursor_on(started_at.elapsed()) {
        Span::raw(CURSOR_CHAR.to_string()).red()
    } else {
        Span::raw("")
    };

    let typed = Text::from(vec![Line::from(vec![
        " ▶ ".red().bold(),
        Span::raw(engine.visible_buffer()).white(),
        rendered_cursor,
    ])]);

    frame.render_widget(
        Paragraph::new(typed)
            .block(colorfull_bordered_block(None).title(" input "))
            .wrap(Wrap { trim: false }),
        input_layout,
    );

    let status_block = colorfull_bordered_block(Some(Borders::TOP)).padding(Padding::horizontal(2));
    let status_inner = status_block.inner(status_layout);
    frame.render_widget(status_block, status_layout);

    let [hints_area, page_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(14)]).areas(status_inner);

    let hints = Line::from(vec![
        "[Enter]".red().bold(),
        " ".into(),
        "perguntar".dim(),
        "  ".into(),
        "[F5]".red().bold(),
        " ".into(),
        "new question".dim(),
        "  ".into(),
        "[Esc]".red().bold(),
        " ".into(),
        "menu".dim(),
        "  ".into(),
        "[Ctrl-C]".red().bold(),
        " ".into(),
        "sair".dim(),
    ]);
    frame.render_widget(Paragraph::new(hints), hints_area);
    frame.render_widget(Paragraph::new("PERGUNTA").dim().right_aligned(), page_area);
}

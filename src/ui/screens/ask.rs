//! 03 · MODO PERGUNTA.

use std::time::{Duration, Instant};

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Offset};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};

use super::common::{colorfull_bordered_block, create_centered_rect, hint_line, render_nav_strip};
use crate::config::Configuration;
use crate::core::engine::Engine;
use crate::ui::effects::{
    CURSOR_CHAR, cursor_on, flash_intensity, flicker_intensity, reveal_is_complete, shake_offset,
    typewriter_reveal,
};
use crate::ui::screens::common::{
    DEMON_ART, DEMON_ART_HEIGHT, DEMON_ART_WIDTH, NavTab, create_screen_block,
};
use crate::ui::template::styled_line;

pub(super) fn render(
    frame: &mut Frame,
    engine: &Engine,
    replied_at: Option<Instant>,
    denied_message: Option<&'static str>,
    started_at: &Instant,
    config: Configuration,
    previous_reply: Option<&str>,
) {
    let time_elapsed_from_the_start_at = started_at.elapsed();

    let palette = config.theme().palette();

    let language = config.language();

    let translation = language.translation();

    let layout = create_screen_block(frame, palette);

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

    render_nav_strip(
        frame,
        nav_layout,
        NavTab::Ask,
        palette,
        language,
        translation,
    );

    let [_, center_art_rect, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
    ])
    .areas(sued_art_top_layout);

    let random_flicker_value = flicker_intensity(rand::random(), config.animations());

    let demon_rect = create_centered_rect(
        center_art_rect,
        Constraint::Length(DEMON_ART_WIDTH),
        Constraint::Length(DEMON_ART_HEIGHT),
    );

    let screen = frame.area();

    let (x_offset, y_offset) = replied_at.map_or((0, 0), |t| {
        shake_offset(
            t.elapsed(),
            rand::random(),
            rand::random(),
            config.animations(),
        )
    });

    let demon_rect = demon_rect
        .offset(Offset {
            x: x_offset as i32,
            y: y_offset as i32,
        })
        .intersection(screen);

    // demon ASCII art will fill this area next (no border, per design)
    frame.render_widget(
        Paragraph::new(DEMON_ART).fg(palette.glow(random_flicker_value)),
        demon_rect,
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
                    match previous_reply {
                        Some(last_reply) => Text::from(last_reply),
                        None => {
                            Text::from(vec![
                                Line::from(translation.ask.welcome_line),
                                Line::from(""), // blank row for breathing space
                                styled_line(
                                    translation.ask.praise,
                                    Style::default().dim(),
                                    palette.accent,
                                ),
                            ])
                        }
                    }
                }
            }
        }
    };

    let flash_effect = replied_at.map_or(0, |t| flash_intensity(t.elapsed(), config.animations()));

    let flash_bg = if flash_effect > 0 {
        palette.glow(flash_effect)
    } else {
        palette.bg
    };

    let speak_widget = Paragraph::new(final_sued_words)
        .block(
            colorfull_bordered_block(None, palette)
                .bg(flash_bg)
                .title(translation.ask.sued_speak)
                .padding(Padding::new(2, 2, 1, 1)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(speak_widget, speak_layout);

    let underline_cursor = if cursor_on(time_elapsed_from_the_start_at) {
        Span::raw("_").dim()
    } else {
        Span::raw("")
    };

    let default_logs_text = Text::from(vec![
        Line::from(vec![
            Span::raw(">").fg(palette.accent),
            Span::raw(" "),
            Span::raw(translation.ask.connection).dim(),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(">").fg(palette.accent),
            Span::raw(" "),
            Span::raw(translation.ask.waiting).dim(),
            underline_cursor,
        ]),
    ]);

    frame.render_widget(
        Paragraph::new(default_logs_text).block(Block::new().padding(Padding::new(4, 2, 0, 0))),
        sued_logs_layout,
    );

    // The render-side twin of the input lock in `App::handle_key`: keys are only
    // swallowed while SueD is mid-reveal. "Never spoke" is not a locked state —
    // it is a fresh screen, so the cursor must already be blinking.
    let input_is_unlocked = match replied_at {
        Some(inst) => {
            let current_sued_words = match denied_message {
                Some(denied_msg) => denied_msg,
                None => engine
                    .revealed()
                    .expect("a reply clock with no reply words is a bug"),
            };
            reveal_is_complete(current_sued_words, inst.elapsed())
        }
        None => true,
    };

    let rendered_cursor = if input_is_unlocked && cursor_on(time_elapsed_from_the_start_at) {
        Span::raw(CURSOR_CHAR.to_string()).fg(palette.accent)
    } else {
        Span::raw("")
    };

    let typed = Text::from(vec![Line::from(vec![
        " ▶ ".fg(palette.accent).bold(),
        Span::raw(engine.visible_buffer()).white(),
        rendered_cursor,
    ])]);

    frame.render_widget(
        Paragraph::new(typed)
            .block(colorfull_bordered_block(None, palette).title(translation.ask.talk_with_me))
            .wrap(Wrap { trim: false }),
        input_layout,
    );

    let status_block =
        colorfull_bordered_block(Some(Borders::TOP), palette).padding(Padding::horizontal(2));
    let status_inner = status_block.inner(status_layout);
    frame.render_widget(status_block, status_layout);

    let [hints_area, page_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(14)]).areas(status_inner);

    let hints = hint_line(translation.ask.hints, palette);
    frame.render_widget(Paragraph::new(hints), hints_area);
    frame.render_widget(
        Paragraph::new(NavTab::Ask.label(language).to_uppercase())
            .dim()
            .right_aligned(),
        page_area,
    );
}

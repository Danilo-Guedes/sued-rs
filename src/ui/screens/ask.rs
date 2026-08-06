//! 03 · MODO PERGUNTA.

use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Offset};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};

use super::common::{colorfull_bordered_block, create_centered_rect, hint_line, render_nav_strip};
use super::{confirm, history};
use crate::app::{App, AskingState};
use crate::conversation::Overlay;
use crate::ui::effects::{
    CURSOR_CHAR, cursor_on, flash_intensity, flicker_intensity, pulse_intensity,
    reveal_is_complete, shake_offset, thinking_dots, typewriter_reveal,
};
use crate::ui::screens::common::{
    DEMON_ART, DEMON_ART_HEIGHT, DEMON_ART_WIDTH, NavTab, create_screen_block,
};
use crate::ui::template::styled_line;

pub(super) fn render(frame: &mut Frame, app: &App, asking_state: &AskingState) {
    let time_elapsed_from_the_start_at = app.started_at().elapsed();

    let config = app.config_object;

    let palette = config.theme().palette();

    let language = config.language();

    let translation = language.translation();

    let reply = asking_state.reply.as_ref();

    let casting_for = reply.filter(|r| r.is_pondering()).map(|r| r.since_asked());

    let speaking_for = reply
        .filter(|r| !r.is_pondering())
        .map(|r| r.speaking_elapsed());

    let spell = asking_state.spell;

    let engine = &asking_state.engine;

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

    let (x_offset, y_offset) = speaking_for.map_or((0, 0), |dur| {
        shake_offset(dur, rand::random(), rand::random(), config.animations())
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

    let elapsed_duration = speaking_for.unwrap_or(Duration::ZERO);

    let final_sued_words = match casting_for {
        // Accent, where every reply is plain white — so the incantation reads as
        // SueD *acting* and the answer as SueD *speaking*. The colour switch is
        // what makes the reply land; a white spell would read as the answer
        // itself, arriving without ceremony.
        //
        // ⚠ AMENDED 2026-08-04 (G18) — THE SPELL NO LONGER TYPES, IT BREATHES.
        // The crawl was deleted from this arm on purpose: `typewriter_reveal` now
        // has only answer call sites, so letters arriving one at a time means
        // exactly one thing — SueD is ANSWERING. The spell became atmosphere
        // rather than a second, slower answer.
        //
        // ⚠ AND THE PULSE DOES NOT UNDO THE COLOUR DECISION ABOVE, WHICH RESTS
        // ON AN INVARIANT WORTH KNOWING: `glow(255)` returns `peak` unscaled, and
        // every palette's `peak` tuple IS its `accent` (`theme.rs:26-44`). So the
        // spell breathes along a single hue — its own — dimming toward
        // `PULSE_INTENSITY_MIN` and back. Give some future palette a `peak` that
        // differs from its `accent` and this line stops animating the ponder and
        // starts silently restyling it.
        //
        // Safe against the reply flash: `flash_bg` is driven by `speaking_for`,
        // which is `None` for the whole ponder, so accent-on-accent can't happen.
        Some(spell_elapsed) => Text::from(format!(
            "{spell}{}",
            ".".repeat(thinking_dots(spell_elapsed))
        ))
        .fg(palette.glow(pulse_intensity(spell_elapsed, config.animations()))),
        None => match engine.revealed() {
            Some(answer) => Text::from(typewriter_reveal(answer, elapsed_duration)),
            None => match reply {
                Some(reply) => Text::from(typewriter_reveal(reply.words(), elapsed_duration)),
                None => match asking_state.previous_reply() {
                    Some(words) => Text::from(words),
                    None => Text::from(vec![
                        Line::from(translation.ask.welcome_line),
                        Line::from(""), // blank row for breathing space
                        styled_line(
                            translation.ask.praise,
                            Style::default().dim(),
                            palette.accent,
                        ),
                    ]),
                },
            },
        },
    };

    let flash_effect = speaking_for.map_or(0, |e| flash_intensity(e, config.animations()));

    let flash_bg = if flash_effect > 0 {
        palette.glow(flash_effect)
    } else {
        palette.bg
    };

    let speak_widget = Paragraph::new(final_sued_words)
        .block(
            colorfull_bordered_block(None, palette)
                .bg(flash_bg)
                .title(format!(" {} ", translation.ask.sued_speak))
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

    let sued_is_speaking = match reply {
        None => false,
        Some(r) if r.is_pondering() => true,
        Some(r) => !reveal_is_complete(r.words(), r.speaking_elapsed()),
    };

    let input_is_unlocked = asking_state.overlay().is_none() && !sued_is_speaking;

    let rendered_cursor = if input_is_unlocked && cursor_on(time_elapsed_from_the_start_at) {
        Span::raw(CURSOR_CHAR.to_string()).fg(palette.accent)
    } else {
        Span::raw("")
    };

    let typed = Text::from(vec![Line::from(vec![
        " ▶ ".fg(palette.accent).bold(),
        Span::raw(if sued_is_speaking {
            asking_state.last_question().unwrap_or_default()
        } else {
            engine.visible_buffer()
        })
        .white(),
        rendered_cursor,
    ])]);

    frame.render_widget(
        Paragraph::new(typed)
            .block(
                colorfull_bordered_block(None, palette)
                    .title(format!(" {} ", translation.ask.talk_with_me)),
            )
            .wrap(Wrap { trim: false }),
        input_layout,
    );

    let status_block =
        colorfull_bordered_block(Some(Borders::TOP), palette).padding(Padding::horizontal(2));
    let status_inner = status_block.inner(status_layout);
    frame.render_widget(status_block, status_layout);

    let [hints_area, page_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(14)]).areas(status_inner);

    let current_hint_slice = match asking_state.overlay() {
        Some(Overlay::Transcript(_)) => translation.history.hints,
        Some(Overlay::ConfirmLeave(_)) => translation.confirm.hints,
        None => translation.ask.hints,
    };

    let hints = hint_line(current_hint_slice, palette);
    frame.render_widget(Paragraph::new(hints), hints_area);

    let current_bottom_title = match asking_state.overlay() {
        Some(Overlay::Transcript(_)) => NavTab::History.label(language),
        Some(Overlay::ConfirmLeave(_)) => NavTab::Confirm.label(language),
        None => NavTab::Ask.label(language),
    };

    frame.render_widget(
        Paragraph::new(current_bottom_title.to_uppercase())
            .dim()
            .right_aligned(),
        page_area,
    );

    // The popover draws LAST, so it lands on top of the screen it covers.
    //
    // It gets the middle band only — art through logs — because the input box
    // and the status strip must stay visible: the strip is now the popover's own
    // hint line (it swapped above), so covering it would hide the only thing
    // telling the operator how to get out.
    //
    // ⚠ `union` is the bounding box of the two, so this spans art→says→logs only
    // while those three stay ADJACENT in the vertical stack above. Reorder that
    // layout and the band silently changes meaning — it still compiles, still
    // draws, and covers the wrong thing.
    if let Some(history_view) = asking_state.transcript() {
        history::render(
            frame,
            asking_state,
            history_view,
            sued_art_top_layout.union(sued_logs_layout),
            palette,
            translation,
        );
    }

    // The confirm dialog shares the transcript's band, but note it does NOT
    // share its footprint: the mockup keeps the demon visible above the box,
    // because the thing you are being asked to abandon should still be looking
    // at you while you decide.
    if let Some(Overlay::ConfirmLeave(choice)) = asking_state.overlay() {
        confirm::render(
            frame,
            sued_art_top_layout.union(sued_logs_layout),
            choice,
            palette,
            translation,
        );
    }
}

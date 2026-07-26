//! 01 · INTRO / Invocação.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Borders, Padding, Paragraph, Wrap};

use crate::config::Configuration;
use crate::ui::effects::flicker_intensity;
use crate::ui::screens::common::{
    NavTab, SUED_BANNER, SUED_BANNER_HEIGHT, SUED_BANNER_WIDTH, colorfull_bordered_block,
    create_centered_rect, create_screen_block, hint_line, render_nav_strip,
};
use crate::ui::template::styled_line;

pub(super) fn render(frame: &mut Frame, config: Configuration) {
    let palette = config.theme().palette();

    let language = config.language();

    let translation = language.translation();

    let layout = create_screen_block(frame, palette);

    let [
        nav_layout,
        _,
        page_title_and_sub_layout,
        intro_text_layout,
        _,
        status_layout,
    ] = Layout::vertical([
        Constraint::Length(4),  // nav strip
        Constraint::Fill(1),    // empty
        Constraint::Fill(3),    // page_title_and_sub
        Constraint::Length(18), // intro_text_layout
        Constraint::Fill(1),    // empty
        Constraint::Length(2),  // status bar
    ])
    .areas(layout);

    render_nav_strip(
        frame,
        nav_layout,
        NavTab::Intro,
        palette,
        language,
        translation,
    );

    let [banner_area, _gap, subtitle_area] = Layout::vertical([
        Constraint::Length(SUED_BANNER_HEIGHT),
        Constraint::Length(1), // breathing space
        Constraint::Length(1), // subtitle line
    ])
    .flex(Flex::Center)
    .areas(page_title_and_sub_layout);

    let banner_rect = create_centered_rect(
        banner_area,
        Constraint::Length(SUED_BANNER_WIDTH),
        Constraint::Length(SUED_BANNER_HEIGHT),
    );

    let random_flicker_value = flicker_intensity(rand::random(), config.animations());

    frame.render_widget(
        Paragraph::new(SUED_BANNER)
            .fg(palette.glow(random_flicker_value))
            .bold(),
        banner_rect,
    );

    frame.render_widget(
        Paragraph::new(translation.intro.subtitle.dim()).centered(),
        subtitle_area,
    );

    // Red rule + breathing space above the ATENÇÃO block (per the design). Split a
    // small strip off the top for the rule; the warning text fills the rest.
    let [divider_area, atencao_area] = Layout::vertical([
        Constraint::Length(3), // red rule (row 0) + a two-row gap below it
        Constraint::Fill(1),   // the warning text block
    ])
    .areas(intro_text_layout);

    // Match the rule to the same centred 50% band the warning text uses.
    let rule_band = divider_area.centered_horizontally(Constraint::Percentage(50));
    frame.render_widget(
        Paragraph::new("─".repeat(rule_band.width as usize)).fg(palette.accent),
        rule_band,
    );

    // A `Line` is a single row, so the multi-row blocks are split on `\n` here:
    // the translation owns where the sentences break, the render turns each one
    // into its own row.
    let mut warning_rows = vec![
        Line::from(translation.intro.attention.fg(palette.accent).bold()),
        Line::from(""), // blank row for breathing space
    ];
    warning_rows.extend(
        translation
            .intro
            .welcome
            .lines()
            .map(|row| styled_line(row, Style::default().dim(), palette.accent)),
    );
    warning_rows.extend(translation.intro.disclaimer.lines().map(Line::from));
    warning_rows.extend([
        Line::from(""),
        Line::from(""),
        Line::from(
            translation
                .intro
                .continue_btn
                .fg(palette.on_accent)
                .bg(palette.accent)
                .bold(),
        ),
    ]);

    let intro_texts = Text::from(warning_rows);

    frame.render_widget(
        Paragraph::new(intro_texts)
            .white()
            .centered()
            .wrap(Wrap { trim: false }),
        atencao_area.centered_horizontally(Constraint::Percentage(50)),
    );

    let status_texts = hint_line(translation.intro.hints, palette);

    frame.render_widget(
        Paragraph::new(status_texts).block(
            colorfull_bordered_block(Some(Borders::TOP), palette).padding(Padding::new(2, 0, 0, 0)),
        ),
        status_layout,
    );
}

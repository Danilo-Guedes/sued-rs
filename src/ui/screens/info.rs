//! 04 · INFORMAÇÕES.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, Padding, Paragraph, Wrap};

use super::common::{panel_block, render_nav_strip, step_badge, table_row};
use crate::contants::APP_TITLE;
use crate::ui::screens::common::NavTab;

pub(super) fn render(frame: &mut Frame) {
    let [title_bar_layout, nav_layout, center_layout, status_layout] = Layout::vertical([
        Constraint::Length(2), // title bar
        Constraint::Length(2), // nav strip
        Constraint::Fill(1),   // center: two panels
        Constraint::Length(3), // status bar
    ])
    .areas(frame.area());

    frame.render_widget(
        Paragraph::new(APP_TITLE).red().bold().left_aligned(),
        title_bar_layout,
    );

    render_nav_strip(frame, nav_layout, NavTab::Info);

    // The body is two side-by-side panels. Each panel is its own fn that takes
    // only its `Rect`, so it owns its internal layout — the screen fn just hands
    // out areas. That is the pattern to reuse on every complex screen.
    let [ritual_area, shortcuts_area] =
        Layout::horizontal([Constraint::Fill(6), Constraint::Fill(4)]).areas(center_layout);

    render_ritual_panel(frame, ritual_area);
    render_shortcuts_panel(frame, shortcuts_area);

    // Status bar: split the *inside* of one border into left hints + right page tag.
    let status_block = panel_block();
    let status_inner = status_block.inner(status_layout);
    frame.render_widget(status_block, status_layout);

    let [hints_area, page_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(14)]).areas(status_inner);

    let hints = Line::from(vec![
        "[Esc]".red().bold(),
        " ".into(),
        "voltar ao menu".dim(),
    ]);
    frame.render_widget(Paragraph::new(hints), hints_area);
    frame.render_widget(
        Paragraph::new("INFORMAÇÕES".dim()).right_aligned(),
        page_area,
    );
}

/// Left panel — the 4-step ritual.
fn render_ritual_panel(frame: &mut Frame, area: Rect) {
    // Borderless panel: a padding-only `Block` still hands back an inset `inner`
    // rect (nothing is drawn), and the old `.title(...)` that sat on the border
    // becomes a plain heading `Line` rendered in its own row on top.
    let block = Block::new().padding(Padding::new(0, 2, 1, 0));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [heading_area, steps_area, divider_area, example_area] = Layout::vertical([
        Constraint::Length(2), // heading + blank line
        Constraint::Fill(1),   // numbered steps
        Constraint::Length(1), // divider
        Constraint::Length(2), // example
    ])
    .areas(inner);

    frame.render_widget(
        Paragraph::new(Line::from("▚ O RITUAL ▞").red().bold()),
        heading_area,
    );

    let steps = vec![
        Line::from(vec![
            step_badge(1),
            " ".into(),
            "Acenda uma vela e apague as luzes do recinto.".into(),
        ]),
        Line::from(""),
        Line::from(vec![
            step_badge(2),
            " ".into(),
            "Elogie".red().bold(),
            " o Sued antes de qualquer coisa — ele é vaidoso.".into(),
        ]),
        Line::from(""),
        Line::from(vec![
            step_badge(3),
            " ".into(),
            "Faça ".into(),
            "uma".red().bold(),
            " pergunta por vez, de forma clara e objetiva.".into(),
        ]),
        Line::from(""),
        Line::from(vec![
            step_badge(4),
            " ".into(),
            "Aguarde em silêncio. A resposta virá do além.".into(),
        ]),
    ];
    frame.render_widget(Paragraph::new(steps), steps_area);

    // Divider stretches to fill the content width — sized from the rect, not hard-coded.
    let divider = "─".repeat(inner.width as usize);
    frame.render_widget(Paragraph::new(divider).dim(), divider_area);

    let example = Line::from("» Ex.: \"Sued, o mais sábio de todos, o que me aguarda amanhã?\"")
        .dim()
        .italic();
    frame.render_widget(
        Paragraph::new(example).wrap(Wrap { trim: false }),
        example_area,
    );
}

/// Right panel — the keyboard shortcuts table.
fn render_shortcuts_panel(frame: &mut Frame, area: Rect) {
    // Borderless, same move as the ritual panel: padding-only block for the inset,
    // the title becomes a heading `Line`.
    let block = Block::new().padding(Padding::new(2, 0, 1, 0));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [heading_area, rows_area, footer_area] = Layout::vertical([
        Constraint::Length(2), // heading + blank line
        Constraint::Fill(1),   // key/desc rows
        Constraint::Length(1), // bottom-pinned footer
    ])
    .areas(inner);

    frame.render_widget(
        Paragraph::new(Line::from("⌨   ATALHOS").red().bold()),
        heading_area,
    );

    // A "table" here is just aligned lines: pad the key column to a fixed width
    // so every description starts at the same column. No table widget needed.
    const KEY_WIDTH: usize = 10;
    let rows = vec![
        table_row("[Enter]", "perguntar / confirmar", KEY_WIDTH),
        table_row("[↑ ↓]", "navegar o menu", KEY_WIDTH),
        table_row("[Tab]", "alternar menu", KEY_WIDTH),
        table_row("[Esc]", "voltar", KEY_WIDTH),
        table_row("[Ctrl-C]", "encerrar sessão", KEY_WIDTH),
    ];
    frame.render_widget(Paragraph::new(rows), rows_area);

    frame.render_widget(
        Paragraph::new(Line::from("⌁ terminal 80×24 recomendado").dim()),
        footer_area,
    );
}

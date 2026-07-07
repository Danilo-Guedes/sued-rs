//! 01 · INTRO / Invocação.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, Wrap};

use crate::contants::APP_TITLE;
use crate::ui::screens::common::{
    DEFAULT_PADDING, SUED_BANNER, SUED_BANNER_HEIGHT, SUED_BANNER_WIDTH, create_centered_rect,
    panel_block,
};

pub(super) fn render(frame: &mut Frame) {
    let [
        title_bar_layout,
        _,
        page_title_and_sub_layout,
        intro_text_layout,
        _,
        status_layout,
    ] = Layout::vertical([
        Constraint::Length(2),  // title bar,
        Constraint::Fill(1),    // empty
        Constraint::Fill(1),    // page_title_and_sub
        Constraint::Length(15), // intro_text_layout
        Constraint::Fill(1),    // empty
        Constraint::Length(3),  // status bar
    ])
    .areas(frame.area());

    frame.render_widget(
        Paragraph::new(APP_TITLE).red().bold().left_aligned(),
        title_bar_layout,
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
    frame.render_widget(Paragraph::new(SUED_BANNER).red().bold(), banner_rect);

    frame.render_widget(
        Paragraph::new("SUA ÚLTIMA ESPERANÇA DIVINA".dim()).centered(),
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
        Paragraph::new("─".repeat(rule_band.width as usize)).red(),
        rule_band,
    );

    let intro_texts = Text::from(vec![
        Line::from("A T E N Ç Ã O".red().bold()),
        Line::from(""), // blank row for breathing space
        Line::from("Você está prestes a abrir uma porta para o desconhecido."),
        Line::from(""),
        Line::from("Aconselho acender uma vela e apagar as luzes antes de executar."),
        Line::from(""),
        Line::from(vec![
            Span::raw("Para que "),
            Span::raw("SUED ").red().bold(),
            Span::raw("responda, você deve elogiá-lo e em seguida pergunte com clareza."),
        ]),
        Line::from(""),
        Line::from("Pessoas fracas e sensíveis não devem utilizar o programa."),
        Line::from(""),
        Line::from("Tenha muito cuidado com o que você irá perguntar..."),
        Line::from(""),
        Line::from(""),
        Line::from("   CONTINUAR ▸   ".black().on_red().bold()),
    ]);

    frame.render_widget(
        Paragraph::new(intro_texts)
            .white()
            .centered()
            .wrap(Wrap { trim: false }),
        atencao_area.centered_horizontally(Constraint::Percentage(50)),
    );

    let status_texts = Line::from(vec![
        DEFAULT_PADDING.into(),
        "[Enter]".red().bold(),
        " ".into(),
        "continuar".dim(),
        " ".into(),
        "[Esc]".red().bold(),
        " ".into(),
        "sair".dim(),
    ]);

    frame.render_widget(
        Paragraph::new(status_texts).block(panel_block()),
        status_layout,
    );
}

//! 01 · INTRO / Invocação.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Paragraph, Wrap};

use crate::contants::APP_TITLE;

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

    let page_title_and_sub_texts = Text::from(vec![
        Line::from("SUED".red().bold()),
        Line::from(""), // blank row for breathing space
        Line::from("SUA ÚLTIMA ESPERANÇA DIVINA".dim()),
    ]);

    frame.render_widget(
        Paragraph::new(page_title_and_sub_texts).centered(),
        page_title_and_sub_layout,
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
        intro_text_layout.centered_horizontally(Constraint::Percentage(50)),
    );

    let status_texts = Line::from(vec![
        "[Enter]".red().bold(),
        " ".into(),
        "continuar".dim(),
        " ".into(),
        "[Esc]".red().bold(),
        " ".into(),
        "sair".dim(),
    ]);

    frame.render_widget(
        Paragraph::new(status_texts).block(Block::bordered()),
        status_layout,
    );
}

//! 01 · INTRO / Invocação.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Paragraph, Wrap};

use crate::contants::APP_TITLE;

pub(super) fn render(frame: &mut Frame) {
    let [
        title_bar_layout,
        page_title_and_sub_layout,
        intro_text_layout,
        status_layout,
    ] = Layout::vertical([
        Constraint::Length(2), // title bar,
        Constraint::Fill(2),   // page_title_and_sub
        Constraint::Fill(3),   // intro_text_layout
        Constraint::Length(3), // status bar
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
        Line::from("ATENÇÃO".bold()),
        Line::from(""), // blank row for breathing space
        Line::from("Você está prestes a abrir uma porta para o desconhecido."),
        Line::from(""),
        Line::from("Aconselho acender uma vela e apagar as luzes antes de executar o programa."),
        Line::from(""),
        Line::from(
            "Para que SUED responda, você deve elogiá-lo e em seguida perguntar de forma clara.",
        ),
        Line::from(""),
        Line::from("Pessoas fracas e sensíveis não devem utilizar o programa."),
        Line::from(""),
        Line::from("Tenha muito cuidado com o que você irá perguntar..."),
    ]);

    frame.render_widget(
        Paragraph::new(intro_texts)
            .red()
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

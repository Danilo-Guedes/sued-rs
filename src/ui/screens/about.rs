//! 05 · SOBRE O SUED. (placeholder — content still to be built)

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Paragraph};

use crate::contants::APP_TITLE;
use crate::ui::screens::common::table_row;

pub(super) fn render(frame: &mut Frame) {
    let [
        title_bar_layout,
        center_layout,
        footer_layout,
        status_layout,
    ] = Layout::vertical([
        Constraint::Length(2), // title bar
        Constraint::Fill(3),   // center: two panels
        Constraint::Fill(1),
        Constraint::Length(3), // status bar
    ])
    .areas(frame.area());

    frame.render_widget(
        Paragraph::new(APP_TITLE).red().bold().left_aligned(),
        title_bar_layout,
    );

    let [art_area, text_area] =
        Layout::horizontal([Constraint::Fill(4), Constraint::Fill(6)]).areas(center_layout);

    frame.render_widget(
        Paragraph::new("ART".dim()).centered(),
        art_area.centered_vertically(Constraint::Percentage(60)),
    );

    let text_lines = Text::from(vec![
        Line::from("SUED, O ORÁCULO".red().bold()),
        Line::from(" "),
        Line::from(vec![
            Span::raw("Uma entidade antiga que tudo vê e tudo sabe. Preso entre mundos, response às perguntas dos mortais tolos o bastante para invocá-lo -").white(),
            Span::raw("nem sempre com a verdade que deseja ouvir.").red(),
        ]),
    ]);

    let [top_text, bottom_table] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(text_area);

    frame.render_widget(
        Paragraph::new(text_lines)
            .left_aligned()
            .block(Block::bordered()),
        top_text,
    );

    let status_block = Block::bordered();
    let status_inner = status_block.inner(status_layout);
    frame.render_widget(status_block, status_layout);

    const KEY_WIDTH: usize = 12;
    let rows = vec![
        table_row("natureza", "oráculo onisciente", KEY_WIDTH),
        table_row("humor", "vaidoso, sarcástico, imprevisível", KEY_WIDTH),
        table_row("origem", "o além · desconhecida", KEY_WIDTH),
        table_row("runtime", "rust · ratatui · crossterm", KEY_WIDTH),
    ];
    frame.render_widget(Paragraph::new(rows).block(Block::bordered()), bottom_table);

    let footer_text = Paragraph::new(
        "sued-rs v0.1.0 · recriação do clássico brasileiro · use por sua conta e risco",
    )
    .dim()
    .centered();

    frame.render_widget(footer_text.block(Block::bordered()), footer_layout);

    let [hints_area, page_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(14)]).areas(status_inner);

    let hints = Line::from(vec![
        "[Esc]".red().bold(),
        " ".into(),
        "voltar ao menu".dim(),
    ]);
    frame.render_widget(Paragraph::new(hints), hints_area);
    frame.render_widget(Paragraph::new("SOBRE".dim()).right_aligned(), page_area);
}

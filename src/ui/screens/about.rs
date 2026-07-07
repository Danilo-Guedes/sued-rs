//! 05 · SOBRE O SUED. (placeholder — content still to be built)

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, Wrap};

use crate::contants::APP_TITLE;
use crate::ui::screens::common::{NavTab, panel_block, render_nav_strip, table_row};

pub(super) fn render(frame: &mut Frame) {
    let [
        title_bar_layout,
        nav_layout,
        _empty_space,
        center_layout,
        footer_layout,
        status_layout,
    ] = Layout::vertical([
        Constraint::Length(2), // title bar
        Constraint::Length(2), // nav strip
        Constraint::Fill(1),   //empty space
        Constraint::Fill(3),   // center: two panels
        Constraint::Fill(1),
        Constraint::Length(3), // status bar
    ])
    .areas(frame.area());

    frame.render_widget(
        Paragraph::new(APP_TITLE).red().bold().left_aligned(),
        title_bar_layout,
    );

    render_nav_strip(frame, nav_layout, NavTab::About);

    let [art_area, text_area, _empty] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
    ])
    .areas(center_layout);

    frame.render_widget(
        Paragraph::new("ART".dim()).centered(),
        art_area.centered_vertically(Constraint::Length(1)),
    );

    let text_lines = Text::from(vec![
        Line::from("SUED, O ORÁCULO".red().bold()),
        Line::from(" "),
        Line::from(vec![
            Span::raw("Uma entidade antiga que tudo vê e tudo sabe. Preso entre mundos, response às perguntas dos mortais tolos o bastante para invocá-lo - ").white(),
            Span::raw("nem sempre com a verdade que deseja ouvir.").red().bold(),
        ]),
    ]);

    let [_, top_text, bottom_table, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Fill(1),
    ])
    .areas(text_area);

    frame.render_widget(
        Paragraph::new(text_lines)
            .left_aligned()
            .wrap(Wrap { trim: (false) }),
        top_text,
    );

    let status_block = panel_block();
    let status_inner = status_block.inner(status_layout);
    frame.render_widget(status_block, status_layout);

    const KEY_WIDTH: usize = 12;
    let rows = vec![
        table_row("natureza", "oráculo onisciente", KEY_WIDTH),
        table_row("humor", "vaidoso, sarcástico, imprevisível", KEY_WIDTH),
        table_row("origem", "o além · desconhecida", KEY_WIDTH),
        table_row("runtime", "rust · ratatui · crossterm", KEY_WIDTH),
    ];
    frame.render_widget(Paragraph::new(rows), bottom_table);

    let [_, bottom_footer_layout] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).areas(footer_layout);

    let footer_text = Paragraph::new(
        "sued-rs v0.1.0 · recriação do clássico brasileiro · use por sua conta e risco",
    )
    .dim()
    .centered();

    frame.render_widget(footer_text, bottom_footer_layout);

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

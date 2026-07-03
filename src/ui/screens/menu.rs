//! 02 · MENU PRINCIPAL.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Paragraph};

use crate::app::Menu;
use crate::contants::APP_TITLE;

pub(super) fn render(frame: &mut Frame, menu: &Menu) {
    let [title_bar_layout, center_layout, status_layout] = Layout::vertical([
        Constraint::Length(2), // title title_bar_layout,
        Constraint::Fill(1),   // center_layout
        Constraint::Length(3), // status_layout
    ])
    .areas(frame.area());

    frame.render_widget(
        Paragraph::new(APP_TITLE).red().bold().left_aligned(),
        title_bar_layout,
    );

    let [menu_left_layout, disclaim_right_layout] =
        Layout::horizontal([Constraint::Fill(6), Constraint::Fill(4)]).areas(center_layout);

    let mut menu_items_lines: Vec<Line> = vec![];

    for (idx, menu_item) in Menu::ALL.iter().enumerate() {
        let label = if idx == menu.index() {
            menu_item.label().to_string().black().on_red()
        } else {
            menu_item.label().to_string().red()
        };

        menu_items_lines.push(Line::from(label));
    }

    frame.render_widget(Paragraph::new(menu_items_lines), menu_left_layout);

    let displaimer_texts = Text::from(vec![
        Line::from("ATENÇÃO").red(),
        Line::from(" "),
        Line::from("Pessoas fracas e sensíveis não devem utilizar o programa.").red(),
        Line::from(" "),
        Line::from("Acenda uma vela. Apague as luzes.").red(),
        Line::from(" "),
        Line::from("Tenha cuidado com o que irá perguntar...").red(),
    ]);

    frame.render_widget(Paragraph::new(displaimer_texts), disclaim_right_layout);

    let status_texts = Line::from(vec![
        "[↑↓]".red().bold(),
        " ".into(),
        "navegar".dim(),
        " ".into(),
        "[Enter]".red().bold(),
        " ".into(),
        "selecionar".dim(),
        "[Esc]".red().bold(),
        " ".into(),
        "sair".dim(),
    ]);

    frame.render_widget(
        Paragraph::new(status_texts).block(Block::bordered()),
        status_layout,
    );
}

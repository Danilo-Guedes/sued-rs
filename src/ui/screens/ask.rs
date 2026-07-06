//! 03 · MODO PERGUNTA.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Padding, Paragraph, Wrap};

use super::common::{create_centered_rect, render_nav_strip};
use crate::contants::APP_TITLE;
use crate::core::engine::Engine;

pub(super) fn render(frame: &mut Frame, engine: &Engine) {
    let [
        title_bar_layout,
        nav_layout,
        sued_art_layout,
        sued_says_layout,
        sued_logs_layout,
        input_layout,
        status_layout,
    ] = Layout::vertical([
        Constraint::Length(2), // title bar,
        Constraint::Length(1), // nav strip
        Constraint::Fill(3),   // sued_art
        Constraint::Fill(2),   // sued_says
        Constraint::Fill(3),   // sued_logs
        Constraint::Length(5), // input box
        Constraint::Length(3), // status bar
    ])
    .areas(frame.area());

    frame.render_widget(Paragraph::new(APP_TITLE).red().bold(), title_bar_layout);

    // The session badge lives in the nav strip now (per the design), so the title
    // bar is just the title.
    // TODO(you): pass Some(NavTab::Pergunta) to light up the active tab.
    render_nav_strip(frame, nav_layout, None);

    frame.render_widget(Block::bordered().title("sued_art"), sued_art_layout);

    let speak_layout = create_centered_rect(
        sued_says_layout,
        Constraint::Length(60),
        Constraint::Fill(1),
    );

    let default_sued_text = Text::from(vec![
        Line::from("Pergunte-me o que deseja saber, humano..."),
        Line::from(""), // blank row for breathing space
        Line::from(vec![
            Span::raw("— elogie-me antes da pergunta, e ").dim(),
            Span::raw("talvez").red(),
            Span::raw(" eu responda.").dim(),
        ]),
    ]);

    let final_sued_words = match engine.revealed() {
        Some(answer) => Text::from(answer),
        None => default_sued_text,
    };

    let speak_widget = Paragraph::new(final_sued_words)
        .block(
            Block::bordered()
                .border_style(Style::default().fg(Color::Red).bold())
                .title(" SUED FALA ")
                .padding(Padding::new(2, 2, 1, 1)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(speak_widget, speak_layout);

    let default_logs_text = Text::from(vec![
        Line::from(vec![
            Span::raw(">").red(),
            Span::raw(" "),
            Span::raw("conexão com o além estabelecida.").dim(),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(">").red(),
            Span::raw(" "),
            Span::raw("aguardando oferenda do mortal_").dim(),
        ]),
    ]);

    frame.render_widget(
        Paragraph::new(default_logs_text).block(
            Block::bordered()
                .title("sued_logs")
                .padding(Padding::new(2, 2, 1, 1)),
        ),
        sued_logs_layout,
    );

    let typed = Text::from(vec![
        "".into(),
        Line::from(vec![
            " ▶ ".red().bold(),
            Span::raw(engine.visible_buffer()).white(),
        ]),
    ]);

    frame.render_widget(
        Paragraph::new(typed)
            .block(
                Block::bordered()
                    .border_style(Style::default().fg(Color::Red).bold())
                    .title(" input "),
            )
            .wrap(Wrap { trim: false }),
        input_layout,
    );

    let status_block = Block::bordered();
    let status_inner = status_block.inner(status_layout);
    frame.render_widget(status_block, status_layout);

    let [hints_area, page_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(14)]).areas(status_inner);

    let hints = Line::from(vec![
        "[Enter]".red().bold(),
        " ".into(),
        "perguntar".dim(),
        "  ".into(),
        "[Esc]".red().bold(),
        " ".into(),
        "menu".dim(),
        "  ".into(),
        "[Ctrl-C]".red().bold(),
        " ".into(),
        "sair".dim(),
    ]);
    frame.render_widget(Paragraph::new(hints), hints_area);
    frame.render_widget(Paragraph::new("PERGUNTA".dim()).right_aligned(), page_area);
}

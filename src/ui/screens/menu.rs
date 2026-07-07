//! 02 · MENU PRINCIPAL.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use super::common::panel_block;
use crate::app::Menu;
use crate::contants::APP_TITLE;

pub(super) fn render(frame: &mut Frame, menu: &Menu) {
    let [title_bar_layout, center_layout, status_layout] = Layout::vertical([
        Constraint::Length(2), // title bar
        Constraint::Fill(1),   // center: menu | aviso
        Constraint::Length(3), // status bar
    ])
    .areas(frame.area());

    frame.render_widget(
        Paragraph::new(APP_TITLE).red().bold().left_aligned(),
        title_bar_layout,
    );

    // Two columns: the selectable menu (left) and the ATENÇÃO warning (right).
    // Each column is its own fn that owns its inner layout — the screen fn just
    // hands out `Rect`s. Same pattern as info.rs.
    let [menu_area, aviso_area] =
        Layout::horizontal([Constraint::Fill(6), Constraint::Fill(4)]).areas(center_layout);

    render_menu_column(frame, menu_area, menu);
    render_aviso_column(frame, aviso_area);
    render_status_bar(frame, status_layout, menu.index());
}

/// Left column — heading, the selectable list, a divider and a hint.
fn render_menu_column(frame: &mut Frame, area: Rect, menu: &Menu) {
    // Split a fixed block (heading + list + divider, no wrap) from the hint below
    // (which *does* wrap). Keeping them in separate rects lets the long hint reflow
    // while the full-width selection bar simply clips instead of shoving the whole
    // list down a row.
    let [list_area, hint_area] =
        Layout::vertical([Constraint::Length(8), Constraint::Fill(1)]).areas(area);

    let width = list_area.width as usize;
    let mut lines: Vec<Line> = vec![
        Line::from("▚ ESCOLHA SEU DESTINO ▞").red().bold(),
        Line::from(""),
    ];

    for (idx, item) in Menu::ALL.iter().enumerate() {
        let label = item.label();
        if idx == menu.index() {
            // Full-width accent bar. A background only paints the cells that hold a
            // char, so to make the red bar span the column we pad with spaces out to
            // `width` (leaving 2 cells for the `⏎` glyph + a right margin). This is
            // the same "spacing is empty cells you place yourself" idiom as the
            // answer screen — the bar doesn't stretch, you fill it.
            let head = format!(" ▶  {label}");
            let pad = width.saturating_sub(head.chars().count() + 3);
            let bar = format!("{head}{}⏎ ", " ".repeat(pad));
            lines.push(Line::from(bar.black().on_red().bold()));
        } else {
            // Unselected: indented 4 spaces to line up under the ` ▶  ` prefix.
            lines.push(Line::from(format!("    {label}")).red().bold());
        }
    }

    lines.push(Line::from(""));
    // The design's divider stops short of the column edge — about 70% width.
    lines.push(Line::from("─".repeat((width * 7) / 10)).dim());

    frame.render_widget(Paragraph::new(lines), list_area);

    let hint = Line::from(
        "» Faça sua pergunta ao oráculo. Elogie-o primeiro, depois pergunte de forma clara e objetiva.",
    )
    .dim()
    .italic();
    frame.render_widget(Paragraph::new(hint).wrap(Wrap { trim: false }), hint_area);
}

/// Right column — the ATENÇÃO warning, with a bottom-pinned footer.
fn render_aviso_column(frame: &mut Frame, area: Rect) {
    // Spacer-above trick (`Fill(1)` then `Length(2)`) pins the footer to the bottom.
    let [body_area, footer_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(area);

    let body = vec![
        Line::from("⚠ ATENÇÃO").red().bold(),
        Line::from(""),
        Line::from("Pessoas fracas e sensíveis não devem utilizar o programa.").dim(),
        Line::from(""),
        Line::from("Acenda uma vela. Apague as luzes.").dim(),
        Line::from(""),
        Line::from("Tenha cuidado com o que irá perguntar...").dim(),
    ];
    frame.render_widget(Paragraph::new(body), body_area);

    let footer = vec![
        Line::from("☠ ☠ ☠").dim(),
        Line::from("sua última esperança divina").dim().italic(),
    ];
    frame.render_widget(Paragraph::new(footer).centered(), footer_area);
}

/// Bottom status bar — key hints on the left, page tag pinned right.
fn render_status_bar(frame: &mut Frame, area: Rect, selected_menu: usize) {
    let block = panel_block();
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [hints_area, page_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(6)]).areas(inner);

    let hints = Line::from(vec![
        "[↑↓]".red().bold(),
        " ".into(),
        "navegar".dim(),
        "  ".into(),
        "[Enter]".red().bold(),
        " ".into(),
        "selecionar".dim(),
        "  ".into(),
        "[Esc]".red().bold(),
        " ".into(),
        "sair".dim(),
    ]);
    frame.render_widget(Paragraph::new(hints), hints_area);
    frame.render_widget(
        Paragraph::new(format!("{}/{}", selected_menu + 1, Menu::ALL.len()).dim()).right_aligned(),
        page_area,
    );
}

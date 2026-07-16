//! 06 · CONFIGURAÇÃO.
//!
//! MOCK — every value on this screen is hardcoded to the design. Nothing here
//! reads a `Config`, and nothing here moves: the selected chips are frozen where
//! the mockup drew them. Wiring the rows to real state, to the cursor `[↑↓]`
//! drives, and to `Config::save` is the next step.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

use crate::ui::screens::common::{
    NavTab, colorfull_bordered_block, create_centered_rect, create_screen_block, render_nav_strip,
};

/// Width of the centred form column — sized to the longest line inside it (the
/// `[↑↓]`/`[Enter]` hint), so the rows, the divider and the hint share an edge.
const FORM_WIDTH: u16 = 64;

/// Pads the label column so every value starts at the same column.
const LABEL_WIDTH: usize = 12;

pub(super) fn render(frame: &mut Frame) {
    let layout = create_screen_block(frame);

    let [nav_layout, center_layout, status_layout] = Layout::vertical([
        Constraint::Length(4), // nav strip
        Constraint::Fill(1),   // the form
        Constraint::Length(2), // status bar
    ])
    .areas(layout);

    render_nav_strip(frame, nav_layout, NavTab::Config);

    // The whole form is one centred column, so its rows, divider and hint all
    // share a left edge. Height stays `Fill(1)` and the gaps between the groups
    // are `Fill` rows: that lets the form breathe on a tall terminal and still
    // fit the 12 rows it truly needs into an 80×24 one.
    let form_area = create_centered_rect(
        center_layout,
        Constraint::Length(FORM_WIDTH),
        Constraint::Fill(1),
    );

    let [
        heading_area,
        subtitle_area,
        _gap_above_rows,
        rows_area,
        _gap_below_rows,
        divider_area,
        _gap_below_divider,
        confirm_area,
    ] = Layout::vertical([
        Constraint::Length(1), // ▓ CONFIGURAÇÃO ▓
        Constraint::Length(1), // subtitle
        Constraint::Fill(1),
        Constraint::Length(7), // 4 rows + a blank line between each
        Constraint::Fill(1),
        Constraint::Length(1), // red divider
        Constraint::Fill(2),
        Constraint::Length(1), // the oracle's standing confirmation
    ])
    .areas(form_area);

    frame.render_widget(
        Paragraph::new(Line::from("▓ CONFIGURAÇÃO ▓").red().bold()).centered(),
        heading_area,
    );

    frame.render_widget(
        Paragraph::new(
            Line::from("ajuste o ritual ao seu gosto — o oráculo observa")
                .dim()
                .italic(),
        )
        .centered(),
        subtitle_area,
    );

    // Same "a table is just aligned lines" move as the Informações screen: one
    // Paragraph, blank lines for the gaps, no table widget.
    let rows = vec![
        option_row("tema", &["SANGUE", "ÂMBAR", "FÓSFORO"], 0),
        Line::from(""),
        option_row("animações", &["SIM", "NÃO"], 0),
        Line::from(""),
        volume_row("volume", 50),
        Line::from(""),
        option_row("idioma", &["PT-BR", "EN-US", "ES-ES"], 0),
    ];
    frame.render_widget(
        Paragraph::new(rows).block(Block::new().padding(Padding::left(4))),
        rows_area,
    );

    let divider = "─".repeat(divider_area.width as usize);
    frame.render_widget(Paragraph::new(divider).red(), divider_area);

    frame.render_widget(
        Paragraph::new(
            Line::from("† suas escolhas foram registradas no além †")
                .dim()
                .italic(),
        )
        .centered(),
        confirm_area,
    );

    let status_block =
        colorfull_bordered_block(Some(Borders::TOP)).padding(Padding::new(2, 2, 0, 0));
    let status_inner = status_block.inner(status_layout);
    frame.render_widget(status_block, status_layout);

    // The page tag is right-aligned, so its Rect only has to be as wide as the
    // word itself — sized to "CONFIG" the way the menu sizes its own to "1/4",
    // rather than copying the 14 the other screens reserve for "INFORMAÇÕES".
    let [hints_area, page_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(8)]).areas(status_inner);

    // No `[Enter]`: changes apply on the keypress, so there is nothing to commit.
    // A "salvar"/"cancelar" hint here would promise a step the app doesn't have.
    let hints = Line::from(vec![
        "[↑↓]".red().bold(),
        " ".into(),
        "navegar".dim(),
        "    ".into(),
        "[↔]".red().bold(),
        " ".into(),
        "alterar".dim(),
        "    ".into(),
        "[Esc]".red().bold(),
        " ".into(),
        "voltar".dim(),
    ]);
    frame.render_widget(Paragraph::new(hints), hints_area);
    frame.render_widget(Paragraph::new("CONFIG".dim()).right_aligned(), page_area);
}

/// One `label   [CHIP] opt opt` row. The selected option wears the same
/// black-on-red chip as the active nav tab; the others sit dim beside it. Every
/// option is padded to `" {option} "` whether or not it's selected, so the row
/// doesn't shift sideways as the selection moves along it.
fn option_row(label: &str, options: &[&str], selected: usize) -> Line<'static> {
    let mut spans = vec![Span::from(format!("{label:<LABEL_WIDTH$}")).dim()];

    for (i, option) in options.iter().enumerate() {
        if i > 0 {
            spans.push("  ".into());
        }

        let chip = Span::from(format!(" {option} "));
        spans.push(if i == selected {
            chip.black().on_red().bold()
        } else {
            chip.dim()
        });
    }

    Line::from(spans)
}

/// The volume row — a bar filled to `percent`, plus the number it stands for.
fn volume_row(label: &str, percent: u8) -> Line<'static> {
    const BAR_WIDTH: usize = 24;

    let filled = BAR_WIDTH * percent.min(100) as usize / 100;

    Line::from(vec![
        Span::from(format!("{label:<LABEL_WIDTH$}")).dim(),
        Span::from("█".repeat(filled)).red(),
        Span::from("░".repeat(BAR_WIDTH - filled)).dim(),
        Span::from(format!(" {percent}%")).dim(),
    ])
}

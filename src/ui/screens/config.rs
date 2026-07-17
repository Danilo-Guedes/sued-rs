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

use crate::app::App;
use crate::ui::screens::common::{
    NavTab, colorfull_bordered_block, create_centered_rect, create_screen_block, render_nav_strip,
};

const FORM_WIDTH: u16 = 64;

/// Pads the label column so every value starts at the same column.
const LABEL_WIDTH: usize = 12;

pub(super) fn render(frame: &mut Frame, app_state: &mut App) {
    let layout = create_screen_block(frame);

    let current_menu_index = app_state.config_navigation.selected() as usize;

    let [nav_layout, center_layout, status_layout] = Layout::vertical([
        Constraint::Length(4), // nav strip
        Constraint::Fill(1),   // the form
        Constraint::Length(2), // status bar
    ])
    .areas(layout);

    render_nav_strip(frame, nav_layout, NavTab::Config);

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

    let rows = vec![
        option_row(
            "tema",
            &["SANGUE", "ÂMBAR", "FÓSFORO"],
            app_state.config().theme() as usize,
            0,
            current_menu_index,
        ),
        Line::from(""),
        option_row(
            "animações",
            &["SIM", "NÃO"],
            if app_state.config().animations() {
                0
            } else {
                1
            },
            1,
            current_menu_index,
        ),
        Line::from(""),
        volume_row(
            "volume",
            app_state.config().audio_volume(),
            2,
            current_menu_index,
        ),
        Line::from(""),
        option_row(
            "idioma",
            &["PT-BR", "EN-US", "ES-ES"],
            app_state.config().language() as usize,
            3,
            current_menu_index,
        ),
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

    let [hints_area, page_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(8)]).areas(status_inner);

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

fn option_row(
    label: &str,
    options: &[&str],
    selected: usize,
    menu_index: usize,
    current_menu_index: usize,
) -> Line<'static> {
    let mut spans = if menu_index == current_menu_index {
        vec![Span::from(format!("{label:<LABEL_WIDTH$}")).red()]
    } else {
        vec![Span::from(format!("{label:<LABEL_WIDTH$}")).dim()]
    };

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
fn volume_row(
    label: &str,
    percent: u8,
    menu_index: usize,
    current_menu_index: usize,
) -> Line<'static> {
    const BAR_WIDTH: usize = 24;

    let filled = BAR_WIDTH * percent.min(100) as usize / 100;

    let label_span = if menu_index == current_menu_index {
        Span::from(format!("{label:<LABEL_WIDTH$}")).red()
    } else {
        Span::from(format!("{label:<LABEL_WIDTH$}")).dim()
    };

    Line::from(vec![
        label_span,
        Span::from("█".repeat(filled)).red(),
        Span::from("░".repeat(BAR_WIDTH - filled)).dim(),
        Span::from(format!(" {percent}%")).dim(),
    ])
}

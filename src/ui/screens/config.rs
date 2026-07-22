//! 06 · CONFIGURAÇÃO.
//!
//! Renders the live `Configuration` as a form: each row shows its current value —
//! the selected chip lit, the volume bar filled — and the row under the `[↑↓]`
//! cursor wears a red label. Changes are applied by the config arm in `crate::app`,
//! so this screen is pure presentation: it reads state, it never mutates it.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

use crate::app::App;
use crate::config::ConfigOption;
use crate::language::Language;
use crate::ui::screens::common::{
    NavTab, colorfull_bordered_block, create_centered_rect, create_screen_block, render_nav_strip,
};
use crate::ui::theme::Theme;

const FORM_WIDTH: u16 = 64;

/// Pads the label column so every value starts at the same column.
const LABEL_WIDTH: usize = 12;

pub(super) fn render(frame: &mut Frame, app_state: &App) {
    let palette = app_state.config().theme().palette();

    let layout = create_screen_block(frame, palette);

    let config = app_state.config();
    let focused = app_state.focused_option();

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

    let theme_chips: Vec<(&str, bool)> = Theme::ALL
        .into_iter()
        .map(|t| (t.label(), t == config.theme()))
        .collect();
    let language_chips: Vec<(&str, bool)> = Language::ALL
        .into_iter()
        .map(|l| (l.label(), l == config.language()))
        .collect();
    let animation_chips = [("SIM", config.animations()), ("NÃO", !config.animations())];

    let rows = vec![
        option_row("TEMA", &theme_chips, focused == ConfigOption::Theme),
        Line::from(""),
        option_row(
            "ANIMAÇÕES",
            &animation_chips,
            focused == ConfigOption::Animations,
        ),
        Line::from(""),
        volume_row(
            "VOLUME",
            config.audio_volume(),
            focused == ConfigOption::Volume,
        ),
        Line::from(""),
        option_row("IDIOMA", &language_chips, focused == ConfigOption::Language),
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
        colorfull_bordered_block(Some(Borders::TOP), palette).padding(Padding::new(2, 2, 0, 0));
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
    frame.render_widget(
        Paragraph::new("CONFIGURAÇÃO".dim()).right_aligned(),
        page_area,
    );
}

fn option_row(label: &str, chips: &[(&str, bool)], is_focused: bool) -> Line<'static> {
    let mut spans = vec![styled_label(label, is_focused)];

    for (i, &(text, selected)) in chips.iter().enumerate() {
        if i > 0 {
            spans.push("  ".into());
        }

        let chip = Span::from(format!(" {text} "));
        spans.push(if selected {
            chip.black().on_red().bold()
        } else {
            chip.dim()
        });
    }

    Line::from(spans)
}

fn volume_row(label: &str, percent: u8, is_focused: bool) -> Line<'static> {
    const BAR_WIDTH: usize = 24;

    let filled = BAR_WIDTH * percent.min(100) as usize / 100;

    Line::from(vec![
        styled_label(label, is_focused),
        Span::from("█".repeat(filled)).red(),
        Span::from("░".repeat(BAR_WIDTH - filled)).dim(),
        Span::from(format!(" {percent}%")).dim(),
    ])
}

fn styled_label(label: &str, is_focused: bool) -> Span<'static> {
    let span = Span::from(format!("{label:<LABEL_WIDTH$}"));
    if is_focused { span.red() } else { span.dim() }
}

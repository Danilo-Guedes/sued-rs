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
    NavTab, colorfull_bordered_block, create_centered_rect, create_screen_block, hint_line,
    render_nav_strip,
};
use crate::ui::theme::{Palette, Theme};

const FORM_WIDTH: u16 = 64;

/// Pads the label column so every value starts at the same column.
const LABEL_GAP: usize = 3;

pub(super) fn render(frame: &mut Frame, app_state: &App) {
    let config = app_state.config();

    let palette = config.theme().palette();

    let layout = create_screen_block(frame, palette);

    let language = config.language();

    let translation = language.translation();

    let max_label_width = translation.config.max_label_width();

    let focused = app_state.focused_option();

    let [nav_layout, center_layout, status_layout] = Layout::vertical([
        Constraint::Length(4), // nav strip
        Constraint::Fill(1),   // the form
        Constraint::Length(2), // status bar
    ])
    .areas(layout);

    render_nav_strip(
        frame,
        nav_layout,
        NavTab::Config,
        palette,
        language,
        translation,
    );

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
        Constraint::Length(1),
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
        Paragraph::new(
            Line::from(format!("▓ {} ▓", translation.config.configuration))
                .fg(palette.accent)
                .bold(),
        )
        .centered(),
        heading_area,
    );

    frame.render_widget(
        Paragraph::new(Line::from(translation.config.subtitle).dim().italic()).centered(),
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
    let animation_chips = [
        (translation.config.yes, config.animations()),
        (translation.config.no, !config.animations()),
    ];

    let rows = vec![
        option_row(
            translation.config.theme,
            &theme_chips,
            focused == ConfigOption::Theme,
            palette,
            max_label_width,
        ),
        Line::from(""),
        option_row(
            translation.config.animations,
            &animation_chips,
            focused == ConfigOption::Animations,
            palette,
            max_label_width,
        ),
        Line::from(""),
        volume_row(
            translation.config.volume,
            config.audio_volume(),
            focused == ConfigOption::Volume,
            palette,
            max_label_width,
        ),
        Line::from(""),
        option_row(
            translation.config.language,
            &language_chips,
            focused == ConfigOption::Language,
            palette,
            max_label_width,
        ),
    ];
    frame.render_widget(
        Paragraph::new(rows).block(Block::new().padding(Padding::left(4))),
        rows_area,
    );

    let divider = "─".repeat(divider_area.width as usize);
    frame.render_widget(Paragraph::new(divider).fg(palette.accent), divider_area);

    frame.render_widget(
        Paragraph::new(
            Line::from(format!("† {} †", translation.config.footer))
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
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(12)]).areas(status_inner);

    let hints = hint_line(translation.config.hints, palette);
    frame.render_widget(Paragraph::new(hints), hints_area);
    frame.render_widget(
        Paragraph::new(NavTab::Config.label(language).to_uppercase())
            .dim()
            .right_aligned(),
        page_area,
    );
}

fn option_row(
    label: &str,
    chips: &[(&str, bool)],
    is_focused: bool,
    palette: Palette,
    max_label_width: usize,
) -> Line<'static> {
    let mut spans = styled_label(label, is_focused, palette, max_label_width);

    for (i, &(text, selected)) in chips.iter().enumerate() {
        if i > 0 {
            spans.push("  ".into());
        }

        let chip = Span::from(format!(" {text} "));
        spans.push(if selected {
            chip.bg(palette.accent).fg(palette.on_accent).bold()
        } else {
            chip.dim()
        });
    }

    Line::from(spans)
}

fn volume_row(
    label: &str,
    percent: u8,
    is_focused: bool,
    palette: Palette,
    max_label_width: usize,
) -> Line<'static> {
    const BAR_WIDTH: usize = 24;

    let filled = BAR_WIDTH * percent.min(100) as usize / 100;

    let mut spans = styled_label(label, is_focused, palette, max_label_width);
    spans.extend([
        Span::from("█".repeat(filled)).fg(palette.accent),
        Span::from("░".repeat(BAR_WIDTH - filled)).dim(),
        Span::from(format!(" {percent}%")).dim(),
    ]);

    Line::from(spans)
}

fn styled_label(
    label: &str,
    is_focused: bool,
    palette: Palette,
    max_label_width: usize,
) -> Vec<Span<'static>> {
    let text = Span::from(label.to_string());
    let pad = " ".repeat(max_label_width + LABEL_GAP - label.chars().count());
    let text = if is_focused {
        text.bg(palette.accent).fg(palette.on_accent)
    } else {
        text.dim()
    };
    vec![text, Span::from(pad)]
}

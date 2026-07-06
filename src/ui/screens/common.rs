//! Small rendering helpers shared across screens.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

/// Center `area` down to `width` × `height`, discarding the surrounding space.
pub(super) fn create_centered_rect(area: Rect, width: Constraint, height: Constraint) -> Rect {
    let [a] = Layout::horizontal([width]).flex(Flex::Center).areas(area);
    let [a] = Layout::vertical([height]).flex(Flex::Center).areas(a);
    a
}

/// Accent "chip" for a step number: black glyphs on the accent colour.
pub(super) fn step_badge(n: u8) -> Span<'static> {
    Span::from(format!(" {n} ")).black().on_red().bold()
}

/// One aligned `key   description` row. `key_width` pads the key so the
/// descriptions line up into a column.
pub(super) fn table_row(key: &str, desc: &str, key_width: usize) -> Line<'static> {
    Line::from(vec![
        Span::from(format!("{:<width$}", key, width = key_width))
            .red()
            .bold(),
        Span::from(desc.to_string()).dim(),
    ])
}

/// A tab in the decorative top-nav strip. This is *orientation only* — it maps a
/// screen to its label; it does not decide which screen you're on. The set is the
/// four "destinations" (note: not the same as `MenuItem`, which also has `Exit`).
#[derive(Clone, Copy, PartialEq)]
pub(super) enum NavTab {
    Invocacao,
    Pergunta,
    Informacoes,
    Sobre,
}

impl NavTab {
    /// Left-to-right order the tabs are drawn in.
    const ALL: [NavTab; 4] = [
        NavTab::Invocacao,
        NavTab::Pergunta,
        NavTab::Informacoes,
        NavTab::Sobre,
    ];

    fn label(self) -> &'static str {
        match self {
            NavTab::Invocacao => "Invocação",
            NavTab::Pergunta => "Pergunta",
            NavTab::Informacoes => "Informações",
            NavTab::Sobre => "Sobre",
        }
    }
}

/// The shared top-nav strip (tabs on the left, session badge on the right).
///
/// Pure layout: pass `Some(tab)` to highlight the current page, or `None` while
/// you wire that up. The active tab renders uppercased in a red chip; the rest
/// are dim. Deciding *which* tab is active is the caller's job — that's the
/// "selected" logic left for you.
pub(super) fn render_nav_strip(frame: &mut Frame, area: Rect, active: Option<NavTab>) {
    let [tabs_area, session_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(24)]).areas(area);

    // Build the tab row as one Line: label, four spaces, next label, ...
    let mut spans: Vec<Span<'static>> = Vec::new();
    for (i, tab) in NavTab::ALL.iter().enumerate() {
        if i > 0 {
            spans.push("    ".into()); // gap between tabs
        }
        if active == Some(*tab) {
            // Active: uppercased, black-on-red chip (leading/trailing space = padding).
            spans.push(
                Span::from(format!(" {} ", tab.label().to_uppercase()))
                    .black()
                    .on_red()
                    .bold(),
            );
        } else {
            spans.push(tab.label().dim());
        }
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), tabs_area);

    // Session badge — static placeholder for now; wire it to real state later.
    let session = Line::from(vec![
        "sessão #013 ".dim(),
        "· ".dim(),
        "●".red(),
        " online".dim(),
    ]);
    frame.render_widget(Paragraph::new(session).right_aligned(), session_area);
}

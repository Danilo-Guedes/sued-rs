//! Small rendering helpers shared across screens.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

pub const DEFAULT_PADDING: &str = "   ";

/// Center `area` down to `width` × `height`, discarding the surrounding space.
pub(super) fn create_centered_rect(area: Rect, width: Constraint, height: Constraint) -> Rect {
    let [a] = Layout::horizontal([width]).flex(Flex::Center).areas(area);
    let [a] = Layout::vertical([height]).flex(Flex::Center).areas(a);
    a
}

/// The shared accent-red panel frame, so every screen's border colour lives in
/// one place (M5/M6 can swap the theme palette here).
pub(super) fn panel_block() -> Block<'static> {
    Block::bordered().border_style(Style::default().fg(Color::Red))
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
    Intro,
    Ask,
    Info,
    About,
}

impl NavTab {
    const ALL: [NavTab; 4] = [NavTab::Intro, NavTab::Ask, NavTab::Info, NavTab::About];

    fn label(self) -> &'static str {
        match self {
            NavTab::Intro => "Invocação",
            NavTab::Ask => "Pergunta",
            NavTab::Info => "Informações",
            NavTab::About => "Sobre",
        }
    }
}

pub(super) fn render_nav_strip(frame: &mut Frame, area: Rect, active: NavTab) {
    // `area` must now be 2 rows tall: one row of tabs plus the red underline.
    let block = Block::new()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::Red))
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [tabs_area, session_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(24)]).areas(inner);

    // Build the tab row as one Line: label, four spaces, next label, ...
    let mut spans: Vec<Span<'static>> = Vec::new();
    for (i, tab) in NavTab::ALL.iter().enumerate() {
        if i == 0 {
            spans.push(DEFAULT_PADDING.into()) // left padding
        } else {
            spans.push("    ".into()); // gap between tabs
        }
        if active == *tab {
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
        "sessão #666 ".dim(),
        "· ".dim(),
        "●".red(),
        " online".dim(),
        DEFAULT_PADDING.into(),
    ]);
    frame.render_widget(Paragraph::new(session).right_aligned(), session_area);
}

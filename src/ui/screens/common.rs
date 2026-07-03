//! Small rendering helpers shared across screens.

use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};

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

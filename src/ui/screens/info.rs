//! 04 · INFORMAÇÕES.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};

use super::common::{colorfull_bordered_block, render_nav_strip, step_badge, table_row};

use crate::config::Configuration;
use crate::language::Translation;
use crate::ui::screens::common::{NavTab, create_screen_block};
use crate::ui::template::styled_line;
use crate::ui::theme::Palette;

pub(super) fn render(frame: &mut Frame, config: Configuration) {
    let palette = config.theme().palette();

    let language = config.language();

    let translation = language.translation();

    let layout = create_screen_block(frame, palette);

    let [nav_layout, center_layout, status_layout] = Layout::vertical([
        Constraint::Length(4), // nav strip
        Constraint::Fill(1),   // center: two panels
        Constraint::Length(2), // status bar
    ])
    .areas(layout);

    render_nav_strip(
        frame,
        nav_layout,
        NavTab::Info,
        palette,
        language,
        translation,
    );

    // The body is two side-by-side panels. Each panel is its own fn that takes
    // only its `Rect`, so it owns its internal layout — the screen fn just hands
    // out areas. That is the pattern to reuse on every complex screen.
    let [ritual_area, shortcuts_area] =
        Layout::horizontal([Constraint::Fill(6), Constraint::Fill(4)]).areas(center_layout);

    render_ritual_panel(frame, ritual_area, palette, translation);
    render_shortcuts_panel(frame, shortcuts_area, palette, translation);

    // Status bar: split the *inside* of one border into left hints + right page tag.
    let status_block =
        colorfull_bordered_block(Some(Borders::TOP), palette).padding(Padding::new(2, 2, 0, 0));
    let status_inner = status_block.inner(status_layout);
    frame.render_widget(status_block, status_layout);

    let [hints_area, page_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(14)]).areas(status_inner);

    let hints = Line::from(vec![
        "[Esc]".fg(palette.accent).bold(),
        " ".into(),
        "voltar ao menu".dim(),
    ]);
    frame.render_widget(Paragraph::new(hints), hints_area);
    frame.render_widget(
        Paragraph::new("INFORMAÇÕES".dim()).right_aligned(),
        page_area,
    );
}

/// Left panel — the 4-step ritual.
fn render_ritual_panel(frame: &mut Frame, area: Rect, palette: Palette, translation: Translation) {
    // Borderless panel: a padding-only `Block` still hands back an inset `inner`
    // rect (nothing is drawn), and the old `.title(...)` that sat on the border
    // becomes a plain heading `Line` rendered in its own row on top.
    let block = Block::new().padding(Padding::new(0, 2, 1, 0));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Steps take their natural height so the divider + example sit *right under*
    // step 4; the trailing `Fill(1)` sinks the leftover space to the bottom.
    let [
        heading_area,
        steps_area,
        divider_area,
        example_area,
        _spacer,
    ] = Layout::vertical([
        Constraint::Length(2),  // heading + blank line
        Constraint::Length(10), // 4 numbered steps + 3 blank lines between them
        Constraint::Length(1),  // red divider
        Constraint::Length(2),  // example, directly below the last step
        Constraint::Fill(1),    // empty space sinks here
    ])
    .areas(inner);

    frame.render_widget(
        Paragraph::new(Line::from(translation.info.title).fg(palette.accent).bold())
            .block(Block::new().padding(Padding::left(2))),
        heading_area,
    );

    let steps: Vec<_> = translation
        .info
        .instructions
        .iter()
        .enumerate()
        .flat_map(|(idx, instruction)| {
            let mut spans = vec![step_badge(idx + 1, palette), " ".into()];
            spans.extend(styled_line(instruction, Style::default(), palette.accent).spans);
            [Line::from(spans), Line::from("")]
        })
        .collect();

    frame.render_widget(
        Paragraph::new(steps).block(Block::new().padding(Padding::new(2, 0, 1, 0))),
        steps_area,
    );

    // Line separating the steps from the example (sized from the rect).
    let divider = "─".repeat(inner.width as usize);
    frame.render_widget(Paragraph::new(divider).fg(palette.accent), divider_area);

    let example = Line::from(translation.info.example).dim().italic();
    frame.render_widget(
        Paragraph::new(example)
            .wrap(Wrap { trim: false })
            .block(Block::new().padding(Padding::new(2, 0, 1, 0))),
        example_area,
    );
}

/// Right panel — the keyboard shortcuts table.
fn render_shortcuts_panel(
    frame: &mut Frame,
    area: Rect,
    palette: Palette,
    translation: Translation,
) {
    // Borderless, same move as the ritual panel: padding-only block for the inset,
    // the title becomes a heading `Line`.
    let block =
        colorfull_bordered_block(Some(Borders::LEFT), palette).padding(Padding::new(2, 0, 1, 0));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [heading_area, rows_area, footer_area] = Layout::vertical([
        Constraint::Length(2), // heading + blank line
        Constraint::Fill(1),   // key/desc rows
        Constraint::Length(1), // bottom-pinned footer
    ])
    .areas(inner);

    frame.render_widget(
        Paragraph::new(
            Line::from(translation.info.shortcut_title)
                .fg(palette.accent)
                .bold(),
        ),
        heading_area,
    );

    // A "table" here is just aligned lines: pad the key column to a fixed width
    // so every description starts at the same column. No table widget needed.
    const KEY_WIDTH: usize = 10;

    let rows = translation
        .info
        .shortcuts
        .iter()
        .map(|(key, action)| table_row(key, action, KEY_WIDTH, palette))
        .collect::<Vec<_>>();

    frame.render_widget(Paragraph::new(rows), rows_area);

    frame.render_widget(
        Paragraph::new(Line::from(translation.info.terminal_hint).dim()),
        footer_area,
    );
}

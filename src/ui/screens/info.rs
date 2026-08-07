//! 04 · O RITUAL.
//!
//! ⚠ **This screen is addressed to the MARK, and G20 is what made that true.**
//! It used to carry a second panel listing keyboard shortcuts — including
//! `[F5]`, the operator's panic button, which burns the staged answer. Printing
//! that on the page you hand the victim is self-sabotage, so the panel is gone
//! and the operator's key table lives in `--how-it-works`, outside the app,
//! where only the operator can read it.
//!
//! Everything drawn here is in character: light a candle, flatter SueD, ask one
//! question, wait. If you find yourself adding a key to this file, it almost
//! certainly belongs in `cli::how_it_works_text` instead.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};

use super::common::{
    aside, colorfull_bordered_block, hint_line, render_nav_strip, shouldered_heading, step_badge,
};

use crate::config::Configuration;
use crate::constants::RECOMMENDED_TERMINAL_SIZE;
use crate::language::Translation;
use crate::ui::screens::common::{NavTab, create_screen_block};
use crate::ui::template::styled_line;
use crate::ui::theme::Palette;

/// How wide the ritual column is allowed to get.
///
/// ⚠ A bounded, centred column rather than the literal full width G20 sketched.
/// Same argument Danilo already accepted twice (the confirm dialog, the story
/// popover): content that does not grow must not breathe with the terminal.
/// Stretched to 132 columns the four steps would sit as four lonely lines with
/// 80 columns of dead air after them, and the divider would become a red rule
/// across the whole screen. This is also the shape G3 wants — a column that is
/// already size-independent is a column G3 barely has to touch.
const RITUAL_WIDTH: u16 = 76;

pub(super) fn render(frame: &mut Frame, config: Configuration) {
    let palette = config.theme().palette();

    let language = config.language();

    let translation = language.translation();

    let layout = create_screen_block(frame, palette);

    let [nav_layout, center_layout, status_layout] = Layout::vertical([
        Constraint::Length(4), // nav strip
        Constraint::Fill(1),   // center: the ritual
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

    // ⬅ ONE column now, where G20 found two. `.min` rather than a bare `Length`
    // so a terminal narrower than the column still gets everything it has,
    // instead of a rect wider than the screen it is drawn on.
    let [ritual_area] =
        Layout::horizontal([Constraint::Length(RITUAL_WIDTH.min(center_layout.width))])
            .flex(Flex::Center)
            .areas(center_layout);

    render_ritual_panel(frame, ritual_area, palette, translation);

    // Status bar: split the *inside* of one border into left hints + right page tag.
    let status_block =
        colorfull_bordered_block(Some(Borders::TOP), palette).padding(Padding::new(2, 2, 0, 0));
    let status_inner = status_block.inner(status_layout);
    frame.render_widget(status_block, status_layout);

    let [hints_area, page_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(14)]).areas(status_inner);

    let hints = hint_line(translation.info.hints, palette);
    frame.render_widget(Paragraph::new(hints), hints_area);
    frame.render_widget(
        Paragraph::new(NavTab::Info.label(language).to_uppercase())
            .dim()
            .right_aligned(),
        page_area,
    );
}

/// The ritual: a heading, the numbered steps, an example, and the one piece of
/// housekeeping that survived the cut.
fn render_ritual_panel(frame: &mut Frame, area: Rect, palette: Palette, translation: Translation) {
    // Borderless panel: a padding-only `Block` still hands back an inset `inner`
    // rect (nothing is drawn), and the old `.title(...)` that sat on the border
    // becomes a plain heading `Line` rendered in its own row on top.
    let block = Block::new().padding(Padding::new(0, 2, 1, 0));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Steps take their natural height so the divider + example sit *right under*
    // step 4; the `Fill(1)` sinks the leftover space to the bottom, which is
    // what pins the terminal hint to the floor of the screen.
    let [
        _top_spacer,
        heading_area,
        steps_area,
        divider_area,
        example_area,
        _bottom_spacer,
        terminal_hint_area,
    ] = Layout::vertical([
        Constraint::Fill(1),    // ⬅ matching spacers centre the block vertically
        Constraint::Length(2),  // heading + blank line
        Constraint::Length(10), // 4 numbered steps + 3 blank lines between them
        Constraint::Length(1),  // red divider
        Constraint::Length(2),  // example, directly below the last step
        Constraint::Fill(1),    // ⬅ …and the leftover splits evenly between them
        Constraint::Length(1),  // the size hint, bottom-pinned
    ])
    .areas(inner);

    frame.render_widget(
        Paragraph::new(
            Line::from(shouldered_heading(translation.info.title))
                .fg(palette.accent)
                .bold(),
        )
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

    let example = Line::from(aside(translation.info.example)).dim().italic();
    frame.render_widget(
        Paragraph::new(example)
            .wrap(Wrap { trim: false })
            .block(Block::new().padding(Padding::new(2, 0, 1, 0))),
        example_area,
    );

    // ⬅ REHOMED BY G20. This used to be the shortcuts panel's footer, so cutting
    // the panel would have taken it with it. It survives the cut on purpose and
    // on the screen's own terms: "your terminal should be this big" is advice for
    // whoever is *running* the séance, not a key the operator must keep secret —
    // the one line in that panel that was never aimed at the wrong reader.
    frame.render_widget(
        Paragraph::new(
            Line::from(format!(
                "⌁ {}",
                translation
                    .info
                    .terminal_hint
                    .replace("{size}", RECOMMENDED_TERMINAL_SIZE)
            ))
            .dim(),
        )
        .block(Block::new().padding(Padding::left(2))),
        terminal_hint_area,
    );
}

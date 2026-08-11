//! 04 · O RITUAL.
//!
//! ⚠ **This screen is addressed to the MARK.** It deliberately carries no
//! keyboard-shortcut panel — that would mean printing `[F5]`, the operator's
//! panic button, on the page you hand the victim, which is self-sabotage. The
//! operator's key table lives in `--how-it-works`, outside the app, where only
//! the operator can read it.
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
/// ⚠ A bounded, centred column rather than the literal full width. Same
/// argument as the confirm dialog and the story popover: content that does not
/// grow must not breathe with the terminal. Stretched to 132 columns the four
/// steps would sit as four lonely lines with 80 columns of dead air after them,
/// and the divider would become a red rule across the whole screen. It is also
/// the shape the min-size guard wants — a column that is already
/// size-independent is one the guard barely has to touch.
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

    // ⬅ ONE column, not two. `.min` rather than a bare `Length`
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
    // step 4; the single `Fill(1)` sinks all the leftover space to the bottom,
    // which is what pins the terminal hint to the floor of the screen.
    //
    // ⚠ The top gap is a FIXED `Length`, not a matching `Fill`. Two `Fill`s would
    // centre the block — and this block does not grow, so its top gap would then
    // breathe with the terminal: ~10 blank rows at 132×41 and worse on anything
    // taller, which reads as the page having sagged rather than as air. Same rule
    // `RITUAL_WIDTH` follows horizontally, now applied on the other axis.
    let [
        _top_spacer,
        heading_area,
        steps_area,
        divider_area,
        example_area,
        _bottom_spacer,
        terminal_hint_area,
    ] = Layout::vertical([
        Constraint::Length(2),  // ⬅ a breath under the nav strip, and no more
        Constraint::Length(2),  // heading + blank line
        Constraint::Length(10), // 4 numbered steps + 3 blank lines between them
        Constraint::Length(1),  // red divider
        Constraint::Length(2),  // example, directly below the last step
        Constraint::Fill(1),    // ⬅ all the leftover sinks here
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

    // ⬅ This line belongs on the mark's screen, unlike the shortcuts it once sat
    // beside: "your terminal should be this big" is advice for whoever is
    // *running* the séance, not a key the operator must keep secret.
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

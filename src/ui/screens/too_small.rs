//! 00 · THE MIN-SIZE NOTICE — the terminal floor guard.
//!
//! Drawn by `screens::render` *instead of* the app whenever the terminal is
//! below `MIN_TERMINAL_WIDTH`×`MIN_TERMINAL_HEIGHT`. Every other screen in this
//! crate gets to assume it has room; this one exists precisely because it does
//! not, so it is written to a different rule than its neighbours.
//!
//! **The inverted rule: nothing here is fixed-size.** Elsewhere the house style
//! is to compute exact rects and refuse to let `Fill` absorb an arithmetic
//! mistake (see `confirm.rs`). That style is right when you know you have the
//! space and wrong here — a notice that needs 6 rows is useless in a 4-row
//! terminal, which is exactly the case it exists for. So this wraps, clamps to
//! whatever height it is given, and **drops to a two-line form** rather than
//! trusting clipping to eat the right end (it does not: the title wraps first,
//! and the wrap is what pushes the numbers off).
//!
//! ⚠ **No border, and that is deliberate.** `create_screen_block` spends a row
//! and a column on each edge. At the sizes this draws at, that frame is a
//! meaningful fraction of the screen, and it would be spent decorating a message
//! whose whole job is to be readable.

use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};

use super::common::create_centered_rect;
use crate::config::Configuration;
use crate::constants::{MIN_TERMINAL_HEIGHT, MIN_TERMINAL_WIDTH};

/// Below either of these the notice drops to two lines: the size required and
/// the size you have, and nothing else.
///
/// ⚠ **This is not polish, it is the whole contract.** Measured at 24×5 the full
/// notice fails: the English title is 28 characters, so it wraps, and the wrap
/// pushes the *current size* off the bottom — leaving a screen that says
/// something is wrong without saying what to do about it. A resize prompt that
/// cannot be read on a small terminal has one job and misses it.
///
/// 📌 The one place in this crate where a real breakpoint earns its keep. Every
/// other screen sits behind the guard and is allowed to assume it has room.
const COMPACT_BELOW_COLS: u16 = 34;
const COMPACT_BELOW_ROWS: u16 = 7;

pub(super) fn render(frame: &mut Frame, config: Configuration) {
    let palette = config.theme().palette();
    let texts = config.language().translation().too_small;

    let area = frame.area();

    // Paint the theme background across the whole frame first. Without this the
    // notice draws on whatever colour the user's terminal happens to be — and
    // the one screen guaranteed to be seen by someone who has never run the app
    // before is a poor place to let the terminal's own colour show through.
    frame.render_widget(Block::new().style(Style::default().bg(palette.bg)), area);

    let size = |w: u16, h: u16| format!("{w}×{h}");
    let required = size(MIN_TERMINAL_WIDTH, MIN_TERMINAL_HEIGHT);
    let current = size(area.width, area.height);

    // The two numbers are the payload; the words around them are context. So
    // when there is no room for context, the numbers are what stays.
    let lines = if area.width < COMPACT_BELOW_COLS || area.height < COMPACT_BELOW_ROWS {
        vec![
            Line::from(
                Span::from(format!("▲ {required}"))
                    .fg(palette.accent)
                    .bold(),
            ),
            Line::from(Span::from(current).white()),
        ]
    } else {
        vec![
            Line::from(texts.title.fg(palette.accent).bold()),
            Line::from(""),
            Line::from(vec![
                Span::from(format!("{} ", texts.needs)).dim(),
                Span::from(required).fg(palette.accent).bold(),
            ]),
            Line::from(vec![
                Span::from(format!("{} ", texts.has)).dim(),
                Span::from(current).white().bold(),
            ]),
            Line::from(""),
            Line::from(texts.hint).dim().italic(),
        ]
    };

    // ⚠ `min(area.height)` is load-bearing, not defensive tidiness: ask for 7
    // rows inside a 4-row terminal and the centring layout has to solve an
    // impossible constraint. Clamping means the notice starts at the top and
    // loses its tail, which is the failure mode the line order above is built
    // for.
    let lines_len = lines.len();

    frame.render_widget(
        Paragraph::new(lines)
            .centered()
            .wrap(Wrap { trim: false })
            .style(Style::default().bg(palette.bg)),
        create_centered_rect(
            area,
            Constraint::Percentage(100),
            Constraint::Length((lines_len as u16).min(area.height)),
        ),
    );
}

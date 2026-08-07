//! The story popover (G16) — "por trás do véu", drawn *over* the About screen,
//! never a `Screen` of its own. Called from `about::render` while
//! `AboutState::story()` is `Some`.
//!
//! **Every dimension here is derived, never guessed** — the G19 rule, and it
//! earns its keep twice over on this box because the content SCROLLS. Two things
//! follow from that and neither is obvious:
//!
//! 1. **The box sizes itself to its prose, then stops at the band.** When the
//!    story fits it shrinks around it and no scrollbar appears; when it does not,
//!    the box fills the band and the prose scrolls inside. One `min`, both cases.
//! 2. **The far end of the scroll is clamped HERE, not in `handle_key`.**
//!    `StoryView::handle_down` is deliberately unbounded because the last legal
//!    row is `wrapped_rows - viewport`, and neither number exists until the text
//!    has been measured against a width. This is the only place that knows both.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    BorderType, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};

use super::common::{colorfull_bordered_block, create_centered_rect};
use crate::app::StoryView;
use crate::constants::{HOW_IT_WORKS_COMMAND, REPO_URL};
use crate::language::Translation;
use crate::ui::theme::Palette;

/// Total width of the box.
///
/// ⚠ **This is NOT `confirm.rs`'s 62 and the two must not be linked.** There the
/// number is a *floor*: `ask.rs` centres a bordered speak panel at `Length(60)`,
/// so a narrower dialog lets its edges peek past the `Clear` as stray brackets.
/// About draws nothing bordered in its centre, so nothing can peek and there is
/// no floor at all. This 62 is a *measure* — minus the margins and the gutter it
/// leaves 56 columns of prose, inside the 45–75 band where running text stays
/// readable. The match is a coincidence; changing one has no bearing on the other.
const STORY_WIDTH: u16 = 62;

/// Inset from the box edge to the text column. ⚠ `Rect::inner` knows nothing
/// about the block's border, so this margin *contains* it — never subtract the
/// border again on top of it. (4th "which width" bug on the confirm dialog.)
const H_MARGIN: u16 = 2;
const V_MARGIN: u16 = 1;

/// The scrollbar's column — **reserved whether or not the bar is drawn**.
///
/// ⚠ Load-bearing, and the reason is circular if you let it be: a gutter that
/// appeared only once the prose overflows would narrow the text column at that
/// exact moment, wrapping the prose to MORE rows — which can itself be what tips
/// it into overflowing. Reserving it unconditionally makes the measured width a
/// constant and breaks the loop.
const GUTTER: u16 = 2;

/// The rule between the scrolling prose and the pinned signature block.
const RULE_ROWS: u16 = 1;

/// Draw the popover over `band` — the slice of the About screen it may cover.
/// The caller hands over the region, not the dimensions: how big the popover is
/// inside that region is the popover's own business. Same contract as
/// `history::render` and `confirm::render`, deliberately.
pub(super) fn render(
    frame: &mut Frame,
    band: Rect,
    story_view: &StoryView,
    palette: Palette,
    translation: Translation,
) {
    let story = translation.about.story;

    // Widths come from the constants, NOT from the rects further down — the
    // rects do not exist yet, because the box's height depends on measuring text
    // at these very widths. The `debug_assert`s below close the loop by proving
    // the two agree once the layout has run.
    let box_width = STORY_WIDTH.min(band.width);
    let signature_width = box_width.saturating_sub(H_MARGIN * 2);
    let text_width = signature_width.saturating_sub(GUTTER);

    // ⚠ Built ONCE, measured, then rendered — the same binding, never a
    // lookalike. Measuring one `Paragraph` and drawing a second that merely
    // resembles it is how the confirm dialog's lore lost its last line.
    let prose = Paragraph::new(story.body)
        .style(Style::default().white())
        .wrap(Wrap { trim: false });
    let prose_rows = prose.line_count(text_width) as u16;

    // The signature is PINNED — it never scrolls. The URL and the command are
    // the actionable payload for the one confused `cargo install` user this
    // whole popover exists for, and below the fold nobody reaches them.
    //
    // ⚠ It is measured too, not assumed to be 5 rows. The bridge line is the
    // longest string in three languages and the one closest to the column width,
    // so "how many rows is this block" is a translation-dependent question. Give
    // the layout a hardcoded 5 and the language whose bridge wraps loses its
    // command line to a silent clip.
    let signature = Paragraph::new(vec![
        Line::from(story.signature).dim(),
        // The scheme is stripped for the reader's benefit only — `REPO_URL`
        // stays the single source of truth, read from `Cargo.toml` at compile
        // time so it cannot drift from what crates.io publishes.
        Line::from(REPO_URL.trim_start_matches("https://")).fg(palette.accent),
        Line::from(""),
        Line::from(story.bridge).white(),
        Line::from(vec![
            Span::from(story.run_prefix).dim(),
            Span::from("  "),
            Span::from(HOW_IT_WORKS_COMMAND).fg(palette.accent).bold(),
        ]),
    ])
    .wrap(Wrap { trim: false });
    let signature_rows = signature.line_count(signature_width) as u16;

    // Everything the box spends that is not the prose viewport.
    let fixed_rows = RULE_ROWS + signature_rows + V_MARGIN * 2 + 2;

    // ⬅ THE ONE `min` THAT MAKES BOTH CASES WORK. Prose that fits gets a box its
    // own size and no scrollbar; prose that does not gets the whole band and
    // scrolls. Note this is the one number here that is NOT purely derived from
    // content — deliberately, because the band is a hard ceiling and clipping is
    // what happens when you ignore it.
    let box_height = (prose_rows + fixed_rows).min(band.height);

    let popover = create_centered_rect(
        band,
        Constraint::Length(box_width),
        Constraint::Length(box_height),
    );

    // ⚠ Two widgets, one rect, both load-bearing. `Clear` is what stops the
    // demon and the spec table showing through — but its render is literally
    // `buf[(x, y)].reset()`, which resets each cell to the *terminal's* default,
    // NOT to the theme. The block has to paint `palette.bg` back on.
    frame.render_widget(Clear, popover);
    frame.render_widget(
        colorfull_bordered_block(None, palette)
            // Double against the single-line screen chrome: in a terminal you
            // cannot draw a shadow, so weight is the only way to say "on top".
            .border_type(BorderType::Double)
            .style(Style::default().bg(palette.bg))
            .title(format!(" ✦ {} ✦ ", story.title)),
        popover,
    );

    let inner = popover.inner(Margin {
        horizontal: H_MARGIN,
        vertical: V_MARGIN,
    });

    // ⬅ `Fill(1)` IS correct here, and it is worth saying why given G19 banned
    // it. There the ban was about deriving the BOX's height from its content —
    // `Fill` would have absorbed an arithmetic mistake instead of revealing it.
    // Here the box's height is already derived above; the viewport genuinely is
    // "whatever is left once the pinned rows have taken theirs", which is the
    // one thing `Fill` is actually for.
    //
    // ⚠ THE PRICE, found by mutation-testing and worth knowing: an undercount in
    // `fixed_rows` is *invisible*. Shrink it by one and this `Fill` silently
    // hands the viewport one row less; no test goes red, because nothing is
    // clipped — the prose is all still reachable, just one row further down the
    // scroll. That is the failure mode `Fill` trades for, and it is acceptable
    // ONLY because this box scrolls. On a box sized to fit its content the same
    // mistake eats a line for good, which is exactly what happened in G19.
    let [viewport_row, rule_area, signature_area] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(RULE_ROWS),
        Constraint::Length(signature_rows),
    ])
    .areas(inner);

    let [prose_area, gutter_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(GUTTER)]).areas(viewport_row);

    // The loop closed: what was measured is what got laid out. Cheap in debug,
    // absent in release, and it fails loudly at exactly the point where the
    // confirm dialog silently ate a row.
    debug_assert_eq!(
        prose_area.width, text_width,
        "the prose was measured at a width the layout did not hand it"
    );
    debug_assert_eq!(
        signature_area.width, signature_width,
        "the signature was measured at a width the layout did not hand it"
    );

    // ⬅ THE FAR CLAMP, resolved where the numbers exist. `StoryView` counts down
    // from the first line and is deliberately unbounded, so an operator leaning
    // on PgDn would otherwise scroll the prose clean off the top of the box.
    let viewport = prose_area.height;
    let max_offset = prose_rows.saturating_sub(viewport);
    let offset = story_view.rows_from_top().min(max_offset);

    frame.render_widget(prose.scroll((offset, 0)), prose_area);

    // Drawn across the popover's FULL width, not the margin-inset `rule_area`, so
    // it meets the side borders instead of floating two cells short of each edge.
    // `rule_area` still reserves the row; this only widens what goes in it.
    //
    // ⚠ The junctions are written out rather than left to `MergeStrategy::Exact`.
    // Merging a `Borders::TOP` block into the frame gives `╬` — a four-way cross,
    // which claims the rule continues out past the box on both sides. `╟ ╢` is the
    // glyph pair that means "a light rule stopping at a heavy wall", and it is the
    // one the mockup uses.
    let rule = format!("╟{}╢", "─".repeat(popover.width.saturating_sub(2) as usize));
    frame.render_widget(
        Paragraph::new(Line::from(rule).fg(palette.accent)),
        Rect::new(popover.x, rule_area.y, popover.width, RULE_ROWS),
    );
    frame.render_widget(signature, signature_area);

    // Only when there is genuinely something below the fold. A scrollbar over
    // content that fits is a lie about the length of the story.
    if max_offset > 0 {
        let mut scrollbar_state = ScrollbarState::new(max_offset as usize + 1)
            .position(offset as usize)
            .viewport_content_length(viewport as usize);

        let accent = Style::default().fg(palette.accent);

        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .thumb_style(accent)
                .track_style(accent.dim())
                .begin_style(accent)
                .end_style(accent),
            gutter_area,
            &mut scrollbar_state,
        );
    }
}

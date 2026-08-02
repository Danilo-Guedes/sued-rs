//! The transcript popover — drawn *over* the ask screen, never a `Screen` of
//! its own. Called from `ask::render` while `AskingState::history_view()` is
//! `Some`.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Margin, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Clear, Paragraph, Wrap};

use super::common::{colorfull_bordered_block, create_centered_rect};
use crate::conversation::Message;
use crate::language::Translation;
use crate::ui::theme::Palette;

const HORIZONTAL_SPACING: u16 = 2;
const VERTICAL_SPACING: u16 = 2;

/// Draw the popover over `band` — the slice of the ask screen it is allowed to
/// cover. The caller hands over the region, not the dimensions: how big the
/// popover is inside that region is the popover's own business.
pub(super) fn render(
    frame: &mut Frame,
    history: &[Message],
    band: Rect,
    palette: Palette,
    translation: Translation,
) {
    let popover =
        create_centered_rect(band, Constraint::Percentage(80), Constraint::Percentage(95));

    // ⚠ Two widgets, one rect, and both are load-bearing.
    //
    // `Clear` is what stops the ask screen showing through — without it the
    // demon and the reply bleed into every cell the bubbles don't cover. But its
    // render is literally `buf[(x, y)].reset()`, which resets each cell to the
    // *terminal's* default, NOT to the theme.
    //
    // So the block re-asserts `palette.bg` explicitly. Deleting either line
    // looks fine on a terminal whose default happens to be black.
    frame.render_widget(Clear, popover);

    frame.render_widget(
        colorfull_bordered_block(None, palette)
            .title(format!(" † {} ", translation.history.title))
            .style(Style::default().bg(palette.bg)),
        popover,
    );

    let inner_popover = popover.inner(Margin {
        horizontal: 4,
        vertical: 2,
    });

    let [sued_column] = Layout::horizontal([Constraint::Percentage(66)])
        .flex(Flex::Start)
        .areas(inner_popover);

    let [user_column] = Layout::horizontal([Constraint::Percentage(66)])
        .flex(Flex::End)
        .areas(inner_popover);

    //measure the messages height wisth and construct rects

    let mut message_list: Vec<(Rect, Paragraph)> = vec![];

    let mut current_y: u16 = 0;

    for message in history {
        match message {
            Message::Sued(sued_said) => {
                let sued_block = colorfull_bordered_block(None, palette).title(
                    Line::from(" SUED ")
                        .style(Style::default().fg(palette.accent))
                        .left_aligned(),
                );

                let paragraph = Paragraph::new(sued_said.as_str())
                    .wrap(Wrap { trim: false })
                    .block(sued_block);

                let bubble_height =
                    paragraph.line_count(sued_column.width - HORIZONTAL_SPACING) as u16;

                let final_height = current_y + bubble_height;

                let y_inside_popover = current_y + inner_popover.y;

                let bubble_rect = Rect::new(
                    sued_column.x,
                    y_inside_popover,
                    sued_column.width,
                    bubble_height,
                );

                message_list.push((bubble_rect, paragraph));

                current_y = final_height + VERTICAL_SPACING;
            }
            Message::User(user_said) => {
                let user_block = colorfull_bordered_block(None, palette).title(
                    Line::from(format!(" {} ", translation.history.you))
                        .style(Style::default().white().dim())
                        .right_aligned(),
                );

                let paragraph = Paragraph::new(user_said.as_str())
                    .wrap(Wrap { trim: false })
                    .block(user_block);

                let bubble_height =
                    paragraph.line_count(user_column.width - HORIZONTAL_SPACING) as u16;

                let final_height = current_y + bubble_height;

                let y_inside_popover = current_y + inner_popover.y;

                let bubble_rect = Rect::new(
                    user_column.x,
                    y_inside_popover,
                    user_column.width,
                    bubble_height,
                );

                message_list.push((bubble_rect, paragraph));

                current_y = final_height + VERTICAL_SPACING;
            }
        }
    }

    //render the messages

    for (rect, parag) in message_list {
        frame.render_widget(parag, rect);
    }
}

/// Rows to skip from the top of the transcript, resolving a scroll position
/// that is stored **from the bottom**.
///
/// ⚠ The bottom anchor is the whole point, and it is not a stylistic choice.
/// The popover opens on the newest message and `[↓ PgDn]` walk back toward it —
/// but "how far down is the newest message" is `total_rows - viewport`, a number
/// that only exists *inside* the render, after the bubbles have been measured.
/// Key handling has no access to it. Anchoring at the bottom makes the position
/// the keys actually store (`from_bottom`) measurement-free: opening is `0`, and
/// scrolling down is a `saturating_sub` toward `0`. Only the upward direction
/// needs the real height, and that is resolved here, where it is known.
///
/// `total_rows` is the full rendered height of every bubble — borders and the
/// blank spacer rows included — and `viewport` is the popover's inner height.
/// Both arrive from `Paragraph::line_count(width)` at the call site, which keeps
/// this pure arithmetic and ratatui-free.
pub fn scroll_offset(from_bottom: u16, total_rows: u16, viewport: u16) -> u16 {
    total_rows.saturating_sub(viewport.saturating_add(from_bottom))
}

#[cfg(test)]
mod tests {
    use super::scroll_offset;

    #[test]
    fn the_newest_end_of_the_thread_is_the_default_view() {
        // `from_bottom == 0` is what F1 stores, and it must land on the last
        // screenful: 40 rows of transcript in a 15-row window means skipping 25.
        assert_eq!(scroll_offset(0, 40, 15), 25);
    }

    #[test]
    fn scrolling_up_walks_back_from_the_newest_end() {
        // Five rows up from the bottom of the same transcript. This is the one
        // test that pins the direction — get the sign wrong and the popover
        // scrolls the wrong way, which reads as "the keys are inverted".
        assert_eq!(scroll_offset(5, 40, 15), 20);
    }

    #[test]
    fn scrolling_past_the_oldest_message_stops_at_the_top() {
        // ⚠ The clamp, and it is free in this model: 99 rows up from a
        // 25-row-deep bottom saturates at 0 — the top of the thread — instead of
        // underflowing. Nothing above the greeting to show, and nothing to panic
        // over. Same habit as `opened_on_last`, third `usize` trap in this crate.
        assert_eq!(scroll_offset(99, 40, 15), 0);
    }

    #[test]
    fn content_shorter_than_the_viewport_never_scrolls() {
        // Six rows of transcript in a 20-row window. There is empty space below,
        // so scrolling at all would be scrolling into nothing — and the bubbles
        // must stay put rather than drifting off the top.
        assert_eq!(scroll_offset(0, 6, 20), 0);
    }

    #[test]
    fn a_viewport_exactly_the_size_of_the_content_never_scrolls() {
        // The boundary between "fits" and "scrolls", which is the case an
        // off-by-one lands on. 15 rows in a 15-row window: nothing to skip.
        assert_eq!(scroll_offset(0, 15, 15), 0);
    }

    #[test]
    fn an_empty_transcript_does_not_panic() {
        // Unreachable today — the greeting seeds `history` — which is exactly
        // why it is worth pinning: the seeding is the caller's promise, not this
        // function's, and step 4's render indexes whatever comes back.
        assert_eq!(scroll_offset(0, 0, 15), 0);
    }

    #[test]
    fn a_zero_row_viewport_does_not_panic() {
        // A terminal squeezed until the popover has no inner height at all.
        // `total_rows - viewport` is the underflow trap on the other side of the
        // arithmetic; there is nothing to show, but nothing to crash over either.
        assert_eq!(scroll_offset(0, 6, 0), 6);
    }
}

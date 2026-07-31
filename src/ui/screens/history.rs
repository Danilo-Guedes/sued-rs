//! The transcript popover — drawn *over* the ask screen, never a `Screen` of
//! its own. Called from `ask::render` while `AskingState::history_view()` is
//! `Some`.

/// How many rows to skip from the top of the transcript so the selected bubble
/// is the first thing in the viewport.
///
/// There is **no stored scroll offset** — the selection *is* the scroll. That is
/// deliberate: an independent offset would be a second cursor, and two cursors
/// can disagree (you could scroll the `▶` caret off screen while the counter
/// still claims `8 de 8`). Deriving it means the caret, the counter and the
/// scrollbar cannot drift apart.
///
/// `heights` is one entry per message, **as rendered** — borders and the blank
/// spacer row included — so this stays pure arithmetic and never touches
/// ratatui. `viewport` is the inner height available for bubbles.
///
/// The result is clamped so the last screenful sits flush with the end of the
/// transcript instead of scrolling past it into empty rows.
fn scroll_offset(heights: &[u16], selected: usize, viewport: u16) -> u16 {
    todo!("sum the bubbles above `selected`, clamped to the last screenful")
}

#[cfg(test)]
mod tests {
    use super::scroll_offset;

    #[test]
    fn the_first_message_never_scrolls() {
        // Nothing sits above it, so there is nothing to skip.
        assert_eq!(scroll_offset(&[3, 3, 3], 0, 6), 0);
    }

    #[test]
    fn the_offset_is_the_stack_of_bubbles_above_the_selection() {
        // Two 3-row bubbles above index 2 → skip 6 rows. Total is 15 against a
        // 6-row viewport, so the end-clamp (15-6 = 9) is not what's binding here
        // — this test pins the sum, and the sum alone.
        assert_eq!(scroll_offset(&[3, 3, 3, 3, 3], 2, 6), 6);
    }

    #[test]
    fn ragged_bubble_heights_still_add_up() {
        // Wrapping makes every bubble a different height, which is the whole
        // reason this takes a slice rather than a count × a constant.
        // 2 + 5 above index 2 = 7; total 14 against a 3-row viewport clamps at
        // 11, so again the sum is what is under test.
        assert_eq!(scroll_offset(&[2, 5, 3, 4], 2, 3), 7);
    }

    #[test]
    fn content_shorter_than_the_viewport_never_scrolls() {
        // 6 rows of transcript in a 20-row window. Selecting the second message
        // must NOT push the first one off the top — there is empty space below,
        // so scrolling at all would be scrolling into nothing.
        assert_eq!(scroll_offset(&[3, 3], 1, 20), 0);
    }

    #[test]
    fn the_last_screenful_sits_flush_with_the_end() {
        // ⚠ The clamp, and the reason it exists. 20 rows of transcript, an
        // 8-row viewport, selection on the last bubble: the naive sum-above is
        // 16, which would leave 4 rows of transcript and 4 rows of void. The
        // honest answer is 20-8 = 12 — the last screenful, flush.
        assert_eq!(scroll_offset(&[4, 4, 4, 4, 4], 4, 8), 12);
    }

    #[test]
    fn an_empty_transcript_does_not_panic() {
        // Unreachable today (the greeting seeds `history`), which is exactly
        // why it is worth pinning: the seeding is a caller's promise, not this
        // function's.
        assert_eq!(scroll_offset(&[], 0, 10), 0);
    }

    #[test]
    fn a_selection_past_the_end_clips_instead_of_panicking() {
        // ⚠ `HistoryView` clamps `selected`, so this is defence in depth — but
        // it is the test that decides HOW you sum. `heights[..selected]` panics
        // here; `heights.iter().take(selected)` saturates. Same lesson as
        // `saturating_sub`: clip, never blow up in front of a mark.
        assert_eq!(scroll_offset(&[3, 3], 99, 4), 2);
    }

    #[test]
    fn a_zero_row_viewport_does_not_panic() {
        // A terminal squeezed until the popover has no inner height at all.
        // `total - viewport` is the underflow trap; there is nothing to show,
        // but there is also nothing to crash over.
        assert_eq!(scroll_offset(&[3, 3], 1, 0), 3);
    }
}

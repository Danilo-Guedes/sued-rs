//! The leave-confirmation dialog (G19) — drawn *over* the ask screen, never a
//! `Screen` of its own. Called from `ask::render` while the overlay is
//! `Overlay::ConfirmLeave`.
//!
//! ⚠⚠ SCAFFOLD ONLY — this draws the empty frame and nothing inside it.
//! `Clear` so the demon stops bleeding through, the shared bordered block so
//! the box lands somewhere you can see and measure. The title, the warning
//! prose, the question and the two key labels are yours: they need a
//! `ConfirmTexts` in `Translation`, which does not exist yet, and the copy is
//! Phase 6 work. Target is `design-refs/03-c-confirm-leave.png`.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::widgets::Clear;

use super::common::{colorfull_bordered_block, create_centered_rect};
use crate::ui::theme::Palette;

/// Draw the dialog over `band` — the slice of the ask screen it is allowed to
/// cover. The caller hands over the region, not the dimensions: how big the
/// dialog is inside that region is the dialog's own business. Same contract as
/// `history::render`, deliberately.
pub(super) fn render(frame: &mut Frame, band: Rect, palette: Palette) {
    // ⚠ Both numbers are guesses off the mockup — MEASURE them before you trust
    // them, the same way `inner_popover.height` turned out to be 9 and not the 8
    // the plan computed.
    //
    // Height is a fixed `Length`, not a percentage, and that is the one real
    // decision here: unlike the transcript this dialog's content never grows, so
    // it should not breathe with the terminal. The mockup's box is border + four
    // wrapped lines of warning + blank + question + blank + the two labels +
    // border.
    let dialog = create_centered_rect(band, Constraint::Percentage(70), Constraint::Length(11));

    // ⚠ Two widgets, one rect, both load-bearing — the same pairing the
    // transcript needs and for the same reason. `Clear`'s render is literally
    // `buf[(x, y)].reset()`, which resets each cell to the *terminal's* default,
    // NOT to the theme — so the block has to paint `palette.bg` back on.
    frame.render_widget(Clear, dialog);
    frame.render_widget(
        colorfull_bordered_block(None, palette).style(Style::default().bg(palette.bg)),
        dialog,
    );
}

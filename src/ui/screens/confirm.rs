//! The confirmation dialog — drawn *over* whichever screen raised it, never a
//! `Screen` of its own. Target is `design-refs/03-c-confirm-leave.png`.
//!
//! **Two callers, one render (G21).** `ask::render` draws it for
//! `Overlay::ConfirmLeave` (leaving the séance); `intro::render` and
//! `menu::render` draw it for `App::confirm_quit` (leaving the program). They
//! differ only in the `ConfirmTexts` handed in — which is exactly why this takes
//! the texts rather than the whole `Translation`. Reaching for
//! `translation.confirm` in here would silently give the quit dialog the
//! séance's words, and it would still compile and still draw.
//!
//! **Every dimension here is derived, never guessed.** The box is as tall as its
//! own wrapped lore and no taller, and the one width that matters is measured on
//! the exact `Paragraph` that later gets rendered — see `render` for why both of
//! those are load-bearing rather than tidy.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

use super::common::{colorfull_bordered_block, create_centered_rect, draw_chip};
use crate::confirm::ConfirmChoice;
use crate::language::ConfirmTexts;
use crate::ui::theme::Palette;

/// Blank columns between the two chips. They already carry a space of padding
/// each, so the gap you SEE is this + 2.
const CHOICE_GAP: &str = "   ";

/// Fixed, not a percentage — and for the same reason the height is a `Length`:
/// this dialog's content never grows, so it must not breathe with the terminal.
///
/// ⚠ **The floor binds the ASK caller only.** `ask.rs` centres SueD's speak
/// panel at `Constraint::Length(60)` and draws its border even when SueD is
/// silent, so a dialog narrower than 62 lets that panel's left and right edges
/// peek out past this one's `Clear` as two stray brackets. Shrink this and they
/// come back — *on Ask*.
///
/// ⚠⚠ **On Intro and Menu (G21) that argument does not apply, and 62 is a
/// coincidence.** Neither screen has a bordered centre panel this must hide:
/// the intro's banner is 33 wide and the menu's aviso column is `Borders::LEFT`
/// only — a single vertical rule the box simply interrupts. They share the
/// width because three identical dialogs is worth more than three tuned ones,
/// **not** because anything forces it. Do not cite the speak panel for them.
const DIALOG_WIDTH: u16 = 62;

/// Inset from the dialog's edge to the text column. This is the knob that keeps
/// the box wide enough to hide the speak panel while the prose still breaks into
/// the mockup's four lines: 62 - 8*2 = 46 columns of text.
///
/// ⚠ `Rect::inner` knows nothing about the block's border, so this margin
/// *contains* it — never subtract the border again on top of it.
const H_MARGIN: u16 = 8;
const V_MARGIN: u16 = 2;

/// Rows the layout below spends on everything that is not the lore:
/// gap + question + gap + buttons.
const CHROME_ROWS: u16 = 4;

/// How much of `band` the dialog blanks out.
///
/// ⚠ This is not a knob — the three callers genuinely differ, and the choice is
/// forced by what each screen puts *beside* the box, which only a buffer dump
/// will tell you.
pub(super) enum Cover {
    /// Blank the box's own rect and nothing else, so whatever shares its rows
    /// stays visible.
    ///
    /// `ask.rs` and `menu.rs` want this. On Ask it is the design — the séance
    /// you are being asked to abandon should still be running around the
    /// question. On the menu there is only the aviso column's `Borders::LEFT`,
    /// a vertical rule the box interrupts, which is what "on top" should look
    /// like.
    JustTheBox,
    /// Blank `band`'s **full width** across the box's rows.
    ///
    /// `intro.rs` needs this, and it is a fix rather than a preference: the
    /// intro centres its rule and its warning text at `Percentage(50)` of the
    /// terminal, so past ~124 columns they are **wider than the box** and their
    /// ends surface either side of it — the rule as two stray dashes welded to
    /// the border, the warning as orphaned words. ⚠ A wider box does not fix
    /// that: the content scales with the terminal, so at 200 columns it is 100
    /// wide and escapes any fixed width.
    TheWholeRow,
}

/// Draw the dialog over `band` — the slice of the calling screen it is allowed
/// to cover. The caller hands over the region, not the dimensions: how big the
/// dialog is inside that region is the dialog's own business. Same contract as
/// `history::render`, deliberately, and it is what lets three screens share this
/// render while each keeps its own idea of what must stay visible.
pub(super) fn render(
    frame: &mut Frame,
    band: Rect,
    choice: ConfirmChoice,
    palette: Palette,
    texts: ConfirmTexts,
    cover: Cover,
) {
    let dialog_width = DIALOG_WIDTH.min(band.width);
    let text_width = dialog_width.saturating_sub(H_MARGIN * 2);

    // ⚠ Built ONCE, measured, then rendered — the same value, never a lookalike.
    // Measuring one `Paragraph` and rendering a second one that merely resembles
    // it is how the lore lost its last line: `line_count` answered for 46 columns
    // while the widget landed in 44, so the text needed a row the box had not
    // reserved and ratatui silently clipped it. Keep these one binding and the
    // two cannot drift.
    let lore = Paragraph::new(texts.lore_text)
        .centered()
        .style(Style::default().white())
        .wrap(Wrap { trim: false });
    let lore_height = lore.line_count(text_width) as u16;

    // The box is exactly its content: the margin the layout will inset, the lore
    // as measured, and the four fixed rows under it. Derived from the same
    // constants the layout uses, so a change to either cannot desync them.
    let dialog = create_centered_rect(
        band,
        Constraint::Length(dialog_width),
        Constraint::Length(lore_height + CHROME_ROWS + V_MARGIN * 2),
    );

    // What gets wiped before the box lands on top — the box alone, or the
    // band's full width across the box's rows. See `Cover`.
    let blanked = match cover {
        Cover::JustTheBox => dialog,
        Cover::TheWholeRow => Rect {
            x: band.x,
            y: dialog.y,
            width: band.width,
            height: dialog.height,
        },
    };

    // ⚠ Two widgets, one rect, both load-bearing — the same pairing the
    // transcript needs and for the same reason. `Clear`'s render is literally
    // `buf[(x, y)].reset()`, which resets each cell to the *terminal's* default,
    // NOT to the theme — so `palette.bg` has to be painted back on.
    //
    // ⚠ The repaint covers `blanked`, NOT `dialog`. Under `TheWholeRow` those
    // differ, and letting the bordered block do the repainting on its own rect
    // would leave the strip either side of it reset to the terminal's colour —
    // a terminal-coloured hole punched straight through the G4-enforced
    // background, which is the one thing `Color::Reset` is banned for.
    frame.render_widget(Clear, blanked);
    frame.render_widget(Block::new().style(Style::default().bg(palette.bg)), blanked);
    frame.render_widget(
        colorfull_bordered_block(None, palette)
            .style(Style::default().bg(palette.bg))
            .title(format!(" † {} † ", texts.title)),
        dialog,
    );

    // Five rows, every one a `Length`, summing to exactly the inner height. No
    // `Fill` anywhere on purpose: `Fill` would quietly swallow an arithmetic
    // mistake by absorbing the slack, which is precisely how the lore ended up
    // one row short of what it needed.
    let [lore_area, _, question_area, _, buttons_area] = Layout::vertical([
        Constraint::Length(lore_height),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(dialog.inner(Margin {
        horizontal: H_MARGIN,
        vertical: V_MARGIN,
    }));

    frame.render_widget(lore, lore_area);

    frame.render_widget(
        Paragraph::new(texts.abandon_question)
            .centered()
            .style(Style::default().fg(palette.accent)),
        question_area,
    );

    // One `Line` holding both chips, centred as a pair — NOT two half-width
    // columns, which would strand each label at its own quarter point. The
    // highlight rides the `Span`, so it is the width of the text; a `Paragraph`
    // background would flood the whole rect instead.
    //
    // `leaving` is derived once and both chips are built from it, so "both lit"
    // and "both dim" are unrepresentable — the failure a two-armed `match` on
    // `ConfirmChoice` invites.
    let leaving = matches!(choice, ConfirmChoice::Leave);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            draw_chip(format!(" {} ", texts.leave), leaving, palette),
            Span::from(CHOICE_GAP),
            draw_chip(format!(" {} ", texts.stay), !leaving, palette),
        ]))
        .centered(),
        buttons_area,
    );
}

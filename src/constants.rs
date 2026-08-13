/// The repository, read from the one place it is already declared.
///
/// ⚠ Not in the three translation tables, and not typed out again here: a URL
/// carries no language, and `Cargo.toml` already has to hold it for crates.io.
/// Three hand-copied strings would be three chances to drift the day the repo
/// moves — the same argument that put `RECOMMENDED_TERMINAL_SIZE` below.
///
/// 📌 This is the SOURCE, and it belongs with the operator's manual rather than
/// in the story popover: reading the repo is how you learn the trick, which is
/// precisely what `--how-it-works` is for and what the on-screen popover is
/// deliberately not.
pub const REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");

/// The author, as opposed to the project. These are what the story popover
/// shows — someone who read a personal memory wants the person, not the crate.
pub const AUTHOR_GITHUB: &str = "https://github.com/Danilo-Guedes";

pub const AUTHOR_LINKEDIN: &str = "https://linkedin.com/in/danilo-guedes-dev";

/// The operator's manual, advertised at the foot of the story popover.
///
/// ⚠ Keep this in step with `cli::Args::how_it_works`. It is a *string* naming a
/// flag, so nothing in the type system ties the two together — rename the flag
/// and the popover cheerfully goes on advertising the old one. The tests around
/// `cli::how_it_works_text` cover the manual's contents, not its name.
pub const HOW_IT_WORKS_COMMAND: &str = "sued-rs --how-it-works";

/// The terminal size the info screen tells people to use.
///
/// It lives here rather than inside the three translations because it is a fact
/// about the app, not a piece of language — three copies of a number are three
/// chances to drift, and the string it replaced had drifted from reality
/// entirely (it recommended `80×24`, inherited from the VT100 default and never
/// measured; the app in fact breaks below the floor defined two constants down).
///
/// This is the *comfortable* size — measured as the one where everything
/// renders correctly. The hard floor is lower
/// (`MIN_TERMINAL_WIDTH`×`MIN_TERMINAL_HEIGHT` below); if the design is ever
/// compacted or the layout banded, this is the one line that changes.
pub const RECOMMENDED_TERMINAL_SIZE: &str = "132×41";

/// Below this, `ui::screens::render` draws the "resize me" notice instead of the
/// app (the min-size guard).
///
/// ⚠ **This is a FLOOR, not a preference.** It means "below here the app is
/// broken", not "below here it is cramped" — so it is the *measured* minimum and
/// deliberately NOT `RECOMMENDED_TERMINAL_SIZE`. Guarding at the comfortable
/// size would refuse to run at sizes where every screen renders perfectly well.
///
/// **Where the width comes from: the NAV STRIP** — five destination labels plus
/// the `session #999 · ● online` block. Measured at 94 (EN, the longest set),
/// 93 (ES), 90 (PT); the widest wins because the floor is one number for all
/// three languages.
///
/// ⚠ **AMENDED — this was 121 for exactly one commit.** The decoy used to be the
/// binding constraint, because the input box had a single text row and dropped
/// whatever would not fit on it. Giving that box a second row (`INPUT_TEXT_ROWS`)
/// took the decoy out of the running entirely: it now needs ~65 columns, so the
/// nav strip inherits the floor and the app runs 27 columns narrower.
/// `the_minimum_width_still_fits_the_longest_decoy` keeps watch on the loser.
pub const MIN_TERMINAL_WIDTH: u16 = 94;

/// The longest decoy in any language, in characters.
///
/// ⚠ A hand-copied fact about data that lives elsewhere, which is exactly the
/// shape that rots — so `the_minimum_width_still_fits_the_longest_decoy`
/// recomputes it from all three decoy pools and fails by name if a decoy is
/// edited past it. ⚠ **Do NOT re-derive this from `MIN_DECOY_CHARS`: that is a
/// test assertion, a LOWER bound on decoy length, not a description of the
/// pool** — reading it as the pool's width is how the floor stayed 29 columns
/// too narrow for months.
#[cfg(test)]
pub const LONGEST_DECOY_CHARS: u16 = 113;

/// Text rows the input box gives the decoy.
///
/// ⚠ **Two, and never grown on demand** — see the layout comment
/// in `ask.rs`. Growing at the wrap point would shift the demon and everything
/// above it up a row *mid-typing*, i.e. the screen twitches at the exact moment
/// the operator is staging the answer in front of the mark.
pub const INPUT_TEXT_ROWS: u16 = 2;

/// Columns the input box spends on frame rather than on the decoy: the screen's
/// two outer border columns, the input block's own two, and the `" ▶ "` prompt.
///
/// ⚠ Measured, not counted off the source — the sweep rendered decoys of known
/// length and read back the width at which the last character survived. Counting
/// padding literals is what produced the estimates it had to replace.
#[cfg(test)]
pub const INPUT_CHROME_COLS: u16 = 8;

/// Below this the demon's ASCII art clips — it is 11 rows against its `Fill(3)`
/// share of what the fixed rows leave, so `(H − 10) × 3/8 ≥ 11` ⇒ `H ≥ 39.33`.
///
/// ⚠ **AMENDED 39 → 40 by the second input row.** Fixed rows went 9 → 10
/// (nav 4, input 4, status 2), which is the entire price of `INPUT_TEXT_ROWS`:
/// one row of height, bought with 27 columns of width. Cheap, because width was
/// the binding problem and height was not.
///
/// 📌 That arithmetic was written *before* anything was measured, and the sweep
/// landed on exactly the row it predicted — both times, before and after the
/// layout changed. The model can be trusted for the demon.
pub const MIN_TERMINAL_HEIGHT: u16 = 40;

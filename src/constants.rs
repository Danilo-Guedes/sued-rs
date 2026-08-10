pub const APP_TITLE: &str = " ☠  SueD — O Oráculo  ☠ ";

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
/// measured; §J.7 found the app actually breaks below ~92×40).
///
/// This is the *comfortable* size — measured as the one where everything
/// renders correctly, and the size the mockups were drawn at. The hard floor is
/// lower (`MIN_TERMINAL_WIDTH`×`MIN_TERMINAL_HEIGHT` below); G3 may yet compact
/// the design or band the layout, and when it does, this is the one line that
/// changes.
pub const RECOMMENDED_TERMINAL_SIZE: &str = "132×41";

/// Below this, `ui::screens::render` draws the "resize me" notice instead of the
/// app (G3's min-size guard).
///
/// ⚠ **This is a FLOOR, not a preference.** It means "below here the app is
/// broken", not "below here it is cramped" — so it is the *measured* minimum and
/// deliberately NOT `RECOMMENDED_TERMINAL_SIZE`. Guarding at the comfortable
/// size would refuse to run at sizes where every screen renders perfectly well.
///
/// **Where the width comes from: the decoy, and nothing else.** §J.7-bis swept
/// every screen a column at a time and found the binding constraint is the
/// staged decoy — the trick itself — which must never clip. The widest decoy in
/// any language is 113 chars, and `INPUT_CHROME_COLS` of frame sits around it.
/// ⚠ Do not re-derive this from `MIN_DECOY_CHARS`: **that constant is a test
/// assertion — a LOWER bound on decoy length, not a description of the pool** —
/// and reading it as the pool's width is exactly how §J.7 came to understate
/// this floor by 29 columns. `the_minimum_width_still_fits_the_longest_decoy`
/// recomputes it from the real decoys, so editing them cannot silently outgrow
/// this number.
///
/// 📌 It is high **because the input box has one row and drops its overflow**.
/// Give that box horizontal scrolling and this collapses to the nav strip's ~94.
pub const MIN_TERMINAL_WIDTH: u16 = LONGEST_DECOY_CHARS + INPUT_CHROME_COLS;

/// The longest decoy in any language, in characters.
///
/// ⚠ A hand-copied fact about data that lives elsewhere, which is exactly the
/// shape that rots — so `the_minimum_width_still_fits_the_longest_decoy`
/// recomputes it from all three decoy pools and fails by name if a decoy is
/// edited past it. **Spelled out as its own constant rather than folded into
/// the 121 so that the floor's provenance is in the code, not in a comment.**
pub const LONGEST_DECOY_CHARS: u16 = 113;

/// Columns the input box spends on frame rather than on the decoy: the screen's
/// two outer border columns, the input block's own two, and the `" ▶ "` prompt.
///
/// ⚠ Measured, not counted off the source — §J.7-bis rendered decoys of known
/// length and read back the width at which the last character survived. Counting
/// padding literals is what produced the estimates §J.7 had to replace.
pub const INPUT_CHROME_COLS: u16 = 8;

/// Below this the demon's ASCII art clips — it is 11 rows against its `Fill(3)`
/// share of `H − 9`, so `(H − 9) × 3/8 ≥ 11` ⇒ `H ≥ 39.33`.
///
/// 📌 That arithmetic was written in §J.7 *before* anything was measured, and
/// the sweep landed on exactly 39. The model can be trusted for the demon.
pub const MIN_TERMINAL_HEIGHT: u16 = 39;

//! CLI args / flags via `clap` derive. Carries the M3 `--no-sound` switch and
//! the M5 `--config` override.

use std::path::PathBuf;

use clap::Parser;

use crate::app::{SHORT_QUESTION_CHARS, THUNDER_AT_CHARS_REMAINING};
use crate::constants::REPO_URL;
use crate::language::Translation;

#[derive(Parser, Debug)]
#[command(
    name = "sued-rs",
    about = "SueD, o oráculo — a horror-themed prank oracle for your terminal."
)]
pub struct Args {
    /// Run with no audio at all (overrides the `audio` build feature).
    #[arg(long)]
    pub no_sound: bool,

    /// Use a specific config file instead of the platform default
    /// (`~/.config/sued-rs/sued.config.json`).
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Explain the trick and how to operate it, then exit.
    #[arg(long)]
    pub how_it_works: bool,
}

/// The operator's manual, in the configured language, with the repository
/// spliced in.
///
/// ⚠⚠ **THIS IS A FLAG AND NOT A SCREEN, AND THAT IS THE WHOLE DESIGN.** Every
/// screen in this app is addressed to the *mark*. Putting "press `;` to secretly
/// type the answer" anywhere on screen means it can be read over the operator's
/// shoulder, or found by a bored mark poking around while the operator fetches a
/// drink. A flag lives outside the performance: it prints to scrollback before
/// the app ever launches, and the mark is never the one at the keyboard.
///
/// 📌 The `{markers}` are substituted here rather than typed into three tables —
/// same treatment as `RECOMMENDED_TERMINAL_SIZE`, and for the same reason: a URL
/// and two tuning constants are facts about the program, not pieces of language.
///
/// ⚠ The two numbers are not decoration. A manual that tells the operator the
/// thunder lands at 15 characters when the constant says 20 is worse than one
/// that says nothing — they will mistime the reveal and blame themselves. That
/// exact drift happened in the first draft, which is why these are spliced from
/// the constants instead of written out.
///
/// 📌 The markers spell the constants' real names, unlike `{size}` and `{repo}`.
/// Deliberate: for a plain fact the short name reads better, but for a tuning
/// knob you want `grep SHORT_QUESTION_CHARS` to find both the definition and
/// every sentence that quotes it.
pub fn how_it_works_text(translation: Translation) -> String {
    translation
        .how_it_works
        .replace("{repo}", REPO_URL)
        .replace("{SHORT_QUESTION_CHARS}", &SHORT_QUESTION_CHARS.to_string())
        .replace(
            "{THUNDER_AT_CHARS_REMAINING}",
            &THUNDER_AT_CHARS_REMAINING.to_string(),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::Language;

    #[test]
    fn nothing_unsubstituted_ever_reaches_the_operator() {
        // ⚠ Written as "no `{` survives" rather than as one assertion per
        // marker, and that generality is the point: `{SHORT_QUESTION_CHARS}`
        // shipped raw to Danilo's terminal precisely because it was a NEW marker
        // and the test only knew about `{repo}`. A per-marker test can only ever
        // catch the markers someone remembered to add to it.
        //
        // No manual has a legitimate `{` in its prose, so the blunt rule costs
        // nothing and covers every marker anyone adds from here on.
        for language in Language::ALL {
            let manual = how_it_works_text(language.translation());

            assert!(
                !manual.contains('{'),
                "{:?}: an unsubstituted placeholder reached the output — the \
                 manual still contains a `{{`",
                language.label()
            );
        }
    }

    #[test]
    fn the_manual_carries_the_facts_it_is_spliced_from() {
        // The other half of the test above: substitution can also happen and be
        // WRONG. Each of these is a fact the operator acts on — the thunder cue
        // is a timing signal, and being off by five characters means mistiming
        // the reveal.
        for language in Language::ALL {
            let manual = how_it_works_text(language.translation());

            assert!(
                manual.contains(REPO_URL),
                "{:?}: the manual must carry the repository — it is the only \
                 place the source is offered, now that the story popover shows \
                 the author's links instead",
                language.label()
            );
            assert!(
                manual.contains(&SHORT_QUESTION_CHARS.to_string()),
                "{:?}: the manual quotes the rebuke threshold, so it must be the \
                 REAL one",
                language.label()
            );
            assert!(
                manual.contains(&THUNDER_AT_CHARS_REMAINING.to_string()),
                "{:?}: the manual quotes when the thunder lands, so it must be \
                 the REAL count",
                language.label()
            );
        }
    }

    #[test]
    fn the_manual_stays_inside_a_narrow_terminal() {
        // ⚠ THE FORMATTING RULE, MADE ENFORCEABLE. This text is hard-wrapped by
        // hand — there is no wrapping layer between it and stdout — so a line
        // someone types past the margin does not wrap gracefully, it wraps
        // wherever the terminal happens to end, mid-word, while every other line
        // stays neat. That is what it looked like the first time, and eyeballing
        // is not a control.
        //
        // ⚠ Checked AFTER substitution: the markers are longer than the numbers
        // they become, so measuring the raw string would pass lines that are
        // actually fine and, worse, could hide ones that are not.
        //
        // 80 because that is the narrowest terminal anyone still uses; the prose
        // is wrapped nearer 72 to leave room for a translation to run long.
        const MAX_COLUMNS: usize = 80;

        for language in Language::ALL {
            for line in how_it_works_text(language.translation()).lines() {
                // `chars()`, not `len()` — these languages are full of accented
                // characters, and a `u8` count would flag correct lines as long.
                assert!(
                    line.chars().count() <= MAX_COLUMNS,
                    "{:?}: this line is {} columns and will wrap raggedly on a \
                     narrow terminal:\n{line}",
                    language.label(),
                    line.chars().count()
                );
            }
        }
    }

    #[test]
    fn the_manual_actually_teaches_the_toggle() {
        // ⚠ The one fact this flag exists to carry. Everything else in it is
        // context; a manual that explains the mood and forgets `;` has failed at
        // its only job, and nothing else in the suite would notice.
        for language in Language::ALL {
            assert!(
                how_it_works_text(language.translation()).contains(';'),
                "{:?}: the manual must name the `;` toggle",
                language.label()
            );
        }
    }
}

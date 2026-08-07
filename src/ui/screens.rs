//! ratatui draw code — one submodule per screen; `render` dispatches on `App`.

mod about;
mod ask;
mod common;
mod config;
mod confirm;
mod history;
mod info;
mod intro;
mod menu;

use ratatui::Frame;

use crate::app::{App, Screen};

pub fn render(frame: &mut Frame, app: &App) {
    match app.screen() {
        Screen::Intro => intro::render(frame, app.config()),
        Screen::Menu => menu::render(frame, app.menu(), app.config()),
        Screen::Asking(asking_state) => ask::render(frame, app, asking_state),
        Screen::Info => info::render(frame, app.config()),
        Screen::About => about::render(frame, app.config()),
        Screen::Config => config::render(frame, app),
    }
}

/// Render smoke tests — the coverage gap that let a panic AND an inverted
/// branch both ship green.
///
/// ⚠ **Before this module, NOT ONE of the project's 247 tests ever called
/// `render`.** They all drive `handle_key` and read state back. So the entire
/// draw path — every `unwrap`, every layout arithmetic, every `Option` branch —
/// had zero coverage, which is exactly how G11's refactor produced an app that
/// crashed on the first frame of the ask screen while the suite stayed green.
///
/// These do not check what the screen *looks* like — that is still verified by
/// running it, and pinning pixels would make every visual tweak a test failure.
/// They check the one thing a unit test genuinely can: **drawing this state does
/// not panic, at sizes real terminals actually are.** That is a floor, not a
/// ceiling, but it is the floor that was missing.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{AppFlow, AskingState};
    use crate::config::Configuration;
    use crate::conversation::Overlay;
    use crate::core::engine::KeyPress;
    use crate::language::Translation;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use std::time::Duration;

    /// The measured floor (§J.7) and a comfortable size. The small one matters:
    /// layout arithmetic that only ever ran on a maximised terminal is precisely
    /// where a subtraction underflows.
    const SIZES: [(u16, u16); 3] = [(132, 41), (92, 40), (80, 24)];

    /// Draw `app` at every size. Panics propagate — that is the whole point.
    fn draw(app: &App) {
        for (width, height) in SIZES {
            let backend = TestBackend::new(width, height);
            let mut terminal =
                Terminal::new(backend).expect("TestBackend must build a terminal in-memory");
            terminal
                .draw(|frame| render(frame, app))
                .unwrap_or_else(|e| panic!("draw failed at {width}x{height}: {e}"));
        }
    }

    fn app_after(keys: &[KeyPress]) -> App {
        let mut app = App::new(Configuration::default());
        for &key in keys {
            assert_eq!(
                app.handle_key(key),
                AppFlow::Stay,
                "fixture keys must not quit the app"
            );
        }
        app
    }

    #[test]
    fn every_screen_draws_in_its_opening_state() {
        // The exact bug this module exists for: a fresh ask screen has
        // `reply: None`, and the refactor's `reply.unwrap()` panicked on frame
        // one — before a single character reached the terminal.
        let screens: [(&str, &[KeyPress]); 6] = [
            ("intro", &[]),
            ("menu", &[KeyPress::Enter]),
            ("ask", &[KeyPress::Enter, KeyPress::Enter]),
            ("info", &[KeyPress::Enter, KeyPress::Down, KeyPress::Enter]),
            (
                "about",
                &[
                    KeyPress::Enter,
                    KeyPress::Down,
                    KeyPress::Down,
                    KeyPress::Enter,
                ],
            ),
            (
                "config",
                &[
                    KeyPress::Enter,
                    KeyPress::Down,
                    KeyPress::Down,
                    KeyPress::Down,
                    KeyPress::Enter,
                ],
            ),
        ];

        for (name, keys) in screens {
            let app = app_after(keys);
            println!("drawing {name}");
            draw(&app);
        }
    }

    #[test]
    fn the_ask_screen_draws_through_the_whole_exchange() {
        // Walks the states the reply `Option` actually passes through, because
        // each one takes a different branch of the draw code and the compiler
        // cannot tell you which are reachable.
        let mut app = app_after(&[KeyPress::Enter, KeyPress::Enter]);
        draw(&app); // 1. nothing asked — `reply` is None

        app.handle_key(KeyPress::Char(';'));
        app.handle_key(KeyPress::Char('4'));
        app.handle_key(KeyPress::Char('2'));
        draw(&app); // 2. mid-question, decoy on screen, still no reply

        app.handle_key(KeyPress::Enter);
        draw(&app); // 3. ⚠ PONDERING — the spell branch, which was INVERTED
        // and stayed invisible to all 247 tests because none of them drew.
    }

    #[test]
    fn the_ask_screen_draws_a_denial() {
        // The other half of the reply `Option`: a denial fills the same field
        // an answer does, so it must draw through the same branches.
        let app = app_after(&[
            KeyPress::Enter,
            KeyPress::Enter,
            KeyPress::Char('o'),
            KeyPress::Char('i'), // a question with no hidden answer
            KeyPress::Enter,     // → Denied
        ]);

        draw(&app);
    }

    #[test]
    fn every_screen_draws_in_every_language() {
        // The i18n sweep was only ever run-verified by hand (Phase 0). Strings
        // differ in length per language, and length is what breaks layouts — so
        // draw the widest screens under each translation at the tightest size.
        for language_steps in 0..3 {
            let mut keys = vec![
                KeyPress::Enter, // Intro → Menu
                KeyPress::Down,
                KeyPress::Down,
                KeyPress::Down,
                KeyPress::Enter, // → Config
                KeyPress::Down,
                KeyPress::Down,
                KeyPress::Down, // → idioma
            ];
            keys.extend(std::iter::repeat_n(KeyPress::Right, language_steps));
            keys.push(KeyPress::Esc); // → Menu

            let mut app = app_after(&keys);
            draw(&app); // menu in this language

            app.handle_key(KeyPress::Up);
            app.handle_key(KeyPress::Up);
            app.handle_key(KeyPress::Up);
            app.handle_key(KeyPress::Enter); // → Asking
            draw(&app);
        }
    }

    // ── Content assertions — one rung above "it didn't panic" ────────────────
    //
    // ⚠ WHY THESE EXIST. The smoke tests above catch a branch that CRASHES.
    // They cannot catch a branch that renders **nothing** — and that has now
    // happened twice in one refactor: `casting_for` was first inverted (spell
    // drawn after the ponder instead of during) and then fed
    // `speaking_elapsed()`, which is contractually `ZERO` for the whole ponder,
    // so the typewriter revealed zero characters. Both drew a perfectly valid,
    // perfectly empty screen. `draw()` was happy; the séance was not.
    //
    // These assert only that **the words that should be on screen are on
    // screen**. No positions, no styling, no layout — those stay verified by
    // running it, and pinning them would make every visual tweak a failure.
    //
    // Run at the comfortable size only, deliberately: at 80 columns a long
    // Portuguese taunt may wrap mid-phrase and a substring search would go flaky
    // for reasons that have nothing to do with the bug class being hunted.
    // Panic-safety is what gets checked at every size, above.

    /// Draw at 132×41 and flatten the buffer to text, one line per row.
    fn screen_text(app: &App) -> String {
        screen_text_at(app, 132, 41)
    }

    /// The same, at a size you choose.
    ///
    /// ⚠ Worth its own helper because **ratatui clips rather than panicking**: a
    /// box that fits at 132×41 can lose its bottom rows at the 80×24 floor with
    /// no error at all, so `draw()` above stays perfectly green through exactly
    /// that failure. Reading the buffer back at the small size is the only way
    /// to see it.
    fn screen_text_at(app: &App, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("TestBackend must build a terminal");
        terminal
            .draw(|frame| render(frame, app))
            .expect("draw must succeed");

        let buffer = terminal.backend().buffer();
        let row_width = buffer.area.width as usize;
        buffer
            .content()
            .chunks(row_width)
            .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The live reply's words, straight off the app state — so the assertion
    /// compares against what was actually rolled, not a hardcoded guess at a
    /// randomly-picked pool entry.
    fn live_reply_words(app: &App) -> String {
        match app.screen() {
            Screen::Asking(AskingState { reply, .. }) => reply
                .as_ref()
                .expect("expected a live reply")
                .words()
                .to_string(),
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    fn live_spell(app: &App) -> &'static str {
        match app.screen() {
            Screen::Asking(AskingState { spell, .. }) => spell,
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    /// ⚠ Guard against the trap this project keeps re-learning: a draw test that
    /// never reaches the state it claims to cover is decoration. F1 is swallowed
    /// mid-crawl by design, so "I pressed F1" is not the same fact as "the
    /// popover is open" — assert the second one.
    fn assert_popover_is_open(app: &App, when: &str) {
        match app.screen() {
            Screen::Asking(state) => {
                assert!(
                    state.transcript().is_some(),
                    "the popover must be OPEN {when}, or this test draws the very \
                     screen every other test already covers"
                )
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    /// ⚠ THE REGRESSION GUARD FOR MOVING DECORATION OUT OF THE TABLES.
    ///
    /// The glyphs (`▚ ▞`, `⚠`, `▓`, `†`, `⌨`, `⌁`, `▸`) used to live inside the
    /// translated strings and now live in the render. Every other test reaches
    /// them as `translation.x`, so they all move together — meaning **every
    /// glyph could disappear and the whole suite would stay green.** These
    /// assertions are deliberately written against the *composed* result, so
    /// they fail if a render site forgets its decoration.
    #[test]
    fn the_decoration_the_tables_no_longer_carry_still_reaches_the_screen() {
        let expected: [(&str, &[KeyPress], fn(Translation) -> Vec<String>); 4] = [
            ("menu", &[KeyPress::Enter], |t| {
                vec![
                    format!("▚ {} ▞", t.menu.choose_your_destiny),
                    format!("⚠ {}", t.menu.attention),
                ]
            }),
            (
                "info",
                &[KeyPress::Enter, KeyPress::Down, KeyPress::Enter],
                |t| {
                    vec![
                        format!("▚ {} ▞", t.info.title),
                        format!("⌨   {}", t.info.shortcut_title),
                    ]
                },
            ),
            (
                "config",
                &[
                    KeyPress::Enter,
                    KeyPress::Down,
                    KeyPress::Down,
                    KeyPress::Down,
                    KeyPress::Enter,
                ],
                |t| {
                    vec![
                        format!("▓ {} ▓", t.config.configuration),
                        format!("† {} †", t.config.footer),
                    ]
                },
            ),
            ("ask", &[KeyPress::Enter, KeyPress::Enter], |t| {
                // The block titles kept their padding spaces — the borders sit
                // right against them, so a lost space is visible and silent.
                vec![
                    format!(" {} ", t.ask.sued_speak),
                    format!(" {} ", t.ask.talk_with_me),
                ]
            }),
        ];

        for (name, keys, wanted) in expected {
            let app = app_after(keys);
            let text = screen_text(&app);
            for want in wanted(app.config().language().translation()) {
                assert!(
                    text.contains(&want),
                    "the {name} screen must render {want:?} — the glyphs moved \
                     from the translation tables into the render, so a render \
                     site that forgot them looks fine to every other test"
                );
            }
        }
    }

    #[test]
    fn the_recommended_terminal_size_is_substituted_not_printed_raw() {
        // The size is one const shared by three languages, spliced in over a
        // `{size}` placeholder. Two ways that breaks, both asserted: the splice
        // never happens (the marker reaches the screen), or it happens against
        // a stale number.
        let app = app_after(&[KeyPress::Enter, KeyPress::Down, KeyPress::Enter]);
        let text = screen_text(&app);

        assert!(
            text.contains(crate::constants::RECOMMENDED_TERMINAL_SIZE),
            "the info screen must show the recommended terminal size"
        );
        assert!(
            !text.contains("{size}"),
            "the `{{size}}` placeholder reached the screen — a render site is \
             printing the raw translation instead of substituting the const"
        );
    }

    #[test]
    fn the_transcript_popover_draws_at_every_size() {
        // ⚠ Every other test in this module drives the popover CLOSED
        // (`Screen::asking` seeds `history_view: None`), so without this one the
        // entire popover draw path has the same zero coverage the ask screen had
        // before this module existed — and for the same reason: nothing ever
        // called it.
        //
        // Drawn both empty-ish and after a real exchange, because the transcript
        // grows and the rungs still to come (bubbles, `line_count`, the offset
        // arithmetic) all key off how much is in it. At 80×24 the popover's
        // inner height is single digits, which is where that arithmetic
        // underflows if it ever stops saturating.
        let mut app = app_after(&[KeyPress::Enter, KeyPress::Enter, KeyPress::F1]);
        assert_popover_is_open(&app, "on a fresh ask screen");
        draw(&app); // 1. open on a thread holding only the seeded greeting

        app.handle_key(KeyPress::Esc); // close, then hold a real séance
        for key in [
            KeyPress::Char(';'),
            KeyPress::Char('4'),
            KeyPress::Char('2'),
            KeyPress::Enter,
        ] {
            app.handle_key(key);
        }

        // ⚠ Not optional, and the assertion below is what proved it: F1 is
        // swallowed while SueD is still speaking (the G8 lock), so without
        // winding the clock past the crawl this case draws a CLOSED popover and
        // passes for the wrong reason.
        app.rewind_reply(Duration::from_secs(60));

        app.handle_key(KeyPress::F1);
        assert_popover_is_open(&app, "after an exchange");
        draw(&app); // 2. open over a question and an answer
    }

    #[test]
    fn opening_the_transcript_puts_its_frame_on_screen() {
        // One rung above "it didn't panic", and the rung that matters at a step
        // whose whole job is drawing: a popover that renders NOTHING leaves the
        // ask screen looking untouched, F1 looking broken, and every smoke test
        // above perfectly green.
        let app = app_after(&[KeyPress::Enter, KeyPress::Enter, KeyPress::F1]);
        let title = app.config().language().translation().history.title;

        assert!(
            screen_text(&app).contains(title.trim()),
            "F1 must put the transcript's frame on screen; without its title \
             nothing distinguishes an open popover from a broken keybinding"
        );
    }

    #[test]
    fn the_confirm_dialog_draws_whole_at_every_size_and_language() {
        // ⚠ THE GAP THIS CLOSES. Nothing else in this module ever raises the
        // confirm dialog, and `draw()` could not catch the failure that matters
        // here even if it did: ratatui CLIPS. A dialog taller than the band it is
        // centred in loses its bottom rows silently — no panic, no error — so the
        // smoke tests stay green while the buttons vanish and `Esc` looks broken.
        //
        // The dialog's height is DERIVED from `Paragraph::line_count`, and the
        // lore wraps to a different number of rows in each language. So the two
        // ENDS of that arithmetic get asserted: the last word of the prose (did
        // the box reserve enough rows for the wrap?) and the button row beneath
        // it (did the whole box fit the band?). At the 80×24 floor especially —
        // that is where the height has the least room to be wrong in.
        for language_steps in 0..3 {
            let mut keys = vec![
                KeyPress::Enter, // Intro → Menu
                KeyPress::Down,
                KeyPress::Down,
                KeyPress::Down,
                KeyPress::Enter, // → Config
                KeyPress::Down,
                KeyPress::Down,
                KeyPress::Down, // → language
            ];
            keys.extend(std::iter::repeat_n(KeyPress::Right, language_steps));
            keys.extend([
                KeyPress::Esc, // → Menu
                KeyPress::Up,
                KeyPress::Up,
                KeyPress::Up,
                KeyPress::Enter, // → Asking
                // A real question first: `Esc` only raises the dialog once there
                // is a séance to burn. Without these four keys it walks straight
                // back to the Menu and every assertion below reads a screen the
                // dialog was never on.
                KeyPress::Char(';'),
                KeyPress::Char('4'),
                KeyPress::Char('2'),
                KeyPress::Enter,
                KeyPress::Esc,
            ]);

            let app = app_after(&keys);
            let confirm = app.config().language().translation().confirm;

            // The precondition, asserted rather than assumed — the lesson from
            // the two G19 tests that passed for the wrong reason.
            match app.screen() {
                Screen::Asking(state) => assert!(
                    matches!(state.overlay(), Some(Overlay::ConfirmLeave(_))),
                    "the confirm dialog must be OPEN, or this test searches the \
                     ordinary ask screen and can only pass by accident"
                ),
                other => panic!("expected Asking, got {other:?}"),
            }

            // Derived, not hardcoded: each language ends its lore on a different
            // word, and a literal here would silently stop checking the other two.
            let last_word = confirm
                .lore_text
                .split_whitespace()
                .next_back()
                .expect("the lore is never empty");

            for (width, height) in SIZES {
                let screen = screen_text_at(&app, width, height);

                assert!(
                    screen.contains(last_word),
                    "the lore's last word ({last_word:?}) must survive at \
                     {width}x{height} — losing it means the box reserved fewer \
                     rows than the prose actually wrapped to"
                );
                assert!(
                    screen.contains(confirm.leave) && screen.contains(confirm.stay),
                    "both choices must be on screen at {width}x{height} — the \
                     button row is the LAST row of the dialog, so it is the first \
                     thing clipping eats"
                );
            }
        }
    }

    #[test]
    fn the_spell_never_reaches_the_screen_on_a_rebuke() {
        // ⚠ THE SYMPTOM, pinned where it was actually seen — the app-side rule
        // lives in `a_rebuke_lands_instantly_because_nothing_was_consulted`, but
        // what Danilo noticed while playing was the SPELL: SueD announcing he was
        // "leafing through the forbidden books of darkness" before telling you he
        // had not read your question. The incantation says he went looking. On a
        // rebuke he never did.
        //
        // ⚠ `asking_state.spell` stays POPULATED through a rebuke — it is re-picked
        // on every exchange and simply must not be drawn — so this asserts the
        // rendered buffer, not the field. Reading the field would pass on the bug.
        // ⚠ Read at elapsed ≈ 0 — NO rewind, deliberately. The spell only ever
        // draws while `is_pondering()`, so winding the clock forward would hide
        // the bug rather than catch it: the buggy build would have finished its
        // ponder by then and this would pass on both.
        let mut keys = vec![KeyPress::Enter, KeyPress::Enter]; // Intro → Menu → Asking
        keys.extend("hello there".chars().map(KeyPress::Char));
        keys.push(KeyPress::Enter); // → Denied, and short enough to be rebuked

        let app = app_after(&keys);
        let spell = live_spell(&app);
        let rebuke = app.config().language().translation().rebuke;

        // Precondition: a rebuke must genuinely be the live reply. Without one the
        // screen falls to the greeting branch, which draws no spell either — and
        // this test would pass having exercised nothing.
        assert_eq!(
            live_reply_words(&app),
            rebuke.replace("{question}", "hello there"),
            "precondition: the live reply must be the rebuke"
        );

        assert!(
            !screen_text(&app).contains(spell),
            "the spell must not be drawn on a rebuke — it reads as SueD consulting \
             the dark, and a rebuke is him refusing to look at all; got {spell:?}"
        );
    }

    #[test]
    fn a_fresh_ask_screen_greets_you() {
        let app = app_after(&[KeyPress::Enter, KeyPress::Enter]);
        let welcome = app.config().language().translation().ask.welcome_line;

        assert!(
            screen_text(&app).contains(welcome),
            "an ask screen with nothing asked yet must show the welcome line, \
             or the séance opens on a blank box"
        );
    }

    #[test]
    fn the_spell_is_visible_while_sued_ponders() {
        // ⚠ THE REGRESSION TEST FOR THE BUG THAT PROMPTED THIS MODULE.
        // The ponder is 3–6s, so rewinding 1s lands safely mid-ponder.
        //
        // ⚠ AMENDED 2026-08-04 (G18) — THE OLD REASON FOR THE PREFIX IS GONE, AND
        // THE TEST STAYED GREEN THROUGH ITS DISAPPEARANCE. This used to read "a
        // prefix rather than the whole spell precisely because it is still
        // typing": the spell crawled at ~55ms/char, so only ~18 characters
        // existed 1s in. G18 deleted that crawl — the spell is drawn whole on
        // frame one and pulses instead — which means this could now assert the
        // entire string, and the sentence justifying the weaker assertion had
        // quietly become false.
        //
        // The prefix STAYS, but on a different footing, written down here rather
        // than left as inertia: `speak_layout` is a fixed `Constraint::Length(60)`
        // at every terminal size, and the longest spell in any pool is ~41 chars
        // plus dots. A full-string search would go flaky the day a pool entry
        // crosses that box and wraps — which is the same flakiness this module's
        // header already refuses to court, for reasons unrelated to the bug being
        // hunted here.
        let mut app = app_after(&[
            KeyPress::Enter,
            KeyPress::Enter,
            KeyPress::Char(';'),
            KeyPress::Char('x'),
            KeyPress::Enter, // → SueD ponders
        ]);
        app.rewind_reply(Duration::from_millis(1000));

        let spell = live_spell(&app);
        let prefix: String = spell.chars().take(8).collect();

        assert!(
            screen_text(&app).contains(&prefix),
            "SueD must be visibly casting {spell:?} during the ponder — a spell \
             fed the wrong clock renders as an empty box for 3-6 seconds"
        );
    }

    #[test]
    fn the_answer_is_visible_once_sued_speaks() {
        let mut app = app_after(&[
            KeyPress::Enter,
            KeyPress::Enter,
            KeyPress::Char(';'), // → Hidden
            KeyPress::Char('x'),
            KeyPress::Char('y'),
            KeyPress::Char('z'),
            KeyPress::Char('z'),
            KeyPress::Char('y'), // the staged answer, deliberately distinctive
            KeyPress::Enter,
        ]);
        app.rewind_reply(Duration::from_secs(60)); // well past ponder + reveal

        assert!(
            screen_text(&app).contains("xyzzy"),
            "the staged answer must reach the screen — this is the payload of \
             the entire prank"
        );
    }

    #[test]
    fn the_taunt_is_visible_once_sued_refuses() {
        let mut app = app_after(&[
            KeyPress::Enter,
            KeyPress::Enter,
            KeyPress::Char('o'),
            KeyPress::Char('i'), // a question with no hidden answer
            KeyPress::Enter,     // → Denied
        ]);
        app.rewind_reply(Duration::from_secs(60));

        let taunt = live_reply_words(&app);
        let prefix: String = taunt.chars().take(20).collect();

        assert!(
            screen_text(&app).contains(&prefix),
            "the denial must reach the screen too — it travels the same field \
             an answer does, so it must travel the same render path"
        );
    }

    // ── G15 · the question lingers until SueD stops speaking ─────────────────
    //
    // ⚠ WHY THESE ARE DRAW TESTS AND NOT STATE TESTS. G15 changes NOTHING the
    // engine holds — `visible_buffer` still clears at `Enter`, exactly as
    // before, which is what keeps the change trick-safe. The whole behaviour
    // lives in which of two strings `ask.rs` hands to a `Span`. There is no
    // state assertion that can see it; only the buffer can.
    //
    // 📌 That is also the answer to the plan's five-day-old claim that
    // `keystrokes_are_ignored_after_a_denial` would have to invert. It never
    // did: that test asks what the ENGINE holds, and this pair asks what the
    // SCREEN draws. One sentence, two facts.

    /// A question the pools cannot accidentally contain. Every assertion below
    /// is a substring search over the whole 132×41 buffer, so a plausible word
    /// would risk matching a taunt in one of three languages and going green for
    /// the wrong reason.
    const QUESTION: &str = "Hey Sued, the best of all times, do you know who is Amanda?";
    const SECOND_QUESTION: &str = "Lord Sued, the king of the dark, where's Wally?";

    fn type_out(app: &mut App, word: &str) {
        for character in word.chars() {
            app.handle_key(KeyPress::Char(character));
        }
    }

    #[test]
    fn the_question_stays_on_screen_while_sued_is_still_speaking() {
        // The bug from live play: the mark's question vanished the instant
        // `Enter` landed, so the oracle was visibly answering nothing. The clock
        // is deliberately NOT wound here — `app_after` advances no time, so the
        // reply is still pondering and the input is still locked, which is
        // precisely the window G15 exists to fill.
        let mut app = app_after(&[KeyPress::Enter, KeyPress::Enter]);
        type_out(&mut app, QUESTION);
        app.handle_key(KeyPress::Enter); // no hidden answer → Denied

        match app.screen() {
            Screen::Asking(AskingState { reply, .. }) => {
                let reply = reply.as_ref().expect("precondition: SueD is replying");
                assert!(
                    reply.is_pondering(),
                    "precondition: SueD must still be mid-reply — once the crawl \
                     ends the input reopens and this test covers the OLD branch"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }

        assert!(
            screen_text(&app).contains(QUESTION),
            "the question must still be on screen while SueD answers it — the \
             engine has already cleared `visible_buffer`, so the only way it can \
             be there is the transcript's last `Message::User`"
        );
    }

    #[test]
    fn opening_the_transcript_must_not_swap_the_question_underneath_it() {
        // ⚠ THIS IS THE RED ONE — it fails on the shipped code.
        //
        // `input_is_unlocked` answers "may keystrokes reach the engine", and it
        // is false for TWO unrelated reasons: SueD is speaking, OR the popover
        // is open. G15 only wants the first. Hanging the text branch off the
        // union means opening the transcript mid-question silently swaps the
        // input line to the PREVIOUS question — and the input line sits below
        // the popover, so the mark can see it happen.
        //
        // The fix is not a rename: it is that the second condition never got a
        // name of its own. Give it one (`sued_is_speaking`), define
        // `input_is_unlocked` in terms of it, and key the text off the cause
        // rather than the consequence.
        let mut app = app_after(&[KeyPress::Enter, KeyPress::Enter]);
        type_out(&mut app, QUESTION);
        app.handle_key(KeyPress::Enter); // 1st exchange → there is now a history
        app.rewind_reply(Duration::from_secs(60)); // SueD stops; input reopens

        type_out(&mut app, SECOND_QUESTION); // half-typed, NOT submitted
        app.handle_key(KeyPress::F1);
        assert_popover_is_open(&app, "while a second question is half-typed");

        // Deliberately a POSITIVE assertion. `QUESTION` is legitimately on
        // screen — it is a bubble in the open transcript — so asserting its
        // absence would fail for an honest reason. What must be true is that the
        // input line still shows what is actually being typed.
        assert!(
            screen_text(&app).contains(SECOND_QUESTION),
            "the input line must keep showing the live buffer while the \
             transcript is open — the popover is not SueD speaking"
        );
    }
}

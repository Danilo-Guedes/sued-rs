//! ratatui draw code — one submodule per screen; `render` dispatches on `App`.

mod about;
mod ask;
mod common;
mod config;
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
        let backend = TestBackend::new(132, 41);
        let mut terminal = Terminal::new(backend).expect("TestBackend must build a terminal");
        terminal
            .draw(|frame| render(frame, app))
            .expect("draw must succeed");

        let buffer = terminal.backend().buffer();
        let width = buffer.area.width as usize;
        buffer
            .content()
            .chunks(width)
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
            Screen::Asking(state) => assert!(
                state.history_view().is_some(),
                "the popover must be OPEN {when}, or this test draws the very \
                 screen every other test already covers"
            ),
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
        // The ponder is 3–6s and the spell types at ~55ms/char, so 1s in is
        // safely mid-ponder with ~18 characters revealed. A prefix is asserted
        // rather than the whole spell precisely because it is still typing.
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
    const QUESTION: &str = "xyzzy";
    const SECOND_QUESTION: &str = "plugh";

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

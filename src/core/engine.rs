#![allow(dead_code)]

use crate::constants::DECOY_STRING;

#[derive(Debug)]
pub struct Engine {
    mode: Mode,
    answer_buffer: String,      // the real, hidden answer
    visible_buffer: String,     // what the audience sees being "typed"
    decoy_char_list: Vec<char>, // the incantation, char-indexed (UTF-8 safe)
    decoy_cursor: usize,        // how many decoy chars revealed so far
    revealed: Option<String>,   // Some(answer) after Enter
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum Mode {
    #[default]
    Normal,
    Hidden,
}

#[derive(Debug, Clone, Copy)]
pub enum KeyPress {
    Char(char),
    Enter,
    Backspace,
    Esc,
    Up,
    Down,
    F5,
    Left,
    Right,
    CtrlC,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateChange {
    None,
    EnteredHidden,
    ExitedHidden,
    Revealed,
    Denied,
}

impl Engine {
    pub fn new(decoy_str: &str) -> Self {
        Self {
            mode: Mode::default(),
            answer_buffer: String::new(),
            visible_buffer: String::new(),
            decoy_char_list: decoy_str.chars().collect(),
            decoy_cursor: 0,
            revealed: None,
        }
    }
    pub fn handle_key(&mut self, key: KeyPress) -> StateChange {
        match key {
            KeyPress::Char(';') => self.toggle_mode(),
            KeyPress::Char(char) => self.type_char(char),
            KeyPress::Enter => self.handle_enter_key(),
            KeyPress::Backspace => self.handle_backspace_key(),
            KeyPress::F5 => self.handle_f5_key(),
            KeyPress::Esc => StateChange::None,
            KeyPress::Up => StateChange::None,
            KeyPress::Down => StateChange::None,
            KeyPress::Left => StateChange::None,
            KeyPress::Right => StateChange::None,
            KeyPress::CtrlC => StateChange::None,
        }
    }

    fn toggle_mode(&mut self) -> StateChange {
        match self.mode {
            Mode::Hidden => {
                self.switch_mode(Mode::Normal);

                StateChange::ExitedHidden
            }
            Mode::Normal => {
                self.switch_mode(Mode::Hidden);

                StateChange::EnteredHidden
            }
        }
    }

    fn type_char(&mut self, ch: char) -> StateChange {
        match self.mode {
            Mode::Hidden => {
                self.consume_decoy_buffer(ch);
                StateChange::None
            }
            Mode::Normal => {
                self.write_to_visible_buffer(ch);
                StateChange::None
            }
        }
    }

    fn switch_mode(&mut self, new_mode: Mode) {
        self.mode = new_mode
    }

    fn write_to_visible_buffer(&mut self, ch: char) {
        self.visible_buffer.push(ch);
    }

    fn write_to_answer_buffer(&mut self, ch: char) {
        self.answer_buffer.push(ch)
    }

    fn consume_decoy_buffer(&mut self, ch: char) {
        if let Some(valid_decoy_ch) = self.decoy_char_list.get(self.decoy_cursor) {
            self.write_to_visible_buffer(*valid_decoy_ch);
        }
        self.advance_decoy(ch);
    }

    fn advance_decoy(&mut self, ch: char) {
        self.write_to_answer_buffer(ch);
        if self.decoy_cursor < self.decoy_char_list.len() {
            self.decoy_cursor += 1;
        }
    }

    fn move_back_decoy(&mut self) {
        if self.answer_buffer.chars().count() < self.decoy_cursor {
            self.decoy_cursor -= 1;
            self.visible_buffer.pop();
        }
    }

    fn handle_enter_key(&mut self) -> StateChange {
        self.visible_buffer = String::new();
        if self.answer_buffer.is_empty() {
            StateChange::Denied
        } else {
            self.revealed = Some(std::mem::take(&mut self.answer_buffer));
            StateChange::Revealed
        }
    }

    fn handle_backspace_key(&mut self) -> StateChange {
        match self.mode {
            Mode::Hidden => {
                if self.answer_buffer.pop().is_some() {
                    self.move_back_decoy();
                }
            }
            Mode::Normal => {
                self.visible_buffer.pop();
            }
        }
        StateChange::None
    }

    fn handle_f5_key(&mut self) -> StateChange {
        self.reset();
        StateChange::None
    }

    pub fn reset(&mut self) {
        *self = Self::new(DECOY_STRING);
    }

    pub fn visible_buffer(&self) -> &str {
        &self.visible_buffer
    }

    pub fn revealed(&self) -> Option<&str> {
        self.revealed.as_deref()
    }
    pub fn mode(&self) -> Mode {
        self.mode
    }
}

#[cfg(test)]
mod tests {
    use crate::constants::DECOY_STRING;

    use super::*;

    fn build_test_engine() -> Engine {
        Engine::new(DECOY_STRING)
    }

    fn simulate_typing(engine: &mut Engine, typed: &str) {
        for ch in typed.chars() {
            engine.handle_key(KeyPress::Char(ch));
        }
    }

    fn simulate_backspaces(engine: &mut Engine, n: usize) {
        for _ in 0..n {
            engine.handle_key(KeyPress::Backspace);
        }
    }

    #[test]
    fn new_engine_starts_in_normal_mode() {
        let engine = build_test_engine();

        assert_eq!(
            engine.mode,
            Mode::Normal,
            "Engine didn't start as Normal Mode, got={:?}",
            engine.mode
        )
    }

    #[test]
    fn typing_in_normal_mode_appends_to_visible() {
        let mut engine = build_test_engine();

        let typed = String::from("Bom Dia! tudo bem com você?");

        simulate_typing(&mut engine, &typed);

        assert_eq!(engine.visible_buffer, typed)
    }

    #[test]
    fn semicolon_toggles_normal_to_hidden() {
        let mut engine = build_test_engine();

        let change = engine.handle_key(KeyPress::Char(';'));

        assert_eq!(
            engine.mode,
            Mode::Hidden,
            "';' in Normal mode should switch to Hidden"
        );
        assert_eq!(
            change,
            StateChange::EnteredHidden,
            "toggling into Hidden should report EnteredHidden"
        );
        assert!(
            engine.visible_buffer.is_empty(),
            "';' must not be typed into visible_buffer, got={:?}",
            engine.visible_buffer
        );
    }

    #[test]
    fn semicolon_toggles_hidden_back_to_normal() {
        let mut engine = build_test_engine();

        engine.handle_key(KeyPress::Char(';')); // Normal -> Hidden
        let change = engine.handle_key(KeyPress::Char(';')); // Hidden -> Normal

        assert_eq!(
            engine.mode,
            Mode::Normal,
            "a second ';' should switch back to Normal"
        );
        assert_eq!(
            change,
            StateChange::ExitedHidden,
            "toggling out of Hidden should report ExitedHidden"
        );
    }

    #[test]
    fn typing_in_hidden_mode_records_answer_and_advances_decoy() {
        let mut engine = Engine::new("ABCDEFG");

        engine.handle_key(KeyPress::Char(';')); // flip to Hidden — the secret switch
        simulate_typing(&mut engine, "42"); // operator secretly types the real answer

        // The real answer is captured in the HIDDEN buffer...
        assert_eq!(
            engine.answer_buffer, "42",
            "chars typed in Hidden mode must go to answer_buffer, not the screen"
        );
        // ...while the audience sees the decoy "type itself" — one char per keystroke.
        assert_eq!(
            engine.visible_buffer, "AB",
            "each Hidden keystroke should reveal one more decoy char"
        );
        // The cursor tracks how far into the decoy we've revealed.
        assert_eq!(
            engine.decoy_cursor, 2,
            "decoy_cursor should advance once per Hidden keystroke"
        );
    }

    #[test]
    fn decoy_clamps_when_exhausted_but_keeps_recording_answer() {
        // Short decoy so it runs out fast (3 chars).
        let mut engine = Engine::new("ABC");

        engine.handle_key(KeyPress::Char(';')); // Hidden
        simulate_typing(&mut engine, "12345"); // type MORE chars than the decoy has

        // Every real char is still captured — even past the end of the decoy.
        assert_eq!(
            engine.answer_buffer, "12345",
            "answer must keep recording after the decoy is exhausted"
        );
        // The visible decoy freezes at its last char: no wrap, no repeat, no panic.
        assert_eq!(
            engine.visible_buffer, "ABC",
            "visible decoy should stop at its final char once exhausted"
        );
        // Cursor clamps at decoy.len() and never indexes out of bounds.
        assert_eq!(
            engine.decoy_cursor, 3,
            "decoy_cursor should clamp at decoy.len()"
        );
    }

    #[test]
    fn enter_reveals_the_buffered_answer() {
        let mut engine = Engine::new("ABCDEFG");

        engine.handle_key(KeyPress::Char(';')); // Hidden
        simulate_typing(&mut engine, "42"); // secret answer
        engine.handle_key(KeyPress::Char(';')); // back to Normal

        let change = engine.handle_key(KeyPress::Enter); // the reveal

        assert_eq!(
            engine.revealed,
            Some("42".to_string()),
            "Enter should move the hidden answer into `revealed`"
        );
        assert_eq!(
            change,
            StateChange::Revealed,
            "Enter on a non-empty answer should report Revealed"
        );
    }

    #[test]
    fn enter_with_empty_answer_is_denied() {
        let mut engine = build_test_engine();

        // Operator hits Enter without ever composing an answer.
        let change = engine.handle_key(KeyPress::Enter);

        // A denial never populates the answer box — nothing is revealed...
        assert_eq!(
            engine.revealed, None,
            "an empty answer must not put anything in `revealed`"
        );
        // ...but SUED still *reacts*: it emits a denial the UI can turn into a
        // taunt, instead of the old silent `None`.
        assert_eq!(
            change,
            StateChange::Denied,
            "Enter on an empty answer should report Denied, not stay silent"
        );
    }

    #[test]
    fn enter_after_typing_a_question_in_normal_mode_is_denied() {
        // The fail-safe: someone who doesn't know the ';' trick just types their
        // question in the open and hits Enter. Normal-mode chars go to the
        // *visible* buffer, so `answer_buffer` is still empty — SUED denies them.
        let mut engine = build_test_engine();

        simulate_typing(&mut engine, "quem é o melhor dev rust?");
        let change = engine.handle_key(KeyPress::Enter);

        assert_eq!(
            change,
            StateChange::Denied,
            "a question typed in the open (empty answer_buffer) earns a denial"
        );
        assert_eq!(
            engine.revealed, None,
            "denial reveals nothing — there was no hidden answer"
        );
        // AMENDED for the conversational flow (G8): this used to pin the
        // opposite ("the denial must not erase what the mortal typed") — right
        // for the old ask-once-freeze-until-F5 world, wrong now. A reply of
        // either kind consumes the offering; the input must read empty while
        // SueD taunts, ready to blink for the next question.
        assert_eq!(
            engine.visible_buffer, "",
            "the denial consumes the question — the input reads empty while SueD taunts"
        );
    }

    #[test]
    fn a_reveal_consumes_the_question_from_the_visible_buffer() {
        // The reveal-side twin: the moment SueD accepts the offering, the
        // question vanishes into the oracle — including the decoy remnants
        // painted during hidden typing. The audience sees an empty, waiting
        // prompt under the crawling reply, never the stale question.
        let mut engine = build_test_engine();

        engine.handle_key(KeyPress::Char(';')); // Hidden — decoy chars go visible
        simulate_typing(&mut engine, "42");
        engine.handle_key(KeyPress::Enter); // reveal

        assert_eq!(
            engine.visible_buffer, "",
            "the reveal must consume the visible question, decoy remnants included"
        );
        assert_eq!(
            engine.revealed(),
            Some("42"),
            "consuming the question must not touch the reply itself"
        );
    }

    #[test]
    fn backspace_in_normal_mode_removes_last_visible_char() {
        let mut engine = build_test_engine();

        simulate_typing(&mut engine, "abc");
        let change = engine.handle_key(KeyPress::Backspace);

        assert_eq!(
            engine.visible_buffer, "ab",
            "Backspace in Normal mode should delete the last visible char"
        );
        assert_eq!(
            change,
            StateChange::None,
            "Backspace is plain editing — it should report no state change"
        );
    }

    #[test]
    fn backspace_on_empty_buffer_is_a_noop() {
        let mut engine = build_test_engine();

        // Nothing typed yet — Backspace must not panic or underflow.
        let change = engine.handle_key(KeyPress::Backspace);

        assert_eq!(
            engine.visible_buffer, "",
            "Backspace on an empty buffer should leave it empty"
        );
        assert_eq!(change, StateChange::None);
    }

    #[test]
    fn backspace_in_hidden_mode_retracts_answer_and_decoy() {
        let mut engine = Engine::new("ABCDEFG");

        engine.handle_key(KeyPress::Char(';')); // Hidden
        simulate_typing(&mut engine, "42"); // answer "42", visible "AB", cursor 2

        engine.handle_key(KeyPress::Backspace); // un-type one secret keystroke

        // The real answer loses its last char...
        assert_eq!(
            engine.answer_buffer, "4",
            "Backspace in Hidden mode should pop the last real answer char"
        );
        // ...and the decoy visibly retreats by one, so the illusion stays consistent.
        assert_eq!(
            engine.visible_buffer, "A",
            "Backspace in Hidden mode should retract one revealed decoy char"
        );
        assert_eq!(
            engine.decoy_cursor, 1,
            "decoy_cursor should step back by one on Hidden Backspace"
        );
    }

    #[test]
    fn backspace_in_hidden_mode_with_no_answer_is_a_noop() {
        let mut engine = Engine::new("ABC");

        engine.handle_key(KeyPress::Char(';')); // Hidden, but nothing typed yet
        let change = engine.handle_key(KeyPress::Backspace);

        assert_eq!(
            engine.answer_buffer, "",
            "nothing to retract → answer stays empty"
        );
        assert_eq!(
            engine.visible_buffer, "",
            "nothing revealed → visible stays empty"
        );
        assert_eq!(engine.decoy_cursor, 0, "cursor must not underflow below 0");
        assert_eq!(change, StateChange::None);
    }

    #[test]
    fn backspace_past_exhausted_decoy_pops_answer_but_keeps_decoy_frozen() {
        let mut engine = Engine::new("ABC");

        engine.handle_key(KeyPress::Char(';')); // Hidden
        simulate_typing(&mut engine, "12345"); // answer "12345", visible "ABC", cursor 3

        simulate_backspaces(&mut engine, 2); // retract the two "silent" chars

        assert_eq!(
            engine.answer_buffer, "123",
            "Backspace should remove the extra answer chars typed past the decoy"
        );
        assert_eq!(
            engine.visible_buffer, "ABC",
            "the exhausted decoy stays frozen while we're still past its end"
        );
        assert_eq!(
            engine.decoy_cursor, 3,
            "cursor stays clamped until we re-enter the decoy region"
        );

        // One more Backspace crosses back into the decoy and DOES retract it.
        engine.handle_key(KeyPress::Backspace);

        assert_eq!(engine.answer_buffer, "12");
        assert_eq!(
            engine.visible_buffer, "AB",
            "once back inside the decoy, Backspace retracts a visible char again"
        );
        assert_eq!(engine.decoy_cursor, 2);
    }

    #[test]
    fn hidden_backspace_retracts_decoy_not_the_normal_typed_prefix() {
        let mut engine = Engine::new("ABC");

        simulate_typing(&mut engine, "go"); // Normal: visible "go"
        engine.handle_key(KeyPress::Char(';')); // Hidden
        simulate_typing(&mut engine, "4"); // answer "4", visible "goA", cursor 1

        engine.handle_key(KeyPress::Backspace);

        assert_eq!(engine.answer_buffer, "", "the one secret char is removed");
        assert_eq!(
            engine.visible_buffer, "go",
            "Backspace removes the revealed decoy char, leaving the Normal-typed text intact"
        );
        assert_eq!(engine.decoy_cursor, 0);
    }

    #[test]
    fn f5_resets_the_engine_to_a_fresh_state() {
        let mut engine = build_test_engine();

        // Dirty every piece of state we can hold at once: Normal typing, then
        // Hidden mode with a secret answer, then reveal it.
        simulate_typing(&mut engine, "oi"); // Normal: visible "oi"
        engine.handle_key(KeyPress::Char(';')); // → Hidden
        simulate_typing(&mut engine, "42"); // secret answer + decoy advances
        engine.handle_key(KeyPress::Enter); // reveal — consumes the visible question (G8)
        simulate_typing(&mut engine, "x"); // re-dirty the visible buffer post-reveal (paints a decoy char)

        // Precondition: prove the engine really is dirty before we reset it,
        // otherwise a no-op reset would pass this test for the wrong reason.
        assert_eq!(engine.mode, Mode::Hidden);
        assert!(engine.revealed.is_some());
        assert!(!engine.visible_buffer.is_empty());
        assert_ne!(engine.decoy_cursor, 0);

        let change = engine.handle_key(KeyPress::F5);

        assert_eq!(
            change,
            StateChange::None,
            "F5 is a silent reset — it reports no transition to the UI/audio layer"
        );
        assert_eq!(engine.mode, Mode::Normal, "mode returns to Normal");
        assert_eq!(engine.revealed, None, "the revealed answer is cleared");
        assert_eq!(engine.visible_buffer, "", "the visible buffer is cleared");
        assert_eq!(engine.decoy_cursor, 0, "the decoy rewinds to the start");
    }
}

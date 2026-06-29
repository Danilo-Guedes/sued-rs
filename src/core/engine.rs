//! THE PRANK CORE — **you implement this at M1.**
//!
//! This is intentionally empty. It's yours to write: the `Mode`/`AppState` enums,
//! the `Engine` struct, the pure `handle_key` transition function, and the decoy
//! mechanic. We'll agree the public API together first, then Claude writes the
//! failing tests (the spec) and you make them green.
//!
//! References:
//! - `../../plan/PLAN.md` §D (M1) and §E (open design questions)
//! - `../../plan/sued-rs-brief.md` §4 and §9 (acceptance criteria)
//
// TEMP(M1): the engine isn't wired into main.rs yet, so the binary sees its
// public API as "dead". Remove this once the M1 plain-terminal loop drives it.
#![allow(dead_code)]

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

#[derive(Debug)]
pub enum Key {
    Char(char),
    Enter,
    Backspace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateChange {
    None,
    EnteredHidden,
    ExitedHidden,
    Revealed,
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
    pub fn handle_key(&mut self, key: Key) -> StateChange {
        match key {
            Key::Char(';') => self.toggle_mode(),
            Key::Char(c) => self.type_char(c),
            Key::Enter => self.handle_enter_key(),
            Key::Backspace => todo!(),
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
            Mode::Normal => {
                self.write_to_visible_buffer(ch);
                StateChange::None
            }
            Mode::Hidden => {
                self.consume_decoy_buffer(ch);
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

    fn handle_enter_key(&mut self) -> StateChange {
        if self.answer_buffer.is_empty() {
            StateChange::None
        } else {
            self.revealed = Some(std::mem::take(&mut self.answer_buffer));
            StateChange::Revealed
        }
    }
}

// simple in the beggining, after we want to do a multy language setup
const DECOY_STRING: &str = "Sued, grande poderoso";

#[cfg(test)]
mod tests {
    // TODO(M1): write the spec tests here.

    use super::*;

    fn build_test_engine() -> Engine {
        Engine::new(DECOY_STRING)
    }

    fn simulate_typing(engine: &mut Engine, typed: &str) {
        for ch in typed.chars() {
            engine.handle_key(Key::Char(ch));
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

        // ';' is the secret switch, NOT text — it must flip the mode and
        // must not land in any buffer.
        let change = engine.handle_key(Key::Char(';'));

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

        engine.handle_key(Key::Char(';')); // Normal -> Hidden
        let change = engine.handle_key(Key::Char(';')); // Hidden -> Normal

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
        // Explicit, easy-to-read decoy so the expected output is obvious.
        let mut engine = Engine::new("ABCDEFG");

        engine.handle_key(Key::Char(';')); // flip to Hidden — the secret switch
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

        engine.handle_key(Key::Char(';')); // Hidden
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

        engine.handle_key(Key::Char(';')); // Hidden
        simulate_typing(&mut engine, "42"); // secret answer
        engine.handle_key(Key::Char(';')); // back to Normal

        let change = engine.handle_key(Key::Enter); // the reveal

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
    fn enter_with_empty_answer_is_a_noop() {
        let mut engine = build_test_engine();

        // Operator hits Enter without ever composing an answer.
        let change = engine.handle_key(Key::Enter);

        assert_eq!(
            engine.revealed, None,
            "revealing an empty answer should do nothing"
        );
        assert_eq!(
            change,
            StateChange::None,
            "Enter on an empty answer should report None"
        );
    }
}

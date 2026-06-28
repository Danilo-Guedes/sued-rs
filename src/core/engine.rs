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
// TODO(M1): implement the Engine here.

#[derive(Debug)]
pub struct Engine {
    mode: Mode,
    answer_buffer: String,    // the real, hidden answer
    visible_buffer: String,   // what the audience sees being "typed"
    decoy: Vec<char>,         // the incantation, char-indexed (UTF-8 safe)
    decoy_cursor: usize,      // how many decoy chars revealed so far
    revealed: Option<String>, // Some(answer) after Enter
}

#[derive(Debug, Default, PartialEq)]
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
}

impl Engine {
    pub fn new(decoy_str: &str) -> Self {
        Self {
            mode: Mode::default(),
            answer_buffer: String::new(),
            visible_buffer: String::new(),
            decoy: decoy_str.chars().collect(),
            decoy_cursor: 0,
            revealed: None,
        }
    }
    pub fn handle_key(&mut self, key: Key) -> StateChange {
        match key {
            Key::Char(typed_char) => {
                self.visible_buffer.push(typed_char);
            }
            Key::Enter => {
                todo!()
            }
            Key::Backspace => {
                todo!()
            }
        };

        StateChange::None
    }
}

// simple in the beggining, after we want to do a multy language setup
const DECOY_STRING: &str = "Sued, grande";

#[cfg(test)]
mod tests {
    // TODO(M1): write the spec tests here.

    use super::*;

    fn build_test_engine() -> Engine {
        Engine::new(DECOY_STRING)
    }

    fn simulate_typing(engine: &mut Engine, typed: &str) -> () {
        for ch in typed.chars() {
            engine.handle_key(Key::Char(ch));
        }
    }

    #[test]
    fn new_engine_starts_in_normal_mode() {
        let engine = build_test_engine();

        dbg!(&engine);

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
}

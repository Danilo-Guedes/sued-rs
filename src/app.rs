//! Top-level application state machine (M2).
//!
//! `AppState` is the **app shell**: which screen are we on, and how do keys move
//! us between screens. The pure prank lives in [`crate::core::engine`] and stays
//! untouched — when we're on the question screen, `AppState::AwaitingQuestion`
//! simply *owns* one `Engine` and forwards keys to it.
//!

#![allow(dead_code, unused_variables)]

use crate::core::engine::{DECOY_STRING, Engine, KeyPress};

#[derive(Default, Debug)]
pub enum AppState {
    #[default]
    Intro,
    Menu(MenuState),
    AwaitingQuestion(Engine),
    Info,
    About,
}

#[derive(Debug, PartialEq)]
pub enum AppFlow {
    Stay,
    Quit,
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum MenuItem {
    #[default]
    Ask,
    Info,
    About,
    Exit,
}

#[derive(Debug, Default)]
pub struct MenuState {
    menu_index: usize,
}

impl MenuState {
    pub const ALL: [MenuItem; 4] = [
        MenuItem::Ask,
        MenuItem::Info,
        MenuItem::About,
        MenuItem::Exit,
    ];
}

impl AppState {
    pub fn new() -> Self {
        AppState::default()
    }
    pub fn handle_key(&mut self, key: KeyPress) -> AppFlow {
        match self {
            AppState::Intro => match key {
                KeyPress::Enter => {
                    *self = AppState::Menu(MenuState::default());
                    AppFlow::Stay
                }
                KeyPress::Esc => AppFlow::Quit,
                _ => AppFlow::Stay,
            },
            AppState::Menu(menu_state) => match key {
                KeyPress::Enter => match menu_state.menu_index() {
                    0 => {
                        *self = AppState::AwaitingQuestion(Engine::new(DECOY_STRING));
                        AppFlow::Stay
                    }
                    1 => {
                        *self = AppState::Info;
                        AppFlow::Stay
                    }
                    2 => {
                        *self = AppState::About;
                        AppFlow::Stay
                    }
                    3 => AppFlow::Quit,
                    _ => AppFlow::Stay,
                },
                KeyPress::Esc => AppFlow::Quit,
                KeyPress::Up => {
                    menu_state.move_menu_up();
                    AppFlow::Stay
                }
                KeyPress::Down => {
                    menu_state.move_menu_down();
                    AppFlow::Stay
                }
                _ => AppFlow::Stay,
            },
            AppState::AwaitingQuestion(engine) => match key {
                KeyPress::Enter => {
                    engine.handle_key(KeyPress::Enter);
                    AppFlow::Stay
                }
                KeyPress::Esc => {
                    *self = AppState::Menu(MenuState::default());
                    AppFlow::Stay
                }
                KeyPress::Backspace => {
                    engine.handle_key(KeyPress::Backspace);
                    AppFlow::Stay
                }
                other_char => {
                    engine.handle_key(other_char);
                    AppFlow::Stay
                }
            },
            AppState::Info => match key {
                KeyPress::Esc => {
                    *self = AppState::Menu(MenuState::default());
                    AppFlow::Stay
                }
                _ => AppFlow::Stay,
            },
            AppState::About => match key {
                KeyPress::Esc => {
                    *self = AppState::Menu(MenuState::new());
                    AppFlow::Stay
                }
                _ => AppFlow::Stay,
            },
        }
    }
}

impl MenuState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn move_menu_down(&mut self) {
        let curr_index = self.menu_index();
        let menu_size = Self::ALL.len();
        let new_index = (curr_index + 1) % menu_size;
        self.menu_index = new_index;
    }

    pub fn move_menu_up(&mut self) {
        let curr_index = self.menu_index();
        let menu_size = Self::ALL.len();
        let new_index = (curr_index + menu_size - 1) % menu_size;
        self.menu_index = new_index;
    }

    pub fn menu_index(&self) -> usize {
        self.menu_index
    }
}

#[cfg(test)]
mod tests {
    use super::MenuState;
    use super::*;
    use crate::core::engine::KeyPress;

    /// Replay a sequence of keystrokes from the intro, handing back the final
    /// state *and* the `AppFlow` returned by the last key (Stay/Quit).
    fn drive_flow(keys: &[KeyPress]) -> (AppState, AppFlow) {
        let mut state = AppState::new();
        let mut flow = AppFlow::Stay;
        for &key in keys {
            flow = state.handle_key(key);
        }
        (state, flow)
    }

    /// Same, when the test only cares about where we landed (a thin wrapper —
    /// one source of truth for the loop, clean call sites for the common case).
    fn drive(keys: &[KeyPress]) -> AppState {
        drive_flow(keys).0
    }

    /// The highlighted menu item, or panic if we're not on the menu.
    fn selected(state: &AppState) -> MenuItem {
        match state {
            AppState::Menu(menu) => MenuState::ALL[menu.menu_index()],
            other => panic!("expected Menu, got {other:?}"),
        }
    }

    // ── Intro ────────────────────────────────────────────────────────────────

    #[test]
    fn new_starts_at_intro() {
        assert!(matches!(AppState::new(), AppState::Intro));
    }

    #[test]
    fn intro_enter_opens_menu_on_first_item() {
        let state = drive(&[KeyPress::Enter]);
        assert!(matches!(state, AppState::Menu(_)));
        assert_eq!(selected(&state), MenuItem::Ask);
    }

    #[test]
    fn intro_esc_quits() {
        let (_state, flow) = drive_flow(&[KeyPress::Esc]);
        assert_eq!(flow, AppFlow::Quit);
    }

    // ── Menu navigation (wraps) ──────────────────────────────────────────────

    #[test]
    fn menu_down_advances_selection() {
        let state = drive(&[KeyPress::Enter, KeyPress::Down]);
        assert_eq!(selected(&state), MenuItem::Info);
    }

    #[test]
    fn menu_down_wraps_past_last_item() {
        // Perguntar → Informacoes → Sobre → Sair → back to Perguntar.
        let state = drive(&[
            KeyPress::Enter,
            KeyPress::Down,
            KeyPress::Down,
            KeyPress::Down,
            KeyPress::Down,
        ]);
        assert_eq!(selected(&state), MenuItem::Ask);
    }

    #[test]
    fn menu_up_wraps_to_last_item() {
        // From the first item, Up lands on Sair.
        let state = drive(&[KeyPress::Enter, KeyPress::Up]);
        assert_eq!(selected(&state), MenuItem::Exit);
    }

    // ── Menu selection (Enter routes per item) ───────────────────────────────

    #[test]
    fn menu_enter_on_perguntar_opens_a_fresh_question() {
        let (state, flow) = drive_flow(&[KeyPress::Enter, KeyPress::Enter]);
        assert_eq!(flow, AppFlow::Stay);
        match state {
            // A brand-new prank session: nothing typed, nothing on screen yet.
            AppState::AwaitingQuestion(engine) => assert_eq!(engine.visible_buffer(), ""),
            other => panic!("expected AwaitingQuestion, got {other:?}"),
        }
    }

    #[test]
    fn menu_enter_on_informacoes_opens_info() {
        let state = drive(&[KeyPress::Enter, KeyPress::Down, KeyPress::Enter]);
        assert!(matches!(state, AppState::Info));
    }

    #[test]
    fn menu_enter_on_sobre_opens_about() {
        let state = drive(&[
            KeyPress::Enter,
            KeyPress::Down,
            KeyPress::Down,
            KeyPress::Enter,
        ]);
        assert!(matches!(state, AppState::About));
    }

    #[test]
    fn menu_enter_on_sair_quits() {
        // Up from the first item wraps to Sair; Enter there quits.
        let (_state, flow) = drive_flow(&[KeyPress::Enter, KeyPress::Up, KeyPress::Enter]);
        assert_eq!(flow, AppFlow::Quit);
    }

    #[test]
    fn menu_esc_quits() {
        let (_state, flow) = drive_flow(&[KeyPress::Enter, KeyPress::Esc]);
        assert_eq!(flow, AppFlow::Quit);
    }

    // ── Question screen forwards to the engine ───────────────────────────────

    #[test]
    fn question_typing_reaches_the_engine() {
        // Open the question screen, then type two chars in Normal mode.
        let state = drive(&[
            KeyPress::Enter,
            KeyPress::Enter,
            KeyPress::Char('o'),
            KeyPress::Char('i'),
        ]);
        match state {
            AppState::AwaitingQuestion(engine) => assert_eq!(engine.visible_buffer(), "oi"),
            other => panic!("expected AwaitingQuestion, got {other:?}"),
        }
    }

    #[test]
    fn question_esc_returns_to_menu() {
        let state = drive(&[KeyPress::Enter, KeyPress::Enter, KeyPress::Esc]);
        assert!(matches!(state, AppState::Menu(_)));
    }

    // ── Static screens bounce back to the menu ───────────────────────────────

    #[test]
    fn info_esc_returns_to_menu() {
        let state = drive(&[
            KeyPress::Enter,
            KeyPress::Down,
            KeyPress::Enter,
            KeyPress::Esc,
        ]);
        assert!(matches!(state, AppState::Menu(_)));
    }

    #[test]
    fn about_esc_returns_to_menu() {
        let state = drive(&[
            KeyPress::Enter,
            KeyPress::Down,
            KeyPress::Down,
            KeyPress::Enter,
            KeyPress::Esc,
        ]);
        assert!(matches!(state, AppState::Menu(_)));
    }
}

//! Top-level application state machine (M2).
//!
//! `AppState` is the **app shell**: which screen are we on, and how do keys move
//! us between screens. The pure prank lives in [`crate::core::engine`] and stays
//! untouched — when we're on the question screen, `AppState::AwaitingQuestion`
//! simply *owns* one `Engine` and forwards keys to it.
//!
//! ## Contract for this file
//! The `#[cfg(test)] mod tests` block below is the **executable spec** (Claude's
//! half). Everything *above* it — the types and `handle_key` logic — is **yours**
//! to write until the suite goes green. Reshape the types if you want a cleaner
//! shape; just keep the names the tests reference.
//!
//!
//! ## Your green-path steps (red → green)
//! 1. `mod app;` in `main.rs` so this file (and `cargo test`) actually sees it.
//! 2. Grow `Key` in `core/engine.rs`: add `Esc`, `Up`, `Down`, and
//!    `#[derive(Clone, Copy)]` (the test helper presses keys by value). The engine
//!    doesn't care about those three — give it no-op match arms (`=> StateChange::None`).
//!    *This is the shared-`Key` call (#3a): one input alphabet for the whole app;
//!    the engine just ignores the keys it has no use for.*
//! 3. Write the types + `handle_key` here until the suite is green.
//!
//! Transitions the suite pins (read the tests as the source of truth):
//! - `Intro`:  Enter → Menu · Esc → Quit
//! - `Menu`:   Up/Down move the selection (**wraps** around the 4 items) ·
//!             Enter routes per selected item (Perguntar → a *fresh* `AwaitingQuestion`,
//!             Informações → Info, Sobre → About, Sair → Quit) · Esc → Quit
//! - `AwaitingQuestion(Engine)`: typing keys forward to the engine · Esc → back to Menu
//! - `Info` / `About`: Esc → back to Menu
//!
//!
//!

use crate::core::engine::{Engine, KeyPress};

#[derive(Default, Debug)]
pub enum AppState {
    #[default]
    Intro,
    Menu(MenuState),
    AwaitingQuestion(Engine),
    Info,
    About,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MenuItem {
    Ask,
    Info,
    About,
    Exit,
}

#[derive(Debug, PartialEq)]
pub enum AppFlow {
    Stay,
    Quit,
}

#[derive(Debug)]
pub struct MenuState {/* your call — selected index, etc. */}

impl AppState {
    pub fn new() -> Self {
        AppState::default()
    }
    pub fn handle_key(&mut self, key: KeyPress) -> AppFlow {
        AppFlow::Stay
    }
}
impl MenuState {
    pub fn selected_item(&self) -> MenuItem {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::engine::KeyPress;

    /// Start at the intro and replay a sequence of keystrokes.
    fn drive(keys: &[KeyPress]) -> AppState {
        let mut state = AppState::new();
        for &key in keys {
            state.handle_key(key);
        }
        state
    }

    /// Same, but hand back the `Flow` returned by the *last* key (Stay/Quit).
    fn drive_flow(keys: &[KeyPress]) -> (AppState, AppFlow) {
        let mut state = AppState::new();
        let mut flow = AppFlow::Stay;
        for &key in keys {
            flow = state.handle_key(key);
        }
        (state, flow)
    }

    /// The highlighted menu item, or panic if we're not on the menu.
    fn selected(state: &AppState) -> MenuItem {
        match state {
            AppState::Menu(menu) => menu.selected_item(),
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
            AppState::AwaitingQuestion(engine) => assert_eq!(engine.get_visible_buffer(), ""),
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
            AppState::AwaitingQuestion(engine) => assert_eq!(engine.get_visible_buffer(), "oi"),
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

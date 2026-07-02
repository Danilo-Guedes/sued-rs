//! Top-level application state machine (M2).
//!
//! `AppState` is the **app shell**: which screen are we on, and how do keys move
//! us between screens. The pure prank lives in [`crate::core::engine`] and stays
//! untouched — when we're on the question screen, `AppState::Asking`
//! simply *owns* one `Engine` and forwards keys to it.
//!

#![allow(unused_variables)]

use crate::core::engine::{DECOY_STRING, Engine, KeyPress};

#[derive(Default, Debug)]
pub enum AppState {
    #[default]
    Intro,
    Menu(MenuState),
    Asking(Engine),
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

impl MenuItem {
    pub fn label(&self) -> &'static str {
        match self {
            MenuItem::Ask => "PERGUNTAR AO ORÁCULO",
            MenuItem::Info => "INFORMAÇÃO",
            MenuItem::About => "SOBRE O SUED",
            MenuItem::Exit => "SAIR",
        }
    }
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
                        *self = AppState::Asking(Engine::new(DECOY_STRING));
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
            AppState::Asking(engine) => match key {
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
        let menu_size = Self::ALL.len();
        self.menu_index = (self.menu_index + 1) % menu_size;
    }

    pub fn move_menu_up(&mut self) {
        let menu_size = Self::ALL.len();
        self.menu_index = (self.menu_index + menu_size - 1) % menu_size;
    }

    pub fn menu_index(&self) -> usize {
        self.menu_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::engine::KeyPress;

    // ── The API these tests pin (option C: MenuState embedded as a field) ─────
    // AppState  = struct { screen: AppScreen, menu_state: MenuState }
    // AppScreen = enum { Intro, Menu, Asking(Engine), Info, About }   (Menu is UNIT)
    // AppState::screen(&self)     -> &AppScreen
    // AppState::menu_state(&self) -> &MenuState
    // MenuState keeps ALL, menu_index(), move_menu_up/down() exactly as today —
    //   it just moves from *inside* the Menu variant to a sibling field, so the
    //   cursor persists across screen changes.
    //
    // If you keep `AppScreen::Menu(MenuState)` (option B), change the one
    // `on_menu` helper below to `matches!(state.screen(), AppScreen::Menu(_))`.

    /// Replay a sequence of keystrokes from a fresh app, handing back the final
    /// state *and* the `AppFlow` returned by the last key (Stay/Quit).
    fn drive_flow(keys: &[KeyPress]) -> (AppState, AppFlow) {
        let mut state = AppState::new();
        let mut flow = AppFlow::Stay;
        for &key in keys {
            flow = state.handle_key(key);
        }
        (state, flow)
    }

    /// Same, when the test only cares about where we landed.
    fn drive(keys: &[KeyPress]) -> AppState {
        drive_flow(keys).0
    }

    /// The currently highlighted menu item. The menu cursor lives on `AppState`
    /// now (inside `menu_state`), so this is *always* valid — assert `on_menu`
    /// separately when the current *screen* is what matters.
    fn selected(state: &AppState) -> MenuItem {
        MenuState::ALL[state.menu_state().menu_index()]
    }

    /// Are we on the menu screen? (Single point to tweak for option B.)
    fn on_menu(state: &AppState) -> bool {
        matches!(state.screen(), AppScreen::Menu)
    }

    // ── Intro ────────────────────────────────────────────────────────────────

    #[test]
    fn new_starts_at_intro() {
        assert!(matches!(AppState::new().screen(), AppScreen::Intro));
    }

    #[test]
    fn intro_enter_opens_menu_on_first_item() {
        let state = drive(&[KeyPress::Enter]);
        assert!(on_menu(&state));
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
        match state.screen() {
            // A brand-new prank session: nothing typed, nothing on screen yet.
            AppScreen::Asking(engine) => assert_eq!(engine.visible_buffer(), ""),
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn menu_enter_on_informacoes_opens_info() {
        let state = drive(&[KeyPress::Enter, KeyPress::Down, KeyPress::Enter]);
        assert!(matches!(state.screen(), AppScreen::Info));
    }

    #[test]
    fn menu_enter_on_sobre_opens_about() {
        let state = drive(&[
            KeyPress::Enter,
            KeyPress::Down,
            KeyPress::Down,
            KeyPress::Enter,
        ]);
        assert!(matches!(state.screen(), AppScreen::About));
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
        match state.screen() {
            AppScreen::Asking(engine) => assert_eq!(engine.visible_buffer(), "oi"),
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn question_esc_returns_to_menu() {
        let state = drive(&[KeyPress::Enter, KeyPress::Enter, KeyPress::Esc]);
        assert!(on_menu(&state));
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
        assert!(on_menu(&state));
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
        assert!(on_menu(&state));
    }

    // ── NEW: menu selection PERSISTS across a sub-screen visit ────────────────
    // The whole point of hoisting `menu_index` to the app struct: the cursor is
    // app-level state, so leaving the menu and returning must NOT reset it to 0.
    // These are the tests that go red on the *behaviour* (not just the types).

    #[test]
    fn info_esc_preserves_menu_selection() {
        // Menu → Down (Info, idx 1) → Enter (into Info) → Esc (back to Menu).
        let state = drive(&[
            KeyPress::Enter,
            KeyPress::Down,
            KeyPress::Enter,
            KeyPress::Esc,
        ]);
        assert!(on_menu(&state));
        assert_eq!(
            selected(&state),
            MenuItem::Info,
            "returning from Info must keep the cursor on Info, not reset to Ask"
        );
    }

    #[test]
    fn about_esc_preserves_menu_selection() {
        // Menu → Down, Down (Sobre, idx 2) → Enter → Esc.
        let state = drive(&[
            KeyPress::Enter,
            KeyPress::Down,
            KeyPress::Down,
            KeyPress::Enter,
            KeyPress::Esc,
        ]);
        assert!(on_menu(&state));
        assert_eq!(selected(&state), MenuItem::About);
    }

    #[test]
    fn question_esc_preserves_menu_selection() {
        // Ask is index 0, so this "worked" by coincidence with default() — pin it
        // so a future menu reorder can't silently break the round-trip.
        let state = drive(&[KeyPress::Enter, KeyPress::Enter, KeyPress::Esc]);
        assert!(on_menu(&state));
        assert_eq!(selected(&state), MenuItem::Ask);
    }

    #[test]
    fn restored_selection_is_a_live_cursor_not_a_frozen_value() {
        // Return from Sobre (idx 2), then Down must advance to Sair (idx 3) —
        // proving the restored index is the real, still-navigable cursor.
        let state = drive(&[
            KeyPress::Enter,
            KeyPress::Down,
            KeyPress::Down, // Sobre (2)
            KeyPress::Enter, // into Sobre
            KeyPress::Esc,   // back to Menu, still at 2
            KeyPress::Down,  // → Sair (3)
        ]);
        assert!(on_menu(&state));
        assert_eq!(selected(&state), MenuItem::Exit);
    }
}

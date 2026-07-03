//! Top-level application state machine (M2).
//!
//! [`App`] is the **app shell** — a struct pairing the current [`Screen`] with the
//! menu cursor ([`Menu`]), so the selection survives moving between screens. The pure
//! prank lives in [`crate::core::engine`] and stays untouched — on the question
//! screen, `Screen::Asking` simply *owns* one `Engine` and forwards keys to it.
//!

#![allow(unused_variables)]

use crate::core::engine::{DECOY_STRING, Engine, KeyPress};

#[derive(Default, Debug)]
pub struct App {
    screen: Screen,
    menu: Menu,
}

#[derive(Default, Debug)]
pub enum Screen {
    #[default]
    Intro,
    Menu,
    Asking(Engine),
    Info,
    About,
}

#[derive(Debug, PartialEq)]
pub enum AppFlow {
    Stay,
    Quit,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MenuItem {
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
pub struct Menu {
    index: usize,
}

impl Menu {
    pub const ALL: [MenuItem; 4] = [
        MenuItem::Ask,
        MenuItem::Info,
        MenuItem::About,
        MenuItem::Exit,
    ];
}

impl App {
    pub fn new() -> Self {
        App::default()
    }
    pub fn handle_key(&mut self, key: KeyPress) -> AppFlow {
        match &mut self.screen {
            Screen::Intro => match key {
                KeyPress::Enter => {
                    self.screen = Screen::Menu;
                    AppFlow::Stay
                }
                KeyPress::Esc => AppFlow::Quit,
                _ => AppFlow::Stay,
            },
            Screen::Menu => match key {
                KeyPress::Enter => match self.menu.index() {
                    0 => {
                        self.screen = Screen::Asking(Engine::new(DECOY_STRING));
                        AppFlow::Stay
                    }
                    1 => {
                        self.screen = Screen::Info;
                        AppFlow::Stay
                    }
                    2 => {
                        self.screen = Screen::About;
                        AppFlow::Stay
                    }
                    3 => AppFlow::Quit,
                    _ => AppFlow::Stay,
                },
                KeyPress::Esc => AppFlow::Quit,
                KeyPress::Up => {
                    self.menu.move_menu_up();
                    AppFlow::Stay
                }
                KeyPress::Down => {
                    self.menu.move_menu_down();
                    AppFlow::Stay
                }
                _ => AppFlow::Stay,
            },
            Screen::Asking(engine) => match key {
                KeyPress::Enter => {
                    engine.handle_key(KeyPress::Enter);
                    AppFlow::Stay
                }
                KeyPress::Esc => {
                    self.screen = Screen::Menu;
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
            Screen::Info => match key {
                KeyPress::Esc => {
                    self.screen = Screen::Menu;
                    AppFlow::Stay
                }
                _ => AppFlow::Stay,
            },
            Screen::About => match key {
                KeyPress::Esc => {
                    self.screen = Screen::Menu;
                    AppFlow::Stay
                }
                _ => AppFlow::Stay,
            },
        }
    }
    pub fn screen(&self) -> &Screen {
        &self.screen
    }

    pub fn menu(&self) -> &Menu {
        &self.menu
    }
}

impl Menu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn move_menu_down(&mut self) {
        let menu_size = Self::ALL.len();
        self.index = (self.index + 1) % menu_size;
    }

    pub fn move_menu_up(&mut self) {
        let menu_size = Self::ALL.len();
        self.index = (self.index + menu_size - 1) % menu_size;
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::engine::KeyPress;

    /// Replay a sequence of keystrokes from a fresh app, handing back the final
    /// state *and* the `AppFlow` returned by the last key (Stay/Quit).
    fn drive_flow(keys: &[KeyPress]) -> (App, AppFlow) {
        let mut state = App::new();
        let mut flow = AppFlow::Stay;
        for &key in keys {
            flow = state.handle_key(key);
        }
        (state, flow)
    }

    fn drive(keys: &[KeyPress]) -> App {
        drive_flow(keys).0
    }

    fn selected(state: &App) -> MenuItem {
        Menu::ALL[state.menu().index()]
    }

    fn on_menu(state: &App) -> bool {
        matches!(state.screen(), Screen::Menu)
    }

    // ── Intro ────────────────────────────────────────────────────────────────

    #[test]
    fn new_starts_at_intro() {
        assert!(matches!(App::new().screen(), Screen::Intro));
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
        match state.screen {
            // A brand-new prank session: nothing typed, nothing on screen yet.
            Screen::Asking(engine) => assert_eq!(engine.visible_buffer(), ""),
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn menu_enter_on_informacoes_opens_info() {
        let state = drive(&[KeyPress::Enter, KeyPress::Down, KeyPress::Enter]);
        assert!(matches!(state.screen(), Screen::Info));
    }

    #[test]
    fn menu_enter_on_sobre_opens_about() {
        let state = drive(&[
            KeyPress::Enter,
            KeyPress::Down,
            KeyPress::Down,
            KeyPress::Enter,
        ]);
        assert!(matches!(state.screen(), Screen::About));
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
        match state.screen {
            Screen::Asking(engine) => assert_eq!(engine.visible_buffer(), "oi"),
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
    // The whole point of hoisting `index` to the app struct: the cursor is
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
            KeyPress::Down,  // Sobre (2)
            KeyPress::Enter, // into Sobre
            KeyPress::Esc,   // back to Menu, still at 2
            KeyPress::Down,  // → Sair (3)
        ]);
        assert!(on_menu(&state));
        assert_eq!(selected(&state), MenuItem::Exit);
    }
}

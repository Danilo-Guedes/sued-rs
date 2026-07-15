//! Top-level application state machine (M2).
//!
//! [`App`] is the **app shell** — a struct pairing the current [`Screen`] with the
//! menu cursor ([`Menu`]), so the selection survives moving between screens. The pure
//! prank lives in [`crate::core::engine`] and stays untouched — on the question
//! screen, `Screen::Asking` simply *owns* one `Engine` and forwards keys to it.
//!

use std::time::Instant;

use crate::{
    audio::AudioCue,
    config::Configuration,
    constants::{DECOY_STRING, DENIED_STRING},
    core::engine::{Engine, KeyPress, StateChange},
};

#[derive(Debug)]
pub struct App {
    screen: Screen,
    menu: MenuIndex,
    started_at: Instant,
    pending_cue: Option<AudioCue>,
    configuration: ConfigIndex,
}

#[derive(Default, Debug)]
pub enum Screen {
    #[default]
    Intro,
    Menu,
    Asking {
        engine: Engine,
        replied_at: Option<Instant>,
        denied_message: Option<&'static str>,
    },
    Info,
    About,
    Config {
        draft: Configuration,
    },
}

#[derive(Debug, PartialEq)]
pub enum AppFlow {
    Stay,
    Quit,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MenuOption {
    Ask,
    Info,
    About,
    Config,
    Exit,
}

impl MenuOption {
    pub fn label(&self) -> &'static str {
        match self {
            MenuOption::Ask => "PERGUNTAR AO ORÁCULO",
            MenuOption::Info => "INFORMAÇÕES",
            MenuOption::About => "SOBRE O SUED",
            MenuOption::Config => "CONFIGURAÇÃO",
            MenuOption::Exit => "SAIR",
        }
    }
}

#[derive(Debug, Default)]
pub struct MenuIndex {
    selected: usize,
}

impl MenuIndex {
    pub const ALL: [MenuOption; 5] = [
        MenuOption::Ask,
        MenuOption::Info,
        MenuOption::About,
        MenuOption::Config,
        MenuOption::Exit,
    ];
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum ConfigOption {
    #[default]
    Theme,
    Animations,
    Volume,
    Language,
}

#[derive(Debug, Default)]
pub struct ConfigIndex {
    selected: usize,
}

impl ConfigIndex {
    pub const ALL: [ConfigOption; 4] = [
        ConfigOption::Theme,
        ConfigOption::Animations,
        ConfigOption::Volume,
        ConfigOption::Language,
    ];
}

impl App {
    pub fn new() -> Self {
        App {
            screen: Screen::default(),
            menu: MenuIndex::default(),
            started_at: Instant::now(),
            pending_cue: None,
            configuration: ConfigIndex::default(),
        }
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
                KeyPress::Enter => match MenuIndex::ALL[self.menu.index()] {
                    MenuOption::Ask => {
                        self.screen = Screen::Asking {
                            engine: Engine::new(DECOY_STRING),
                            replied_at: None,
                            denied_message: None,
                        };
                        AppFlow::Stay
                    }
                    MenuOption::Info => {
                        self.screen = Screen::Info;
                        AppFlow::Stay
                    }
                    MenuOption::About => {
                        self.screen = Screen::About;
                        AppFlow::Stay
                    }
                    MenuOption::Config => {
                        self.screen = Screen::Config {
                            draft: Configuration::default(),
                        };
                        AppFlow::Stay
                    }
                    MenuOption::Exit => AppFlow::Quit,
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
            Screen::Asking {
                engine,
                replied_at,
                denied_message,
            } => match key {
                KeyPress::Enter => {
                    let state = engine.handle_key(KeyPress::Enter);

                    match state {
                        StateChange::Revealed => {
                            *denied_message = None;
                            *replied_at = Some(Instant::now());
                            self.pending_cue = Some(AudioCue::Laugh);
                        }
                        StateChange::Denied => {
                            *denied_message = Some(DENIED_STRING);
                            *replied_at = Some(Instant::now());
                            self.pending_cue = Some(AudioCue::JumpScare);
                        }
                        _ => {}
                    }

                    AppFlow::Stay
                }
                KeyPress::Esc => {
                    self.screen = Screen::Menu;
                    AppFlow::Stay
                }
                KeyPress::Backspace => {
                    if replied_at.is_none() {
                        engine.handle_key(KeyPress::Backspace);
                    }
                    AppFlow::Stay
                }
                KeyPress::F5 => {
                    engine.handle_key(KeyPress::F5);
                    *replied_at = None;
                    *denied_message = None;
                    self.pending_cue = None;
                    AppFlow::Stay
                }
                other_char => {
                    if replied_at.is_none() {
                        engine.handle_key(other_char);
                    }
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
            Screen::Config { draft: _ } => match key {
                KeyPress::Enter => todo!(),
                KeyPress::Esc => {
                    self.screen = Screen::Menu;
                    AppFlow::Stay
                }
                KeyPress::Up => {
                    self.configuration.move_config_menu_up();
                    AppFlow::Stay
                }
                KeyPress::Down => {
                    self.configuration.move_config_menu_down();
                    AppFlow::Stay
                }
                KeyPress::Left => todo!(),
                KeyPress::Right => todo!(),
                _ => AppFlow::Stay,
            },
        }
    }
    pub fn screen(&self) -> &Screen {
        &self.screen
    }

    pub fn menu(&self) -> &MenuIndex {
        &self.menu
    }

    pub fn started_at(&self) -> &Instant {
        &self.started_at
    }

    pub fn take_cue(&mut self) -> Option<AudioCue> {
        self.pending_cue.take()
    }
}

impl MenuIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn move_menu_down(&mut self) {
        let menu_size = Self::ALL.len();
        self.selected = (self.selected + 1) % menu_size;
    }

    pub fn move_menu_up(&mut self) {
        let menu_size = Self::ALL.len();
        self.selected = (self.selected + menu_size - 1) % menu_size;
    }

    pub fn index(&self) -> usize {
        self.selected
    }
}

impl ConfigIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn move_config_menu_down(&mut self) {
        let menu_size = Self::ALL.len();
        self.selected = (self.selected + 1) % menu_size;
    }

    pub fn move_config_menu_up(&mut self) {
        let menu_size = Self::ALL.len();
        self.selected = (self.selected + menu_size - 1) % menu_size;
    }

    pub fn index(&self) -> usize {
        self.selected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{audio::AudioCue, constants::DENIED_STRING, core::engine::KeyPress};

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

    fn selected(state: &App) -> MenuOption {
        MenuIndex::ALL[state.menu().index()]
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
        assert_eq!(selected(&state), MenuOption::Ask);
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
        assert_eq!(selected(&state), MenuOption::Info);
    }

    #[test]
    fn menu_down_wraps_past_last_item() {
        // Perguntar → Informacoes → Sobre → Config → Sair → back to Perguntar.
        let state = drive(&[
            KeyPress::Enter,
            KeyPress::Down,
            KeyPress::Down,
            KeyPress::Down,
            KeyPress::Down,
            KeyPress::Down,
        ]);
        assert_eq!(selected(&state), MenuOption::Ask);
    }

    #[test]
    fn menu_up_wraps_to_last_item() {
        // From the first item, Up lands on Sair.
        let state = drive(&[KeyPress::Enter, KeyPress::Up]);
        assert_eq!(selected(&state), MenuOption::Exit);
    }

    // ── Menu selection (Enter routes per item) ───────────────────────────────

    #[test]
    fn menu_enter_on_perguntar_opens_a_fresh_question() {
        let (state, flow) = drive_flow(&[KeyPress::Enter, KeyPress::Enter]);
        assert_eq!(flow, AppFlow::Stay);
        match state.screen {
            // A brand-new prank session: nothing typed, nothing on screen yet.
            Screen::Asking { engine, .. } => assert_eq!(engine.visible_buffer(), ""),
            other => panic!("expected Asking {{ engine, replied_at }}, got {other:?}"),
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
            Screen::Asking { engine, .. } => assert_eq!(engine.visible_buffer(), "oi"),
            other => panic!("expected Asking {{ engine, revealed_ay }}, got {other:?}"),
        }
    }

    #[test]
    fn question_esc_returns_to_menu() {
        let state = drive(&[KeyPress::Enter, KeyPress::Enter, KeyPress::Esc]);
        assert!(on_menu(&state));
    }

    // ── Denial: SUED rejects the uninitiated ─────────────────────────────────
    // Someone who doesn't know the ';' trick types a question in the open, so the
    // engine's `answer_buffer` stays empty and Enter yields `StateChange::Denied`.
    // The app must then surface a denial *phrase* for the SUED FALA box — the
    // taunt lives app-side (the engine only emits the event).

    #[test]
    fn enter_with_no_hidden_answer_shows_the_denial_phrase() {
        let state = drive(&[
            KeyPress::Enter, // Intro → Menu
            KeyPress::Enter, // Menu → Asking
            KeyPress::Char('o'),
            KeyPress::Char('i'), // a question typed in the open
            KeyPress::Enter,     // ask with an empty answer_buffer → Denied
        ]);
        match state.screen {
            Screen::Asking {
                engine,
                denied_message,
                ..
            } => {
                assert_eq!(
                    denied_message,
                    Some(DENIED_STRING),
                    "a denial must surface SUED's taunt phrase for the UI to show"
                );
                assert_eq!(engine.revealed(), None, "a denial reveals no answer");
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn revealing_a_real_answer_carries_no_denial() {
        // The mutually-exclusive case: a proper hidden answer reveals normally and
        // must NOT also carry a denial phrase.
        let state = drive(&[
            KeyPress::Enter,
            KeyPress::Enter,     // → Asking
            KeyPress::Char(';'), // Hidden
            KeyPress::Char('4'),
            KeyPress::Char('2'), // secret answer "42"
            KeyPress::Enter,     // reveal
        ]);
        match state.screen {
            Screen::Asking {
                engine,
                replied_at,
                denied_message,
            } => {
                assert_eq!(engine.revealed(), Some("42"));
                assert!(replied_at.is_some(), "the reveal clock started");
                assert_eq!(denied_message, None, "a real reveal carries no denial");
            }
            other => panic!("expected Asking, got {other:?}"),
        }
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

    // ── Menu selection PERSISTS across a sub-screen visit ─────────────────────
    // The whole point of hoisting `index` to the app struct: the cursor is
    // app-level state, so leaving the menu and returning must NOT reset it to 0.

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
            MenuOption::Info,
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
        assert_eq!(selected(&state), MenuOption::About);
    }

    #[test]
    fn question_esc_preserves_menu_selection() {
        // Ask is index 0, so this "worked" by coincidence with default() — pin it
        // so a future menu reorder can't silently break the round-trip.
        let state = drive(&[KeyPress::Enter, KeyPress::Enter, KeyPress::Esc]);
        assert!(on_menu(&state));
        assert_eq!(selected(&state), MenuOption::Ask);
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
            KeyPress::Down,  // → Config (3)
            KeyPress::Down,  // → Sair (4)
        ]);
        assert!(on_menu(&state));
        assert_eq!(selected(&state), MenuOption::Exit);
    }

    #[test]
    fn f5_returns_the_question_screen_to_a_fresh_state() {
        // Keystrokes that open the question screen and dirty it: reveal a
        // secret answer typed in Hidden mode.
        let dirty = [
            KeyPress::Enter,     // Intro → Menu
            KeyPress::Enter,     // Menu → Asking (fresh)
            KeyPress::Char(';'), // → Hidden
            KeyPress::Char('4'),
            KeyPress::Char('2'), // secret answer "42"
            KeyPress::Enter,     // reveal
        ];

        // Precondition: after that sequence the screen really is dirty —
        // otherwise a no-op F5 would pass this test for the wrong reason.
        let dirtied = drive(&dirty);
        match dirtied.screen {
            Screen::Asking {
                engine, replied_at, ..
            } => {
                assert!(engine.revealed().is_some(), "precondition: answer revealed");
                assert!(replied_at.is_some(), "precondition: reveal clock started");
            }
            other => panic!("expected Asking, got {other:?}"),
        }

        // Press F5 on top of that dirty state → a brand-new question session.
        let mut keys = dirty.to_vec();
        keys.push(KeyPress::F5);
        let reset = drive(&keys);
        match reset.screen {
            Screen::Asking {
                engine, replied_at, ..
            } => {
                assert_eq!(engine.visible_buffer(), "", "buffers cleared");
                assert_eq!(engine.revealed(), None, "no revealed answer");
                assert!(replied_at.is_none(), "reveal clock reset");
            }
            other => panic!("expected a fresh Asking, got {other:?}"),
        }
    }

    #[test]
    fn f5_clears_a_pending_denial() {
        // Dirty the screen with a DENIAL this time (the reveal path is covered
        // above): type a question in the open so answer_buffer stays empty, then
        // Enter → Denied, which parks a taunt in `denied_message`.
        let dirty = [
            KeyPress::Enter, // Intro → Menu
            KeyPress::Enter, // Menu → Asking (fresh)
            KeyPress::Char('o'),
            KeyPress::Char('i'), // a question typed in the open
            KeyPress::Enter,     // empty answer → Denied
        ];

        // Precondition: the denial really parked a taunt — otherwise a no-op F5
        // would pass this test for the wrong reason.
        let dirtied = drive(&dirty);
        match dirtied.screen {
            Screen::Asking { denied_message, .. } => {
                assert_eq!(
                    denied_message,
                    Some(DENIED_STRING),
                    "precondition: the denial parked a taunt to clear"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }

        // F5 = "new question" → the taunt must be gone, not linger into the fresh
        // session (else the SUED FALA box renders blank instead of the prompt).
        let mut keys = dirty.to_vec();
        keys.push(KeyPress::F5);
        let reset = drive(&keys);
        match reset.screen {
            Screen::Asking {
                replied_at,
                denied_message,
                ..
            } => {
                assert_eq!(denied_message, None, "F5 must clear the pending denial");
                assert!(replied_at.is_none(), "F5 resets the animation clock");
            }
            other => panic!("expected a fresh Asking, got {other:?}"),
        }
    }

    // ── Input locks once SUED has replied ────────────────────────────────────
    // After the oracle speaks (a denial OR a reveal), plain keystrokes must stop
    // reaching the input — only the hint-bar keys (Enter/F5/Esc/Ctrl-C) still act.

    #[test]
    fn keystrokes_are_ignored_after_a_denial() {
        // Ask a question in the open → Denied. SUED has replied.
        let until_reply = [
            KeyPress::Enter, // Intro → Menu
            KeyPress::Enter, // Menu → Asking
            KeyPress::Char('o'),
            KeyPress::Char('i'), // question → visible "oi"
            KeyPress::Enter,     // empty answer → Denied (a reply)
        ];

        // Precondition: SUED really replied and the input still holds the question.
        match drive(&until_reply).screen {
            Screen::Asking {
                engine,
                denied_message,
                ..
            } => {
                assert!(
                    denied_message.is_some(),
                    "precondition: SUED replied (denied)"
                );
                assert_eq!(engine.visible_buffer(), "oi");
            }
            other => panic!("expected Asking, got {other:?}"),
        }

        // Hammer more chars after the reply — they must be swallowed.
        let mut keys = until_reply.to_vec();
        keys.extend([KeyPress::Char('x'), KeyPress::Char('y')]);
        match drive(&keys).screen {
            Screen::Asking { engine, .. } => {
                assert_eq!(
                    engine.visible_buffer(),
                    "oi",
                    "keystrokes after a reply must not reach the input"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn keystrokes_are_ignored_after_a_reveal() {
        // The other reply path: reveal a hidden answer, then keep typing. The
        // decoy prefix already on screen must not grow — compared against itself
        // so the test doesn't hard-code the decoy's content.
        let until_reply = [
            KeyPress::Enter,
            KeyPress::Enter,     // → Asking
            KeyPress::Char(';'), // Hidden
            KeyPress::Char('4'),
            KeyPress::Char('2'), // secret answer "42"
            KeyPress::Char(';'), // back to Normal
            KeyPress::Enter,     // reveal (a reply)
        ];

        let visible_at_reply = match drive(&until_reply).screen {
            Screen::Asking { engine, .. } => {
                assert!(
                    engine.revealed().is_some(),
                    "precondition: SUED replied (revealed)"
                );
                engine.visible_buffer().to_string()
            }
            other => panic!("expected Asking, got {other:?}"),
        };

        let mut keys = until_reply.to_vec();
        keys.extend([KeyPress::Char('x'), KeyPress::Char('y')]);
        match drive(&keys).screen {
            Screen::Asking { engine, .. } => {
                assert_eq!(
                    engine.visible_buffer(),
                    visible_at_reply,
                    "post-reveal keystrokes must not reach the input"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn a_reveal_queues_the_laugh_cue() {
        let mut state = drive(&[
            KeyPress::Enter,
            KeyPress::Enter,     // → Asking
            KeyPress::Char(';'), // Hidden
            KeyPress::Char('4'),
            KeyPress::Char('2'), // secret answer "42"
            KeyPress::Enter,     // reveal
        ]);
        assert_eq!(state.take_cue(), Some(AudioCue::Laugh));
    }

    #[test]
    fn a_denial_queues_the_jump_scare_cue() {
        let mut state = drive(&[
            KeyPress::Enter,
            KeyPress::Enter, // → Asking
            KeyPress::Char('o'),
            KeyPress::Char('i'), // a question typed in the open
            KeyPress::Enter,     // empty answer → Denied
        ]);
        assert_eq!(state.take_cue(), Some(AudioCue::JumpScare));
    }

    #[test]
    fn take_cue_drains_so_the_sound_fires_once() {
        let mut state = drive(&[
            KeyPress::Enter,
            KeyPress::Enter,
            KeyPress::Char(';'),
            KeyPress::Char('4'),
            KeyPress::Enter, // reveal
        ]);
        assert_eq!(
            state.take_cue(),
            Some(AudioCue::Laugh),
            "the first drain gets the cue"
        );
        assert_eq!(
            state.take_cue(),
            None,
            "the second drain is empty — a reply plays its sound exactly once"
        );
    }

    #[test]
    fn plain_typing_queues_no_cue() {
        let mut state = drive(&[
            KeyPress::Enter,
            KeyPress::Enter,
            KeyPress::Char('o'),
            KeyPress::Char('i'), // typed a question, but no Enter yet
        ]);
        assert_eq!(state.take_cue(), None, "no reply yet → nothing to play");
    }
}

//! Top-level application state machine (M2).
//!
//! [`App`] is the **app shell** — a struct pairing the current [`Screen`] with the
//! menu cursor ([`Menu`]), so the selection survives moving between screens. The pure
//! prank lives in [`crate::core::engine`] and stays untouched — on the question
//! screen, `Screen::Asking` simply *owns* one `Engine` and forwards keys to it.
//!

use std::time::{Duration, Instant};

use crate::{
    audio::AudioCue,
    config::{ConfigOption, Configuration, Direction},
    conversation::{ConfirmChoice, HistoryView, Message, Overlay},
    core::engine::{Engine, KeyPress, StateChange},
    language::{Language, Translation, pick},
    ui::effects::{is_thinking, reveal_elapsed, reveal_is_complete, thinking_duration},
};

pub const THUNDER_AT_CHARS_REMAINING: usize = 20;

/// G17 — a question of this many characters **or fewer** earns the rebuke
/// instead of a random denial. Inclusive: 18 is short, 19 is not.
///
/// ⚠ His call, 2026-08-06, and the tradeoff is deliberate rather than an
/// oversight: 18 catches every greeting he named (`hello there` = 11,
/// `what is this?` = 13, `how you doing?` = 14) **and also catches genuinely
/// short real questions** — `does she love me?` is 17, `vou passar?` is 11.
/// That is in character (the ritual demands you flatter and elaborate), so a
/// terse question earning a rebuke is the feature, not a bug. Do not "fix" it.
pub const SHORT_QUESTION_CHARS: usize = 18;

#[derive(Debug)]
pub struct App {
    screen: Screen,
    menu: MenuIndex,
    pending_cue: Option<AudioCue>,
    config_navigation: ConfigIndex,
    pending_save: Option<Configuration>,
    pub config_object: Configuration,
    pub started_at: Instant,
}

#[derive(Debug)]
pub struct AskingState {
    thunder_played: bool,
    overlay: Option<Overlay>,
    pub history: Vec<Message>,
    pub engine: Engine,
    pub spell: &'static str,
    pub reply: Option<Reply>,
}

impl AskingState {
    /// The last thing SueD actually said, or `None` when the only thing in the
    /// transcript is the seeded greeting — which is exactly the "nothing has been
    /// asked yet" case the welcome screen exists for.
    pub fn previous_reply(&self) -> Option<&str> {
        let mut spoken = self
            .history
            .iter()
            .skip(1) // skip greeting msg
            .rev()
            .filter_map(|message| match message {
                Message::Sued(words) => Some(words.as_str()),
                Message::User(_) => None,
            });

        if self.reply.is_some() {
            spoken.next(); // the newest thing SueD said IS the live reply, we don't want that, we want the prior to that
        }

        spoken.next()
    }

    pub fn last_question(&self) -> Option<&str> {
        let mut spoken = self
            .history
            .iter()
            .rev()
            .filter_map(|message| match message {
                Message::User(words) => Some(words.as_str()),
                Message::Sued(_) => None,
            });

        spoken.next()
    }

    pub fn overlay(&self) -> Option<&Overlay> {
        self.overlay.as_ref()
    }

    pub fn transcript(&self) -> Option<&HistoryView> {
        match self.overlay() {
            Some(Overlay::Transcript(hv)) => Some(hv),
            _ => None,
        }
    }

    pub fn is_transcript_dirty(&self) -> bool {
        self.last_question().is_some()
    }
}

// `Asking` is 201 bytes against clippy's 200-byte threshold — one byte over,
// which `thunder_played` tipped. There is exactly ONE `Screen` in the program
// (`App.screen`, never in a collection), so the "waste" this lint is protecting
// against totals 207 bytes, once. Boxing would buy that back with a heap
// allocation per screen transition and a pointer indirection on every `engine`
// access — and `engine` is read in the render path, which runs every tick.
// ⏳ Revisit at G11+G12: folding these fields into `Option<Reply>` reshapes the
// variant anyway, and may drop it back under the threshold on its own.
#[allow(clippy::large_enum_variant)]
#[derive(Default, Debug)]
pub enum Screen {
    #[default]
    Intro,
    Menu,
    Asking(AskingState),
    Info,
    About,
    Config,
}

impl Screen {
    fn asking(translations: Translation) -> Self {
        Screen::Asking(AskingState {
            engine: Engine::new(pick(translations.decoys, rand::random())),
            reply: None,
            spell: pick(translations.ask.spells, rand::random()),
            thunder_played: false,
            history: vec![Message::Sued(String::from(translations.ask.welcome_line))],
            overlay: None,
        })
    }
}

#[derive(Debug)]
pub struct Reply {
    words: String,
    asked_at: Instant,
    thinking_for: Duration,
}

impl Reply {
    pub fn new(words: String, thinking_for: Duration) -> Self {
        Reply {
            words,
            asked_at: Instant::now(),
            thinking_for,
        }
    }
    pub fn words(&self) -> &str {
        self.words.as_str()
    }

    pub fn is_pondering(&self) -> bool {
        self.thinking_for > self.asked_at.elapsed()
    }

    pub fn speaking_elapsed(&self) -> Duration {
        reveal_elapsed(self.asked_at.elapsed(), self.thinking_for)
    }

    pub fn since_asked(&self) -> Duration {
        self.asked_at.elapsed()
    }
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
    pub fn label(&self, language: Language) -> &'static str {
        match self {
            MenuOption::Ask => match language {
                Language::PtBr => "PERGUNTAR AO ORÁCULO",
                Language::EnUs => "ASK THE ORACLE",
                Language::EsEs => "PREGUNTAR AL ORÁCULO",
            },

            MenuOption::Info => match language {
                Language::PtBr => "INFORMAÇÕES",
                Language::EnUs => "INFORMATION",
                Language::EsEs => "INFORMACIÓN",
            },

            MenuOption::About => match language {
                Language::PtBr => "SOBRE O SUED",
                Language::EnUs => "ABOUT SUED",
                Language::EsEs => "SOBRE SUED",
            },
            MenuOption::Config => match language {
                Language::PtBr => "CONFIGURAÇÃO",
                Language::EnUs => "CONFIGURATION",
                Language::EsEs => "CONFIGURACIÓN",
            },
            MenuOption::Exit => match language {
                Language::PtBr => "SAIR",
                Language::EnUs => "EXIT",
                Language::EsEs => "SALIR",
            },
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

    pub fn selected(&self) -> ConfigOption {
        ConfigIndex::ALL[self.selected]
    }
}

impl App {
    pub fn new(parsed_json_config: Configuration) -> Self {
        App {
            screen: Screen::default(),
            menu: MenuIndex::new(),
            started_at: Instant::now(),
            config_navigation: ConfigIndex::new(),
            pending_cue: None,
            config_object: parsed_json_config,
            pending_save: None,
        }
    }
    pub fn handle_key(&mut self, key: KeyPress) -> AppFlow {
        let translations = self.config().language().translation();

        match &mut self.screen {
            Screen::Intro => match key {
                KeyPress::Enter => {
                    self.screen = Screen::Menu;
                    AppFlow::Stay
                }
                KeyPress::Esc => AppFlow::Quit,
                KeyPress::CtrlC => AppFlow::Quit,
                _ => AppFlow::Stay,
            },
            Screen::Menu => match key {
                KeyPress::Enter => match MenuIndex::ALL[self.menu.index()] {
                    MenuOption::Ask => {
                        self.screen = Screen::asking(translations);
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
                        self.screen = Screen::Config;
                        AppFlow::Stay
                    }
                    MenuOption::Exit => AppFlow::Quit,
                },
                KeyPress::Esc => {
                    self.screen = Screen::Intro;
                    AppFlow::Stay
                }
                KeyPress::Up => {
                    self.menu.move_menu_up();
                    AppFlow::Stay
                }
                KeyPress::Down => {
                    self.menu.move_menu_down();
                    AppFlow::Stay
                }
                KeyPress::CtrlC => AppFlow::Quit,
                _ => AppFlow::Stay,
            },
            Screen::Asking(asking_state) => {
                let is_transcript_dirty = asking_state.is_transcript_dirty();

                match asking_state.overlay() {
                    Some(Overlay::Transcript(_)) => {
                        //first we discard the not allowed keypress
                        // to avoind leak somethig to the engine
                        // while the history popover is shown
                        match key {
                            KeyPress::F1
                            | KeyPress::Esc
                            | KeyPress::Up
                            | KeyPress::Down
                            | KeyPress::PageUp
                            | KeyPress::PageDown => {} // the popover's own keys
                            KeyPress::F5 => {}
                            KeyPress::CtrlC => return AppFlow::Quit,
                            _ => return AppFlow::Stay, // everything else swallowed
                        }
                    }
                    Some(Overlay::ConfirmLeave(_)) => {
                        match key {
                            KeyPress::Esc | KeyPress::Left | KeyPress::Right | KeyPress::Enter => {} // the confirm's own keys
                            KeyPress::F5 => {}
                            KeyPress::CtrlC => return AppFlow::Quit,
                            _ => return AppFlow::Stay, // everything else swallowed
                        }
                    }
                    None => {}
                }

                // The conversation guard (G8). Three time-paths converge on the
                // ordinary key handling below: SueD never spoke → fall straight
                // through; SueD mid-reply → swallow the key (only F5/Esc still
                // act); SueD finished → this key begins the next exchange, so
                // the live reply rotates aside and everything re-arms first.
                if let Some(replied) = &mut asking_state.reply {
                    let current_sued_words = replied.words();

                    let sued_finished_speaking = reveal_is_complete(
                        current_sued_words,
                        reveal_elapsed(replied.asked_at.elapsed(), replied.thinking_for),
                    );

                    if sued_finished_speaking {
                        asking_state
                            .engine
                            .reset(pick(translations.decoys, rand::random()));
                        // A new decoy owes a new warning. This is the SECOND
                        // `engine.reset` site (F5 is the other) — re-arming
                        // belongs wherever a decoy begins, not to one key.
                        asking_state.thunder_played = false;

                        asking_state.reply = None;
                    } else {
                        match key {
                            // The lock only holds keys that would feed the
                            // question — the panic button and the door still
                            // work while SueD is speaking.
                            KeyPress::F5 => {}
                            KeyPress::Esc => {}
                            KeyPress::CtrlC => return AppFlow::Quit,
                            _ => return AppFlow::Stay,
                        }
                    }
                }

                match key {
                    KeyPress::Enter => {
                        match &mut asking_state.overlay {
                            Some(Overlay::Transcript(_)) => {}
                            Some(Overlay::ConfirmLeave(confirm_choises)) => match confirm_choises {
                                ConfirmChoice::Leave => {
                                    self.screen = Screen::Menu;
                                }
                                ConfirmChoice::Stay => {
                                    asking_state.overlay = None;
                                }
                            },
                            None => {
                                let question = asking_state.engine.visible_buffer().to_string();

                                let state = asking_state.engine.handle_key(KeyPress::Enter);

                                let (sued_words, thinking_duration) = match state {
                                    StateChange::Revealed => (
                                        Some(
                                            asking_state
                                                .engine
                                                .revealed()
                                                .expect("Revealed implied a revealed answer")
                                                .to_string(),
                                        ),
                                        thinking_duration(rand::random()),
                                    ),
                                    StateChange::Denied => {
                                        if question.chars().count() <= SHORT_QUESTION_CHARS {
                                            (
                                                Some(
                                                    translations
                                                        .rebuke
                                                        .replace("{question}", &question),
                                                ),
                                                Duration::ZERO,
                                            )
                                        } else {
                                            (
                                                Some(
                                                    pick(translations.denials, rand::random())
                                                        .to_string(),
                                                ),
                                                thinking_duration(rand::random()),
                                            )
                                        }
                                    }
                                    _ => (None, Duration::ZERO),
                                };

                                if let Some(words) = sued_words {
                                    asking_state.history.push(Message::User(question));
                                    asking_state.history.push(Message::Sued(words.clone()));
                                    asking_state.reply = Some(Reply::new(words, thinking_duration));
                                    asking_state.spell =
                                        pick(translations.ask.spells, rand::random());
                                    asking_state.thunder_played = false;
                                }
                            }
                        }

                        AppFlow::Stay
                    }
                    KeyPress::Esc => {
                        match &mut asking_state.overlay {
                            Some(Overlay::Transcript(_)) => {
                                asking_state.overlay = None;
                            }
                            Some(Overlay::ConfirmLeave(_)) => {
                                asking_state.overlay = None;
                            }

                            None => {
                                // here user is in the main ask screen
                                if is_transcript_dirty {
                                    asking_state.overlay =
                                        Some(Overlay::ConfirmLeave(ConfirmChoice::default()));
                                } else {
                                    self.screen = Screen::Menu;
                                }
                            }
                        }

                        AppFlow::Stay
                    }
                    KeyPress::Backspace => {
                        asking_state.engine.handle_key(KeyPress::Backspace);
                        AppFlow::Stay
                    }
                    KeyPress::F5 => {
                        self.screen = Screen::asking(translations);
                        AppFlow::Stay
                    }
                    KeyPress::CtrlC => AppFlow::Quit,
                    KeyPress::F1 => {
                        if let Some(Overlay::Transcript(_)) = &mut asking_state.overlay {
                            asking_state.overlay = None;
                        } else {
                            // here the hitview is close. needs toggle on
                            asking_state.overlay = Some(Overlay::Transcript(HistoryView::new()));
                        }
                        AppFlow::Stay
                    }
                    KeyPress::PageUp => {
                        if let Some(Overlay::Transcript(inner_hist_view)) =
                            &mut asking_state.overlay
                        {
                            inner_hist_view.handle_page_up();
                        }
                        AppFlow::Stay
                    }
                    KeyPress::PageDown => {
                        if let Some(Overlay::Transcript(inner_hist_view)) =
                            &mut asking_state.overlay
                        {
                            inner_hist_view.handle_page_down();
                        }
                        AppFlow::Stay
                    }
                    KeyPress::Up => {
                        // logic here

                        if let Some(Overlay::Transcript(inner_hist_view)) =
                            &mut asking_state.overlay
                        {
                            inner_hist_view.handle_up();
                        }
                        AppFlow::Stay
                    }
                    KeyPress::Down => {
                        // logic here
                        if let Some(Overlay::Transcript(inner_hist_view)) =
                            &mut asking_state.overlay
                        {
                            inner_hist_view.handle_down();
                        }
                        AppFlow::Stay
                    }
                    KeyPress::Left | KeyPress::Right => {
                        if let Some(Overlay::ConfirmLeave(choices)) = &mut asking_state.overlay {
                            choices.toggle();
                        }
                        AppFlow::Stay
                    }

                    other_char => {
                        asking_state.engine.handle_key(other_char);

                        if !asking_state.thunder_played
                            && (asking_state.engine.decoy_chars_remaining()
                                <= THUNDER_AT_CHARS_REMAINING)
                        {
                            self.pending_cue = Some(AudioCue::Thunder);
                            asking_state.thunder_played = true;
                        }

                        AppFlow::Stay
                    }
                }
            }
            Screen::Info => match key {
                KeyPress::Esc => {
                    self.screen = Screen::Menu;
                    AppFlow::Stay
                }
                KeyPress::CtrlC => AppFlow::Quit,
                _ => AppFlow::Stay,
            },
            Screen::About => match key {
                KeyPress::Esc => {
                    self.screen = Screen::Menu;
                    AppFlow::Stay
                }
                KeyPress::CtrlC => AppFlow::Quit,
                _ => AppFlow::Stay,
            },
            Screen::Config => match key {
                KeyPress::Esc => {
                    self.pending_save = Some(self.config_object);
                    self.screen = Screen::Menu;
                    AppFlow::Stay
                }
                KeyPress::CtrlC => {
                    self.pending_save = Some(self.config_object);
                    AppFlow::Quit
                }
                KeyPress::Up => {
                    self.config_navigation.move_config_menu_up();
                    AppFlow::Stay
                }
                KeyPress::Down => {
                    self.config_navigation.move_config_menu_down();
                    AppFlow::Stay
                }
                KeyPress::Left => {
                    self.config_object
                        .step(self.config_navigation.selected(), Direction::Previous);
                    AppFlow::Stay
                }
                KeyPress::Right => {
                    self.config_object
                        .step(self.config_navigation.selected(), Direction::Next);
                    AppFlow::Stay
                }
                _ => AppFlow::Stay,
            },
        }
    }

    pub fn screen(&self) -> &Screen {
        &self.screen
    }

    /// Test-only: pretend the live question was asked `by` earlier.
    ///
    /// The reply's interesting states are 3–6 seconds of wall-clock apart, so a
    /// render test cannot reach them without either sleeping (slow and flaky) or
    /// moving the clock. This moves the clock. `#[cfg(test)]`, so it never ships.
    ///
    /// Same rewind trick as `finish_the_reveal` in this module's own tests —
    /// this one exists because `Reply`'s fields are private, which is exactly the
    /// encapsulation that stopped a call site picking the wrong clock.
    #[cfg(test)]
    pub(crate) fn rewind_reply(&mut self, by: Duration) {
        let Screen::Asking(AskingState {
            reply: Some(reply), ..
        }) = &mut self.screen
        else {
            panic!("rewind_reply expects a live reply on the ask screen");
        };
        reply.asked_at = reply
            .asked_at
            .checked_sub(by)
            .expect("the test clock must be able to rewind");
    }

    pub fn menu(&self) -> &MenuIndex {
        &self.menu
    }

    pub fn started_at(&self) -> &Instant {
        &self.started_at
    }

    pub fn take_pending_cue(&mut self) -> Option<AudioCue> {
        self.pending_cue.take()
    }

    pub fn config(&self) -> Configuration {
        self.config_object
    }

    pub fn focused_option(&self) -> ConfigOption {
        self.config_navigation.selected()
    }

    pub fn take_pending_save(&mut self) -> Option<Configuration> {
        self.pending_save.take()
    }

    /// True while SueD has been asked but has not started speaking yet.
    ///
    /// `main`'s tick loop watches this and fires the reply sting on the FALLING
    /// edge — the instant the ponder ends. That transition happens with no
    /// keypress at all, which is why the old `pending_cue` seam could not express
    /// it: that one was drained inside the keypress block, so it fired the sting
    /// at Enter, seconds before SueD actually spoke. It was removed with G13.
    ///
    /// ⚠ This is deliberately LEVEL-triggered — it answers "is SueD pondering
    /// *right now*", and stays true for the whole pause. The "play the sting
    /// exactly once" guarantee therefore lives entirely in `main`'s
    /// `was_pondering && !pondering_now` edge check, which no test covers
    /// (it is two operators at the I/O edge). Do not "simplify" that call site
    /// into a plain level test, or the sting replays every tick for 3-6s.
    pub fn is_pondering(&self) -> bool {
        match &self.screen {
            Screen::Asking(AskingState {
                reply: Some(reply), ..
            }) => is_thinking(reply.asked_at.elapsed(), reply.thinking_for),
            _ => false,
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{
        DENIED_QUESTION, DENIED_QUESTION_PT, REBUKED_QUESTION, ask_and_be_denied,
        ask_and_be_rebuked, ask_openly, typing,
    };
    use crate::{conversation::PAGE_ROWS, core::engine::KeyPress};
    use std::time::Duration;

    /// Replay a sequence of keystrokes from a fresh app, handing back the final
    /// state *and* the `AppFlow` returned by the last key (Stay/Quit).
    fn drive_flow(keys: &[KeyPress]) -> (App, AppFlow) {
        let mut state = App::new(Configuration::default());
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

    // ── G8: the exchange is a conversation, not a wipe ───────────────────────
    // The old flow answered once and froze until F5. The new one: SueD replies,
    // the crawl finishes, the input reopens EMPTY — and the answer you just got
    // stays on screen while you type the next question, so the screen reads as a
    // back-and-forth. Only F5 (or leaving) forgets the conversation.
    //
    // The unlock is *time*-driven — it happens when the typewriter finishes, not
    // when a key arrives — so these tests rewind the reply clock rather than
    // sleeping. `finish_the_reveal` is what "SueD stopped talking" looks like to
    // the app, and no test below is allowed to depend on wall-clock speed.

    /// Keep driving an app that is already mid-conversation. `drive` always
    /// starts from scratch, which can't express "reply, wait, then type".
    fn feed(app: &mut App, keys: &[KeyPress]) {
        for &key in keys {
            app.handle_key(key);
        }
    }

    /// Rewind the reply clock far enough that the crawl has certainly ended —
    /// the app must now behave as though SueD has finished speaking.
    fn finish_the_reveal(app: &mut App) {
        match &mut app.screen {
            Screen::Asking(AskingState {
                reply: Some(reply), ..
            }) => {
                reply.asked_at = reply
                    .asked_at
                    .checked_sub(Duration::from_secs(60))
                    .expect("the test clock must be able to rewind 60s");
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    /// Menu → Asking, then whisper "42" and reveal it.
    const ASK_AND_REVEAL: [KeyPress; 6] = [
        KeyPress::Enter,     // Intro → Menu
        KeyPress::Enter,     // Menu → Asking
        KeyPress::Char(';'), // Hidden
        KeyPress::Char('4'),
        KeyPress::Char('2'), // the secret answer
        KeyPress::Enter,     // reveal
    ];

    // ── Intro ────────────────────────────────────────────────────────────────

    #[test]
    fn new_starts_at_intro() {
        assert!(matches!(
            App::new(Configuration::default()).screen(),
            Screen::Intro
        ));
    }

    #[test]
    fn new_seeds_the_live_config_from_the_loaded_value() {
        // `main` loads `sued.config.json` at startup and hands the result to `App::new`;
        // the app must adopt it as its live config, not silently fall back to
        // defaults. A non-default value proves the seed actually threads through.
        let loaded =
            Configuration::from_json(r#"{ "audio_volume": 33 }"#).expect("a valid partial config");

        let app = App::new(loaded);

        assert_eq!(
            app.config(),
            loaded,
            "App::new must adopt the config it is given"
        );
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
            Screen::Asking(AskingState { engine, .. }) => assert_eq!(engine.visible_buffer(), ""),
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
    fn menu_esc_should_return_to_intro() {
        let state = drive(&[KeyPress::Enter, KeyPress::Esc]);
        assert!(matches!(state.screen(), Screen::Intro));
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
            Screen::Asking(AskingState { engine, .. }) => assert_eq!(engine.visible_buffer(), "oi"),
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
    fn enter_with_no_hidden_answer_shows_the_denial_phrase_if_pass_rebuke_char_count() {
        let state = drive(&ask_and_be_denied());

        let denials = state.config().language().translation().denials;
        match state.screen {
            Screen::Asking(AskingState {
                engine,
                reply: Some(reply),
                ..
            }) => {
                let taunt = reply.words();
                assert!(
                    denials.contains(&taunt),
                    "the taunt must come from the active language's denial pool, got {taunt:?}"
                );
                assert_eq!(engine.revealed(), None, "a denial reveals no answer");
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
            Screen::Asking(AskingState { engine, reply, .. }) => {
                assert!(engine.revealed().is_some(), "precondition: answer revealed");
                assert!(reply.is_some(), "precondition: Sued replied");
            }
            other => panic!("expected Asking, got {other:?}"),
        }

        // Press F5 on top of that dirty state → a brand-new question session.
        let mut keys = dirty.to_vec();
        keys.push(KeyPress::F5);
        let reset = drive(&keys);
        match reset.screen {
            Screen::Asking(AskingState { engine, reply, .. }) => {
                assert_eq!(engine.visible_buffer(), "", "buffers cleared");
                assert_eq!(engine.revealed(), None, "no revealed answer");
                assert!(reply.is_none(), "no reply struct");
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
            Screen::Asking(AskingState { reply, .. }) => {
                assert!(
                    reply.is_some(),
                    "precondition: the is dirty and should be a some"
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
            Screen::Asking(AskingState { reply, .. }) => {
                assert!(reply.is_none(), "F5 must clear the pending denial");
            }
            other => panic!("expected a fresh Asking, got {other:?}"),
        }
    }

    // ── Input locks once SUED has replied ────────────────────────────────────
    // After the oracle speaks (a denial OR a reveal), plain keystrokes must stop
    // reaching the input — only the hint-bar keys (Enter/F5/Esc/Ctrl+C) still act.

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

        // Precondition: SUED really replied — and the reply CONSUMED the
        // question (G8 amendment): the ENGINE's buffer is already empty while
        // SueD taunts.
        //
        // ⚠ AMENDED 2026-08-04 (G15) — READ THE WORD "ENGINE". This test's claim
        // survived G15 untouched, but the sentence that used to describe it
        // ("the input already reads empty") did not, because it was two facts
        // wearing one coat:
        //
        //   what the ENGINE holds  → cleared at `Enter`. Still true. Asserted here.
        //   what the SCREEN draws  → the mark's question, until SueD stops
        //                            speaking. Inverted by G15. NOT asserted here.
        //
        // The clearing is load-bearing for the trick (`visible_buffer` must not
        // keep growing behind a reply), so this assertion protects the gimmick
        // and must not be weakened into "the screen looks empty" — it never
        // checked that, and since G15 that would be false. The screen side is
        // pinned by `the_question_stays_on_screen_while_sued_is_still_speaking`
        // in `ui/screens.rs`, which has to be a DRAW test for exactly this
        // reason: no state assertion can see a change that only picks which
        // string reaches a `Span`.
        match drive(&until_reply).screen {
            Screen::Asking(AskingState { engine, reply, .. }) => {
                assert!(reply.is_some(), "precondition: SUED replied (denied)");
                assert_eq!(
                    engine.visible_buffer(),
                    "",
                    "the reply must consume the question from the input"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }

        // Hammer more chars after the reply — they must be swallowed.
        let mut keys = until_reply.to_vec();
        keys.extend([KeyPress::Char('x'), KeyPress::Char('y')]);
        match drive(&keys).screen {
            Screen::Asking(AskingState { engine, .. }) => {
                assert_eq!(
                    engine.visible_buffer(),
                    "",
                    "keystrokes after a reply must not reach the freshly emptied input"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    // ── G15 · the question that lingers ──────────────────────────────────────
    //
    // ⚠ These pin the ACCESSOR, not the render. `ask.rs` calls it every frame
    // and leans on both facts below: `None` on frame one (it becomes `""` via
    // `unwrap_or_default`, which is the pre-G15 behaviour), and *newest*, not
    // *previous*, once the séance is under way.
    //
    // ⚠ NAMING DEBT THESE TESTS DELIBERATELY DOCUMENT: `previous_user_message`
    // sits beside `previous_reply`, whose prefix is earned — it does `skip(1)`
    // AND an extra `next()` to step over the live reply. This one does neither,
    // because G15 wants the question being answered *right now*. Same prefix,
    // opposite semantics. The rename (`last_user_message` /
    // `question_being_answered`) is owed; these tests are written against the
    // current name so the suite still compiles, and the second one exists so a
    // future reader cannot mistake which of the two behaviours is intended.

    #[test]
    fn the_seeded_greeting_is_not_a_question() {
        // A fresh ask screen's `history` holds exactly one thing: `welcome_line`,
        // as `Message::Sued`. So on frame one there is no question to linger, and
        // the accessor must say so rather than handing back SueD's own greeting —
        // which is what a naive "just take the last message" would do.
        match drive(&[KeyPress::Enter, KeyPress::Enter]).screen {
            Screen::Asking(state) => assert_eq!(
                state.last_question(),
                None,
                "the seeded greeting belongs to SueD, not to the mark"
            ),
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn the_question_that_lingers_is_the_newest_one_not_the_one_before_it() {
        // ⚠ THE ASYMMETRY WITH `previous_reply`, made executable. Two exchanges,
        // so "newest" and "previous" are different strings and a test can tell
        // them apart — with one exchange this passes either way and pins nothing.
        //
        // G15 draws the question SueD is answering *right now*. Step over it the
        // way `previous_reply` steps over the live reply and the input line shows
        // the mark the wrong question — subtly, plausibly, and only on the second
        // exchange onward, which is exactly the kind of bug that survives a demo.
        let mut app = drive(&[
            KeyPress::Enter,
            KeyPress::Enter, // → Asking
            KeyPress::Char('u'),
            KeyPress::Char('m'),
            KeyPress::Enter, // 1st question, no hidden answer → Denied
        ]);

        // ⚠ Not optional. G8 locks the input while SueD speaks, so without
        // winding the clock the second question never reaches the engine and
        // this test quietly asserts against a ONE-exchange transcript — where
        // "newest" and "previous" are the same string and nothing is pinned.
        // (F5 is not an option here either: it rebuilds the screen and takes
        // `history` with it, which destroys the very thing under test.)
        finish_the_reveal(&mut app);
        feed(
            &mut app,
            &[
                KeyPress::Char('d'),
                KeyPress::Char('o'),
                KeyPress::Char('i'),
                KeyPress::Char('s'),
                KeyPress::Enter, // 2nd question → Denied
            ],
        );

        match app.screen {
            Screen::Asking(state) => {
                assert_eq!(
                    state.history.len(),
                    5,
                    "precondition: greeting + two questions + two replies"
                );
                assert_eq!(
                    state.last_question(),
                    Some("dois"),
                    "the lingering question must be the one being answered NOW"
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
            Screen::Asking(AskingState { engine, .. }) => {
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
            Screen::Asking(AskingState { engine, .. }) => {
                assert_eq!(
                    engine.visible_buffer(),
                    visible_at_reply,
                    "post-reveal keystrokes must not reach the input"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    // ── Config screen: [←→] alter values, immediate-apply ─────────────────────
    // Slice A of M5: the config lives in `App.config` (no draft). `[↑↓]` move the
    // row cursor; `[←→]` alter the selected row's value and apply it live. Discrete
    // rows (tema/animações/idioma) step through their options with WRAP; the one
    // continuous row (volume) steps ±10 and CLAMPS at 0/100. Nothing is written to
    // disk yet — persistence is Slice B.
    use crate::{language::Language, ui::theme::Theme};

    /// Drive a fresh app onto the Config screen (cursor on the first row, `tema`),
    /// then apply `then`. Menu order is Ask·Info·About·Config·Exit, so Config is
    /// three Downs from the top.
    fn on_config(then: &[KeyPress]) -> App {
        let mut keys = vec![
            KeyPress::Enter, // Intro → Menu
            KeyPress::Down,  // → Info
            KeyPress::Down,  // → About
            KeyPress::Down,  // → Config
            KeyPress::Enter, // → Screen::Config, cursor on `tema`
        ];
        keys.extend_from_slice(then);
        drive(&keys)
    }

    #[test]
    fn config_screen_opens_on_todays_defaults() {
        // Slice A has no loading yet, so a fresh app carries Configuration::default().
        let app = on_config(&[]);
        assert!(matches!(app.screen(), Screen::Config));
        assert_eq!(app.config().theme(), Theme::Sangue);
        assert_eq!(app.config().audio_volume(), 80);
        assert!(app.config().animations());
        assert_eq!(app.config().language(), Language::default());
    }

    #[test]
    fn right_on_tema_advances_the_theme() {
        let app = on_config(&[KeyPress::Right]);
        assert_eq!(app.config().theme(), Theme::Ambar, "Sangue → Âmbar");
    }

    #[test]
    fn left_on_tema_wraps_to_the_last_theme() {
        let app = on_config(&[KeyPress::Left]);
        assert_eq!(
            app.config().theme(),
            Theme::Fosforo,
            "Sangue ← wraps to Fósforo"
        );
    }

    #[test]
    fn tema_cycles_full_circle() {
        let app = on_config(&[KeyPress::Right, KeyPress::Right, KeyPress::Right]);
        assert_eq!(
            app.config().theme(),
            Theme::Sangue,
            "three steps return to the start"
        );
    }

    #[test]
    fn animacoes_toggles_both_ways() {
        // Down once from `tema` lands on `animações`.
        let off = on_config(&[KeyPress::Down, KeyPress::Right]);
        assert!(!off.config().animations(), "Right turns the effects off");

        let back_on = on_config(&[KeyPress::Down, KeyPress::Right, KeyPress::Left]);
        assert!(back_on.config().animations(), "Left turns them back on");
    }

    #[test]
    fn volume_steps_down_by_ten() {
        // Down twice from `tema` lands on `volume` (default 80).
        let app = on_config(&[KeyPress::Down, KeyPress::Down, KeyPress::Left]);
        assert_eq!(app.config().audio_volume(), 70);
    }

    #[test]
    fn volume_steps_up_by_ten() {
        let app = on_config(&[KeyPress::Down, KeyPress::Down, KeyPress::Right]);
        assert_eq!(app.config().audio_volume(), 90);
    }

    #[test]
    fn volume_clamps_at_the_ceiling() {
        // 80 → 90 → 100 → stays 100. Volume clamps; it must NOT wrap round to 0.
        let app = on_config(&[
            KeyPress::Down,
            KeyPress::Down,
            KeyPress::Right,
            KeyPress::Right,
            KeyPress::Right,
        ]);
        assert_eq!(app.config().audio_volume(), 100);
    }

    #[test]
    fn volume_clamps_at_the_floor() {
        // 80 needs eight Lefts to reach 0; a ninth must stay at 0, not wrap to 100.
        let mut keys = vec![KeyPress::Down, KeyPress::Down];
        keys.extend(vec![KeyPress::Left; 9]);
        let app = on_config(&keys);
        assert_eq!(app.config().audio_volume(), 0);
    }

    #[test]
    fn idioma_cycles_even_though_i18n_is_not_wired_yet() {
        // Down three times from `tema` lands on `idioma`. It's a "dumb" control for
        // now — it changes and persists the value, but nothing retranslates until
        // i18n lands. The chip still moves, so it's feedback, not a silent no-op.
        let app = on_config(&[
            KeyPress::Down,
            KeyPress::Down,
            KeyPress::Down,
            KeyPress::Right,
        ]);
        assert_eq!(app.config().language(), Language::PtBr, " EN-US → PT-BR");
    }

    #[test]
    fn altering_a_value_keeps_you_on_the_config_screen() {
        let app = on_config(&[KeyPress::Right]);
        assert!(
            matches!(app.screen(), Screen::Config),
            "[←→] alters a value, it doesn't navigate away"
        );
    }

    #[test]
    fn navigating_rows_leaves_every_value_untouched() {
        // [↑↓] only move the cursor — they must never change a setting.
        let app = on_config(&[
            KeyPress::Down,
            KeyPress::Down,
            KeyPress::Down,
            KeyPress::Up,
            KeyPress::Up,
        ]);
        assert_eq!(app.config().theme(), Theme::Sangue);
        assert_eq!(app.config().audio_volume(), 80);
        assert!(app.config().animations());
        assert_eq!(app.config().language(), Language::EnUs);
    }

    #[test]
    fn altering_a_row_does_not_move_the_cursor() {
        // Alter `tema` (Right), then Down must reach `animações` and Right toggles
        // IT — proving [←→] changed a value without disturbing the row cursor.
        let app = on_config(&[KeyPress::Right, KeyPress::Down, KeyPress::Right]);
        assert_eq!(app.config().theme(), Theme::Ambar, "the theme step stuck");
        assert!(
            !app.config().animations(),
            "and the cursor still advanced to animações"
        );
    }

    #[test]
    fn left_on_idioma_wraps_to_the_last_language() {
        // The mirror of the Right test — and the case my first spec FORGOT, which
        // is exactly how a bug could hide on the Left-only path.
        let app = on_config(&[
            KeyPress::Down,
            KeyPress::Down,
            KeyPress::Down,
            KeyPress::Left,
        ]);
        assert_eq!(
            app.config().language(),
            Language::EsEs,
            "PT-BR ← wraps to ES-ES"
        );
    }

    #[test]
    fn enter_on_the_config_screen_does_nothing() {
        // Under immediate-apply there is nothing to commit, so Enter must be inert:
        // not a panic, and not a navigation away.
        let app = on_config(&[KeyPress::Enter]);
        assert!(
            matches!(app.screen(), Screen::Config),
            "Enter must not leave the config screen"
        );
        assert_eq!(
            app.config(),
            Configuration::default(),
            "Enter must not alter any setting"
        );
    }

    #[test]
    fn leaving_config_queues_the_changed_config_for_saving() {
        // Change the theme, then Esc back to the menu. The value that rides along
        // must be the live (changed) config, ready for main to persist.
        let mut app = on_config(&[KeyPress::Right, KeyPress::Esc]);
        let live = app.config();

        assert_eq!(
            live.theme(),
            Theme::Ambar,
            "precondition: the change applied"
        );
        assert_eq!(
            app.take_pending_save(),
            Some(live),
            "Esc from config must queue the live config for saving"
        );
    }

    #[test]
    fn leaving_config_unchanged_still_queues_a_save() {
        // We keep no baseline, so we never ask "did anything change?" — every exit
        // writes once. A redundant identical write is the cheap price of not
        // tracking what disk holds.
        let mut app = on_config(&[KeyPress::Esc]);
        let live = app.config();

        assert_eq!(
            app.take_pending_save(),
            Some(live),
            "leaving config always queues a save, changed or not"
        );
    }

    #[test]
    fn take_pending_save_drains_so_the_file_is_written_once() {
        let mut app = on_config(&[KeyPress::Right, KeyPress::Esc]);

        assert!(
            app.take_pending_save().is_some(),
            "the first drain gets the queued config"
        );
        assert_eq!(
            app.take_pending_save(),
            None,
            "the second drain is empty — a visit persists exactly once"
        );
    }

    #[test]
    fn leaving_a_non_config_screen_queues_no_save() {
        // Only the config screen persists. Bouncing out of Info (Menu → Info → Esc)
        // must not queue anything.
        let mut app = drive(&[
            KeyPress::Enter,
            KeyPress::Down,
            KeyPress::Enter, // → Info
            KeyPress::Esc,   // → Menu
        ]);

        assert_eq!(
            app.take_pending_save(),
            None,
            "leaving a screen other than config must not queue a save"
        );
    }

    #[test]
    fn a_fresh_app_has_nothing_queued_to_save() {
        let mut app = drive(&[]);
        assert_eq!(app.take_pending_save(), None);
    }

    // ── G10: Ctrl+C is a door, not a kill ────────────────────────────────────
    // Ctrl+C used to be intercepted in `main::translate_key` and never reached
    // `handle_key`, so quitting from the config screen skipped the exit-write and
    // the visit's edits silently died. Now Ctrl+C is an ordinary `KeyPress`:
    // every screen answers Quit, and the config screen queues its save first —
    // the same promise Esc makes, kept on the impatient exit too. (`main` drains
    // `pending_save` before it honours `Quit`, so queueing is all App must do.)

    #[test]
    fn ctrl_c_quits_from_every_screen() {
        let routes: [(&str, &[KeyPress]); 6] = [
            ("intro", &[]),
            ("menu", &[KeyPress::Enter]),
            ("asking", &[KeyPress::Enter, KeyPress::Enter]),
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

        for (screen, route) in routes {
            let mut keys = route.to_vec();
            keys.push(KeyPress::CtrlC);
            let (_, flow) = drive_flow(&keys);
            assert_eq!(
                flow,
                AppFlow::Quit,
                "Ctrl+C on the {screen} screen must quit"
            );
        }
    }

    #[test]
    fn ctrl_c_from_config_queues_the_changed_config_for_saving() {
        // The hole G10 exists to close: change a value, then quit with Ctrl+C
        // instead of Esc. The edit must ride out with the quit.
        let mut app = on_config(&[KeyPress::Right]);

        let flow = app.handle_key(KeyPress::CtrlC);
        let live = app.config();

        assert_eq!(
            live.theme(),
            Theme::Ambar,
            "precondition: the change applied"
        );
        assert_eq!(flow, AppFlow::Quit);
        assert_eq!(
            app.take_pending_save(),
            Some(live),
            "Ctrl+C from config must queue the live config, exactly like Esc"
        );
    }

    #[test]
    fn ctrl_c_from_config_unchanged_still_queues_a_save() {
        // Same no-baseline policy as Esc: every exit from config writes once,
        // changed or not.
        let mut app = on_config(&[]);
        app.handle_key(KeyPress::CtrlC);

        assert!(
            app.take_pending_save().is_some(),
            "Ctrl+C from config queues a save even when nothing changed"
        );
    }

    #[test]
    fn ctrl_c_outside_config_queues_no_save() {
        // Only a config visit persists. A quit from anywhere else must not
        // write — otherwise every exit would touch the file for no reason.
        let (mut app, flow) = drive_flow(&[KeyPress::Enter, KeyPress::CtrlC]);

        assert_eq!(flow, AppFlow::Quit);
        assert_eq!(
            app.take_pending_save(),
            None,
            "quitting from the menu must not queue a save"
        );
    }

    #[test]
    fn ctrl_c_still_quits_while_sued_is_speaking() {
        // The G8 lock swallows keys while the crawl runs — but the panic button
        // must never be locked. (F5 and Esc pass; Ctrl+C joins them.)
        let mut app = drive(&ASK_AND_REVEAL); // reply clock ticking, crawl unfinished

        let flow = app.handle_key(KeyPress::CtrlC);

        assert_eq!(
            flow,
            AppFlow::Quit,
            "the mid-reveal input lock must not hold Ctrl+C"
        );
    }

    #[test]
    fn enter_with_an_empty_question_earns_no_reply_at_all() {
        // An empty offering is ignored outright: no denial, no reply clock,
        // nothing to play. The taunt is reserved for mortals who actually ask.
        let app = drive(&[KeyPress::Enter, KeyPress::Enter, KeyPress::Enter]);

        match &app.screen {
            Screen::Asking(AskingState { reply, .. }) => {
                assert!(reply.is_none(), "no reply clock started");
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn a_fresh_oracle_has_no_earlier_reply_to_show() {
        // Nothing has been asked yet, so SUED FALA has no previous reply to keep
        // — which is what lets the render show its welcome line instead.
        match drive(&[KeyPress::Enter, KeyPress::Enter]).screen {
            Screen::Asking(asking_state) => {
                assert_eq!(
                    asking_state.previous_reply(),
                    None,
                    "a fresh oracle remembers nothing"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn the_input_stays_locked_while_sued_is_still_talking() {
        // The half of the old rule that survives: mid-crawl, keystrokes are still
        // swallowed. Without the clock rewind the reveal has barely begun, so
        // this is the "still talking" case by construction.
        let mut app = drive(&ASK_AND_REVEAL);

        feed(&mut app, &[KeyPress::Char('x')]);

        match &app.screen {
            Screen::Asking(asking_state) => {
                assert_eq!(
                    asking_state.engine.revealed(),
                    Some("42"),
                    "the engine must still hold the reply it is mid-way through speaking"
                );
                assert_eq!(
                    asking_state.previous_reply(),
                    None,
                    "nothing rotates while SUED is still talking"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn a_finished_reply_reopens_the_input() {
        // The new half: once the crawl ends, the very next keystroke lands in the
        // input instead of being swallowed.
        let mut app = drive(&ASK_AND_REVEAL);
        finish_the_reveal(&mut app);

        feed(&mut app, &[KeyPress::Char('x')]);

        match &app.screen {
            Screen::Asking(AskingState { engine, .. }) => {
                assert_eq!(
                    engine.visible_buffer(),
                    "x",
                    "the keystroke must reach a freshly emptied input"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn starting_the_next_question_keeps_the_answer_on_screen() {
        // The heart of G8. Typing again must NOT blank SUED FALA: the answer
        // moves aside into `previous_reply`, which is what the render keeps
        // showing until a new reply lands.
        let mut app = drive(&ASK_AND_REVEAL);
        finish_the_reveal(&mut app);

        feed(&mut app, &[KeyPress::Char('x')]);

        match &app.screen {
            Screen::Asking(asking_state) => {
                assert_eq!(
                    asking_state.previous_reply(),
                    Some("42"),
                    "the answer must survive the start of the next question"
                );
                assert_eq!(
                    asking_state.engine.revealed(),
                    None,
                    "the engine is re-armed for the new question"
                );
                assert!(
                    asking_state.reply.is_none(),
                    "the reply clock is re-armed too"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn a_finished_denial_is_remembered_the_same_way_as_an_answer() {
        // A taunt is a reply too — it must linger on screen exactly like an
        // answer does, not vanish the moment you start typing again.
        let mut app = drive(&ask_and_be_denied());
        finish_the_reveal(&mut app);

        feed(&mut app, &[KeyPress::Char('x')]);

        let denials = app.config().language().translation().denials;
        match &app.screen {
            Screen::Asking(asking_state) => {
                let kept = asking_state
                    .previous_reply()
                    .expect("the denial must survive the start of the next question");

                assert!(
                    denials.contains(&kept),
                    "the remembered reply must be the denial taunt, got {kept:?}"
                );
                assert!(
                    asking_state.reply.is_none(),
                    "the live denial is cleared once it has been rotated aside"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn the_input_starts_the_next_question_from_scratch() {
        // "Reset the input" means the decoy restarts from its first character —
        // not that it continues where the last question left off. Three hidden
        // keystrokes must therefore paint exactly three decoy chars.
        let mut app = drive(&ASK_AND_REVEAL);
        finish_the_reveal(&mut app);

        feed(
            &mut app,
            &[
                KeyPress::Char(';'), // Hidden
                KeyPress::Char('a'),
                KeyPress::Char('b'),
                KeyPress::Char('c'),
            ],
        );

        let decoys = app.config().language().translation().decoys;
        match &app.screen {
            Screen::Asking(AskingState { engine, .. }) => {
                let visible = engine.visible_buffer();
                assert_eq!(
                    visible.chars().count(),
                    3,
                    "the decoy must restart at its first char, got {visible:?}"
                );
                assert!(
                    decoys.iter().any(|d| d.starts_with(visible)),
                    "the new question must paint a pool decoy from the beginning, got {visible:?}"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn enter_after_a_finished_reply_also_begins_the_next_question() {
        // Enter is an input key like any other: pressing it once SUED has stopped
        // talking starts the next exchange rather than re-firing on the old
        // engine — the answer it replaces is kept. And since the fresh input is
        // empty, this Enter earns nothing: silence, not a taunt.
        let mut app = drive(&ASK_AND_REVEAL);
        finish_the_reveal(&mut app);

        feed(&mut app, &[KeyPress::Enter]);

        match &app.screen {
            Screen::Asking(asking_state) => {
                assert_eq!(
                    asking_state.previous_reply(),
                    Some("42"),
                    "the earlier answer must be kept, not overwritten by the new reply"
                );
                assert!(
                    asking_state.reply.is_none(),
                    "no reply fired: the oracle stays quiet on an empty offering"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn each_new_answer_replaces_the_one_before_it() {
        // Two full exchanges. Only ever ONE earlier reply is kept — this is a
        // rolling last-answer, deliberately not a transcript.
        let mut app = drive(&ASK_AND_REVEAL);
        finish_the_reveal(&mut app);

        feed(
            &mut app,
            &[
                KeyPress::Char(';'), // rotates "42" aside, then Hidden
                KeyPress::Char('9'),
                KeyPress::Char('9'),
                KeyPress::Enter, // reveal "99"
            ],
        );

        match &app.screen {
            Screen::Asking(asking_state) => {
                assert_eq!(
                    asking_state.engine.revealed(),
                    Some("99"),
                    "the new answer is live"
                );
                assert_eq!(
                    asking_state.previous_reply(),
                    Some("42"),
                    "while the new reply is speaking, the old one is still the kept reply"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }

        // ...and once THIS reply finishes and the third question begins, "99"
        // takes "42"'s place. The old answer is gone for good, not stacked.
        finish_the_reveal(&mut app);
        feed(&mut app, &[KeyPress::Char('x')]);

        match &app.screen {
            Screen::Asking(asking_state) => {
                assert_eq!(
                    asking_state.previous_reply(),
                    Some("99"),
                    "only the most recent reply is kept"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn f5_forgets_the_whole_conversation() {
        // F5 stays the hard reset: not just the live reply but the kept one too,
        // so SUED FALA returns to its opening welcome.
        let mut app = drive(&ASK_AND_REVEAL);
        finish_the_reveal(&mut app);
        feed(&mut app, &[KeyPress::Char('x')]); // "42" is now the kept reply

        feed(&mut app, &[KeyPress::F5]);

        match &app.screen {
            Screen::Asking(asking_state) => {
                assert_eq!(
                    asking_state.previous_reply(),
                    None,
                    "F5 must clear the kept reply, or the welcome line never returns"
                );
                assert_eq!(asking_state.engine.revealed(), None);
                assert_eq!(asking_state.engine.visible_buffer(), "");
                assert!(asking_state.reply.is_none());
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn leaving_the_oracle_starts_a_clean_conversation_next_time() {
        // Esc is a door, not a pause: walking back in must not resurrect the
        // last exchange.
        let mut app = drive(&ASK_AND_REVEAL);
        finish_the_reveal(&mut app);
        feed(&mut app, &[KeyPress::Char('x')]); // "42" is now the kept reply

        // ⚠ AMENDED BY G19 — see `leaving_the_oracle_burns_the_transcript_too`.
        feed(
            &mut app,
            &[
                KeyPress::Esc,
                KeyPress::Left,
                KeyPress::Enter, // → Menu
                KeyPress::Enter, // → Asking again
            ],
        );

        match &app.screen {
            Screen::Asking(asking_state) => {
                assert_eq!(
                    asking_state.previous_reply(),
                    None,
                    "a new visit to the oracle starts a new conversation"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    // ── G12 step 2: the séance keeps a record ────────────────────────────────
    // `previous_reply` above is a rolling *view* — one reply, the one still on
    // screen. This is the record: every bubble the audience saw, in the order it
    // happened, starting with SueD's opening greeting. It lives in the
    // `Screen::Asking` payload and dies with the screen (F5 or Esc), so there is
    // no size cap and nothing to prune.
    //
    // ⚠ THE RULE THE WHOLE GIMMICK RESTS ON. A question is recorded from
    // `engine.visible_buffer()` — the DECOY the audience read — and it must be
    // read BEFORE `Enter` reaches the engine, because `handle_enter_key` empties
    // that buffer on its way through. Record the wrong buffer, or the right one
    // too late, and the transcript hands the operator's secret to the first
    // person who scrolls up.
    //
    // These specs name a `Message` type that does not exist yet, so this red
    // phase opens as compile errors rather than failing assertions.

    /// The transcript of the ask screen we are standing on, or a panic if we
    /// aren't standing on one.
    fn transcript(app: &App) -> &[Message] {
        match &app.screen {
            Screen::Asking(AskingState { history, .. }) => history,
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    /// SueD's opening line, in whatever language the app is currently in.
    fn greeting_of(app: &App) -> &'static str {
        app.config().language().translation().ask.welcome_line
    }

    #[test]
    fn a_fresh_oracle_opens_with_sueds_greeting_alone() {
        // The mockup's counter reads `6/6` with the greeting counted, so the
        // transcript starts holding the line the audience has been reading since
        // the first frame — not empty.
        let app = drive(&[KeyPress::Enter, KeyPress::Enter]);

        match transcript(&app) {
            [Message::Sued(greeting)] => assert_eq!(greeting, greeting_of(&app)),
            other => panic!("expected SueD's greeting and nothing else, got {other:?}"),
        }
    }

    #[test]
    fn the_greeting_is_seeded_in_the_active_language() {
        // Same discipline as the G2 pins below: flip idioma first, so a
        // hardcoded Portuguese string cannot pass. The seed must read the active
        // translation, not whichever one happened to be the default.
        let app = ask_in_portuguese(&[]);

        match transcript(&app) {
            [Message::Sued(greeting)] => assert_eq!(
                greeting,
                Language::PtBr.translation().ask.welcome_line,
                "the greeting must be seeded from the live translation"
            ),
            other => panic!("expected SueD's greeting and nothing else, got {other:?}"),
        }
    }

    #[test]
    fn an_answered_exchange_records_the_question_then_the_answer() {
        // Order is the whole point of a transcript: you asked, then it spoke.
        let app = drive(&ASK_AND_REVEAL);

        match transcript(&app) {
            [
                Message::Sued(greeting),
                Message::User(question),
                Message::Sued(answer),
            ] => {
                assert_eq!(greeting, greeting_of(&app));
                assert!(
                    !question.is_empty(),
                    "the question the audience read must be recorded too"
                );
                assert_eq!(answer, "42", "SueD's reply joins the record verbatim");
            }
            other => panic!("expected greeting → question → answer, got {other:?}"),
        }
    }

    #[test]
    fn a_taunt_is_recorded_like_any_other_reply() {
        // A denial is something SueD said out loud in front of the mark, so the
        // transcript must not quietly drop it and disagree with the screen.
        let app = drive(&ask_and_be_denied());
        let denials = app.config().language().translation().denials;

        match transcript(&app) {
            [
                Message::Sued(_),
                Message::User(question),
                Message::Sued(taunt),
            ] => {
                assert_eq!(
                    question, DENIED_QUESTION,
                    "a question asked in the open is recorded exactly as typed"
                );
                assert!(
                    denials.contains(&taunt.as_str()),
                    "the taunt must come from the active language's denial pool, got {taunt:?}"
                );
            }
            other => panic!("expected greeting → question → taunt, got {other:?}"),
        }
    }

    // ── G17 · the short-question rebuke ──────────────────────────────────────
    //
    // A refusal now comes in two flavours, chosen by the LENGTH of what was
    // typed: `<= SHORT_QUESTION_CHARS` earns the rebuke — SueD echoing the
    // question back and pointing at the rule — and anything longer still earns a
    // random line from the `denials` pool.
    //
    // ⚠⚠ THE HAZARD THESE EXIST TO PIN. The length test MUST live INSIDE the
    // `Denied` arm, never before it. `engine.rs:150` branches on
    // `answer_buffer.is_empty()`, NOT on the question — and hidden-mode
    // keystrokes advance the decoy 1:1, so an operator who stages a SHORT secret
    // (`sim`, `42`, a name) leaves a 2–3 character visible buffer. A guard placed
    // ahead of the answer check would rebuke the operator's own setup and throw
    // the staged answer away at the exact moment the prank lands. Short answers
    // are the COMMON case here, not the freak one.
    // `a_short_staged_answer_is_revealed_not_rebuked` is that tripwire.

    /// The words SueD is currently speaking, whatever kind of reply they are.
    fn live_reply(app: &App) -> String {
        match &app.screen {
            Screen::Asking(AskingState {
                reply: Some(reply), ..
            }) => reply.words().to_string(),
            other => panic!("expected Asking with a live reply, got {other:?}"),
        }
    }

    #[test]
    fn a_short_question_is_rebuked_in_sueds_own_words() {
        let app = drive(&ask_and_be_rebuked());
        let translation = app.config().language().translation();

        assert_eq!(
            live_reply(&app),
            translation.rebuke.replace("{question}", REBUKED_QUESTION),
            "a throwaway question must come back quoted, with the rule pointed at \
             — that is the whole feature: it teaches the ritual IN character, \
             which is the one channel G16's out-of-app manual cannot use"
        );
    }

    #[test]
    fn a_long_question_still_earns_a_random_denial() {
        // The other half of the branch. Without this the rebuke could swallow
        // every refusal and the eight-line denial pool would go silently dead.
        let app = drive(&ask_and_be_denied());
        let translation = app.config().language().translation();
        let spoken = live_reply(&app);

        assert!(
            translation.denials.contains(&spoken.as_str()),
            "a question past the threshold must still draw from the denial pool, \
             got {spoken:?}"
        );
        assert!(
            !spoken.contains(DENIED_QUESTION),
            "and it must NOT echo the question — echoing is what marks the rebuke \
             as different from an ordinary refusal"
        );
    }

    #[test]
    fn the_short_question_threshold_is_inclusive() {
        // Both sides derived from the constant, so retuning it moves the test
        // with it instead of breaking it. `<=` is the spec: 18 is short, 19 is
        // not.
        let at_the_line = "x".repeat(SHORT_QUESTION_CHARS);
        let one_past_it = "x".repeat(SHORT_QUESTION_CHARS + 1);

        let rebuked = drive(&ask_openly(&at_the_line));
        let translation = rebuked.config().language().translation();

        assert_eq!(
            live_reply(&rebuked),
            translation.rebuke.replace("{question}", &at_the_line),
            "exactly SHORT_QUESTION_CHARS must still count as short — the bound \
             is inclusive"
        );

        let denied = drive(&ask_openly(&one_past_it));
        assert!(
            translation.denials.contains(&live_reply(&denied).as_str()),
            "one character past the bound must fall through to the denial pool"
        );
    }

    #[test]
    fn a_short_staged_answer_is_revealed_not_rebuked() {
        // ⚠⚠ THE TRIPWIRE FOR THE ONE WAY THIS FEATURE CAN RUIN THE PRANK.
        // The operator stages `42` in hidden mode: three keystrokes, so the
        // DECOY on screen is only a couple of characters long — comfortably
        // inside the rebuke's range. If the length test is ever hoisted above the
        // answer check, this is the case that silently discards the secret and
        // taunts the operator in front of the mark.
        let app = drive(&[
            KeyPress::Enter, // Intro → Menu
            KeyPress::Enter, // Menu → Asking
            KeyPress::Char(';'),
            KeyPress::Char('4'),
            KeyPress::Char('2'),
            KeyPress::Enter,
        ]);

        // The precondition, asserted rather than assumed — otherwise a long decoy
        // would make this pass without ever exercising the hazard.
        match transcript(&app) {
            [_, Message::User(shown), ..] => assert!(
                shown.chars().count() <= SHORT_QUESTION_CHARS,
                "this test is only meaningful while the decoy is SHORT enough to \
                 be rebuked; got {shown:?}, so the tripwire is not armed"
            ),
            other => panic!("expected a question in the transcript, got {other:?}"),
        }

        assert_eq!(
            live_reply(&app),
            "42",
            "a staged answer ALWAYS wins — length may only ever choose which \
             flavour of refusal you get, never whether the secret survives"
        );
    }

    #[test]
    fn a_rebuke_is_recorded_like_any_other_reply() {
        // It is something SueD said out loud in front of the mark, so the
        // transcript must not quietly disagree with the screen.
        let app = drive(&ask_and_be_rebuked());
        let translation = app.config().language().translation();

        match transcript(&app) {
            [
                Message::Sued(_),
                Message::User(question),
                Message::Sued(rebuke),
            ] => {
                assert_eq!(question, REBUKED_QUESTION);
                assert_eq!(
                    rebuke,
                    &translation.rebuke.replace("{question}", REBUKED_QUESTION)
                );
            }
            other => panic!("expected greeting → question → rebuke, got {other:?}"),
        }
    }

    #[test]
    fn every_language_substitutes_the_question_into_its_rebuke() {
        // `{question}` is real substitution, not `{{markup}}` — so the failure
        // this catches is the placeholder reaching the screen as literal text,
        // which no amount of drawing would reveal.
        for language_steps in 0..3 {
            let mut keys = vec![
                KeyPress::Enter,
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
                KeyPress::Esc,
                KeyPress::Up,
                KeyPress::Up,
                KeyPress::Up,
                KeyPress::Enter, // → Asking
            ]);
            keys.extend(typing(REBUKED_QUESTION));
            keys.push(KeyPress::Enter);

            let app = drive(&keys);
            let spoken = live_reply(&app);

            assert!(
                spoken.contains(REBUKED_QUESTION),
                "every language must echo the question back; got {spoken:?}"
            );
            assert!(
                !spoken.contains("{question}"),
                "the placeholder must be SUBSTITUTED, not printed — got {spoken:?}"
            );
        }
    }

    #[test]
    fn no_rebuke_carries_markup_or_loses_its_placeholder() {
        // Two content rules that only a table sweep can hold, both of them
        // invisible until the app is running.
        //
        // ⚠ Markup and the typewriter DO NOT COMPOSE: replies crawl out through
        // `typewriter_reveal`, which reveals a PREFIX, and a prefix of
        // `"foo {{bar}} baz"` is `"foo {{ba"` — broken braces on screen for every
        // frame of the reveal. Same family as markup-cannot-cross-a-newline.
        for language in [Language::PtBr, Language::EnUs, Language::EsEs] {
            let rebuke = language.translation().rebuke;

            assert!(
                !rebuke.contains("{{"),
                "{language:?}'s rebuke must be markup-free — it renders through \
                 the typewriter, which would leave broken braces mid-crawl"
            );
            assert!(
                rebuke.contains("{question}"),
                "{language:?}'s rebuke must keep the placeholder, or the echo \
                 silently vanishes in that language alone"
            );
        }
    }

    #[test]
    fn the_transcript_records_the_decoy_never_the_secret() {
        // THE test. Everything but the Enter, so the decoy can be read off the
        // screen while it still exists.
        let mut app = drive(&ASK_AND_REVEAL[..5]);
        let on_screen = match &app.screen {
            Screen::Asking(AskingState { engine, .. }) => engine.visible_buffer().to_string(),
            other => panic!("expected Asking, got {other:?}"),
        };
        assert_eq!(
            on_screen.chars().count(),
            2,
            "precondition: two hidden keystrokes painted two decoy chars"
        );

        feed(&mut app, &[KeyPress::Enter]);

        match transcript(&app) {
            [_, Message::User(question), _] => {
                // Deliberately a POSITIVE assertion. `!question.contains("42")`
                // alone is satisfied by an EMPTY bubble — which is exactly what
                // reading the buffer after the Enter produces. Move the capture
                // below the forward and this equality is what must fail.
                assert_eq!(
                    question, &on_screen,
                    "the transcript keeps what the AUDIENCE read"
                );
                assert!(
                    !question.contains('4') && !question.contains('2'),
                    "the operator's secret must never reach the transcript, got {question:?}"
                );
            }
            other => panic!("expected greeting → question → answer, got {other:?}"),
        }
    }

    #[test]
    fn no_secret_reaches_the_transcript_across_a_whole_seance() {
        // One exchange proves the happy path. A séance proves the rule holds on
        // every branch — a second question recorded from the wrong buffer would
        // sail straight past the test above.
        let mut app = drive(&ASK_AND_REVEAL);
        finish_the_reveal(&mut app);
        feed(
            &mut app,
            &[
                KeyPress::Char(';'), // rotates the exchange, then Hidden again
                KeyPress::Char('9'),
                KeyPress::Char('9'),
                KeyPress::Enter, // reveal "99"
            ],
        );

        let decoys = app.config().language().translation().decoys;
        match transcript(&app) {
            [
                Message::Sued(_),
                Message::User(first),
                Message::Sued(answer),
                Message::User(second),
                Message::Sued(next_answer),
            ] => {
                assert_eq!(answer, "42");
                assert_eq!(next_answer, "99");
                for question in [first, second] {
                    assert!(
                        decoys.iter().any(|d| d.starts_with(question.as_str())),
                        "every recorded question must be a decoy prefix, got {question:?}"
                    );
                    assert!(
                        !question.contains(['4', '2', '9']),
                        "no secret may survive anywhere in the transcript, got {question:?}"
                    );
                }
            }
            other => panic!("expected two full exchanges after the greeting, got {other:?}"),
        }
    }

    #[test]
    fn an_empty_offering_leaves_no_trace() {
        // The engine ignores an empty Enter outright — no question, no reply, so
        // nothing to record. An empty `You` bubble would be a lie about what was
        // ever on screen.
        let app = drive(&[KeyPress::Enter, KeyPress::Enter, KeyPress::Enter]);

        match transcript(&app) {
            [Message::Sued(_)] => {}
            other => panic!("expected the greeting alone, got {other:?}"),
        }
    }

    #[test]
    fn f5_burns_the_transcript_and_reseeds_the_greeting() {
        // F5 is the panic button: the séance never happened. It clears back to a
        // freshly greeted screen, not to an empty list — the greeting is part of
        // what the audience sees, so it is part of what the record holds.
        let mut app = drive(&ASK_AND_REVEAL);
        finish_the_reveal(&mut app);

        feed(&mut app, &[KeyPress::F5]);

        match transcript(&app) {
            [Message::Sued(greeting)] => assert_eq!(greeting, greeting_of(&app)),
            other => panic!("expected a freshly greeted screen, got {other:?}"),
        }
    }

    #[test]
    fn leaving_the_oracle_burns_the_transcript_too() {
        // The same claim as `leaving_the_oracle_starts_a_clean_conversation_next_time`,
        // now that there is a whole thread to forget rather than one reply. The
        // conversation dies on Esc — which is the reason the transcript can live
        // in the screen payload instead of moving up to `App`.
        let mut app = drive(&ASK_AND_REVEAL);
        finish_the_reveal(&mut app);

        // ⚠ AMENDED BY G19 — Esc raises the confirm, so leaving is now
        // Esc → Left (off the safe default) → Enter. The trailing Enter is the
        // menu's, walking back into the oracle.
        feed(
            &mut app,
            &[
                KeyPress::Esc,
                KeyPress::Left,
                KeyPress::Enter, // → Menu
                KeyPress::Enter, // → Asking again
            ],
        );

        match transcript(&app) {
            [Message::Sued(greeting)] => assert_eq!(greeting, greeting_of(&app)),
            other => panic!("expected a brand-new thread, got {other:?}"),
        }
    }

    // ── G12 step 3: the popover, and who gets the keys ───────────────────────
    // `history_view: Option<HistoryView>` is the popover: `None` closed, `Some`
    // open. `HistoryView { selected }` is the CURSOR in the scrollback — the
    // mockup's `▶` caret, the `6/6` counter and the scrollbar thumb are all read
    // off it, the last two derived rather than stored. It opens on the NEWEST
    // message, because a scrollback opens where the action is.
    //
    // ⚠⚠ THE ROUTING RULE, AND IT IS TRICK-CRITICAL. While the popover is open
    // the keys belong to it and MUST NOT reach the engine — otherwise the
    // operator paints decoy characters into a question nobody can see, behind an
    // overlay. So the popover guard runs BEFORE the G8 conversation guard.
    // `Esc` disambiguates off that same `Option`: close if open, else leave.
    //
    // ⚠ `Up`/`Down` CLAMP here rather than wrapping like the menu does — wrapping
    // from the newest message straight to the greeting reads as a glitch, not as
    // navigation. Both ends are pinned below, and both are load-bearing: `Up`
    // past zero would underflow a `usize`, and `Down` past the end would index
    // out of bounds the moment step 4 renders the selection.
    //
    // These specs name `HistoryView` plus three `KeyPress` variants that do not
    // exist yet (`F1`, `Home`, `End`), so this phase opens as compile errors.

    /// The transcript cursor of the ask screen we are standing on: `None` when
    /// the popover is closed.
    /// The popover's scroll position in rows back from the newest message, or
    /// `None` when it is shut. `Some(0)` is the view F1 opens on.
    fn transcript_scroll(app: &App) -> Option<u16> {
        match &app.screen {
            Screen::Asking(state) => state.transcript().map(HistoryView::rows_from_bottom),
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    /// One COMPLETE exchange: the transcript holds greeting → question → answer
    /// and SueD has stopped speaking, which matters because the popover cannot be
    /// opened while she is still talking (see the last test in this block).
    fn after_one_exchange() -> App {
        let mut app = drive(&ASK_AND_REVEAL);
        finish_the_reveal(&mut app);
        app
    }

    #[test]
    fn f1_opens_the_transcript_on_the_newest_message() {
        // The popover opens on what SueD just said, not on the greeting three
        // bubbles up — and under the bottom anchor that is `0`, not a computed
        // index. "Open at the newest" costing no arithmetic is the whole reason
        // the scroll is stored from the bottom.
        let mut app = after_one_exchange();

        feed(&mut app, &[KeyPress::F1]);

        assert_eq!(
            transcript_scroll(&app),
            Some(0),
            "the popover opens flush with the newest message"
        );
    }

    #[test]
    fn f1_toggles_the_transcript_shut_again() {
        let mut app = after_one_exchange();

        feed(&mut app, &[KeyPress::F1, KeyPress::F1]);

        assert_eq!(transcript_scroll(&app), None, "the same key closes it");
    }

    #[test]
    fn esc_closes_the_transcript_instead_of_leaving_the_oracle() {
        // Esc is overloaded, and the popover wins: closing an overlay is what
        // Esc means everywhere else in the world.
        let mut app = after_one_exchange();

        feed(&mut app, &[KeyPress::F1, KeyPress::Esc]);

        assert_eq!(transcript_scroll(&app), None);
        assert!(
            !on_menu(&app),
            "the first Esc closes the popover, it must not also walk out of the oracle"
        );
    }

    #[test]
    fn esc_leaves_the_oracle_once_the_transcript_is_shut() {
        // ...and the door still works on the second press. Same key, two
        // meanings, disambiguated entirely by the `Option`.
        let mut app = after_one_exchange();

        // ⚠ AMENDED BY G19 — the door now asks first. The third Esc raises the
        // confirm rather than leaving, so walking out costs `Left` (off the safe
        // default) then `Enter`. The claim under test is unchanged: two Escs
        // still mean "close the popover, then head for the door".
        feed(
            &mut app,
            &[
                KeyPress::F1,
                KeyPress::Esc,
                KeyPress::Esc,
                KeyPress::Left,
                KeyPress::Enter,
            ],
        );

        assert!(on_menu(&app));
    }

    #[test]
    fn up_scrolls_back_one_row() {
        // ⚠ One ROW, not one message. The unit changed when the popover became a
        // pager, and a bubble is several rows tall — so a stray `+= 1` per
        // *message* would still pass a "did it move" assertion while scrolling
        // three times too far.
        let mut app = after_one_exchange();

        feed(&mut app, &[KeyPress::F1, KeyPress::Up]);

        assert_eq!(transcript_scroll(&app), Some(1));
    }

    #[test]
    fn scrolling_up_past_the_thread_is_left_for_the_render_to_clamp() {
        // ⚠ THIS ONE PINS A DIVISION OF LABOUR, not an arithmetic result, so read
        // it before "fixing" it. `HistoryView` does not know how tall the
        // transcript is — that needs the rendered bubble heights, which only
        // exist inside the render. So `handle_up` is deliberately UNBOUNDED, and
        // `scroll_offset` saturates the excess away at draw time.
        //
        // The cost of that split is real and known: over-scrolling banks dead
        // keypresses that Down has to spend before anything moves on screen. It
        // is recorded here so the day it gets clamped is a decision rather than
        // an accident.
        let mut app = after_one_exchange();

        feed(
            &mut app,
            &[
                KeyPress::F1,
                KeyPress::Up,
                KeyPress::Up,
                KeyPress::Up,
                KeyPress::Up,
                KeyPress::Up,
            ],
        );

        assert_eq!(
            transcript_scroll(&app),
            Some(5),
            "the view counts rows; the render decides which of them exist"
        );
    }

    #[test]
    fn down_clamps_at_the_newest_end() {
        // The popover already opens flush with the newest message, so Down has
        // nowhere to go. This is the one direction the view CAN clamp on its own
        // — `0` is a bound it knows without measuring anything — and a raw
        // `- 1` here panics in debug and wraps to ~65535 in release.
        let mut app = after_one_exchange();

        feed(&mut app, &[KeyPress::F1, KeyPress::Down, KeyPress::Down]);

        assert_eq!(transcript_scroll(&app), Some(0));
    }

    #[test]
    fn page_up_and_page_down_move_by_a_page() {
        // Derived from `PAGE_ROWS` rather than written as a literal, so retuning
        // the jump by eye once the popover draws does not turn this red.
        let mut app = after_one_exchange();

        feed(&mut app, &[KeyPress::F1, KeyPress::PageUp]);
        assert_eq!(
            transcript_scroll(&app),
            Some(PAGE_ROWS),
            "PgUp jumps a page back, not a row"
        );

        feed(&mut app, &[KeyPress::PageDown]);
        assert_eq!(
            transcript_scroll(&app),
            Some(0),
            "PgDn brings the same page back"
        );
    }

    #[test]
    fn page_down_clamps_at_the_newest_end() {
        // Same bound as Down, and the same underflow: a page-sized subtraction
        // from `0` is a bigger wrap than a single row, not a different bug.
        let mut app = after_one_exchange();

        feed(&mut app, &[KeyPress::F1, KeyPress::PageDown]);

        assert_eq!(transcript_scroll(&app), Some(0));
    }

    #[test]
    fn the_engine_never_sees_a_key_while_the_transcript_is_open() {
        // ⚠ THE TRICK-CRITICAL ONE. Keys that leak past an open popover paint
        // decoy characters into an input the operator cannot see, and an Enter
        // that leaks asks a question they never finished typing. Nothing below
        // may reach the engine, and the transcript itself must not grow.
        let mut app = after_one_exchange();

        feed(
            &mut app,
            &[
                KeyPress::F1,
                KeyPress::Char('x'),
                KeyPress::Char(';'), // the mode toggle, of all keys, must not land
                KeyPress::Char('4'),
                KeyPress::Enter,
                KeyPress::Backspace,
            ],
        );

        assert_eq!(
            transcript_scroll(&app),
            Some(0),
            "precondition: the popover is still open, so these keys were its own"
        );
        match &app.screen {
            Screen::Asking(AskingState {
                engine,
                reply,
                history,
                ..
            }) => {
                assert_eq!(
                    engine.visible_buffer(),
                    "",
                    "not one keystroke may reach the input behind the overlay"
                );
                assert!(
                    reply.is_none(),
                    "a leaked Enter would have asked a question"
                );
                assert_eq!(
                    history.len(),
                    3,
                    "the transcript must not grow while you are reading it"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    // ⚠ The two tests above and below this line look like they overlap, and they
    // do not. `after_one_exchange` leaves SueD finished speaking, so the opening
    // `F1` trips the G8 rotation on its way in and the engine is freshly reset —
    // which means the test above meets `Enter` with an EMPTY buffer, where the
    // engine's own no-op-on-empty rule swallows it regardless of the guard.
    //
    // These two open the popover MID-TYPING instead. That is the only state in
    // which a leak has anything to destroy, and it is the state the operator is
    // actually in when they reach for the transcript.

    #[test]
    fn enter_cannot_ask_a_question_from_behind_the_transcript() {
        // A leaked `Enter` does not merely reach the engine: it stamps `*reply`,
        // so `main`'s tick loop fires the reply sting and THE MARK HEARS SUED
        // ANSWER while the operator is still reading. It also pushes two
        // messages into `history` while the popover is displaying it.
        let mut app = drive(&[
            KeyPress::Enter,     // Intro → Menu
            KeyPress::Enter,     // Menu → Asking
            KeyPress::Char(';'), // Hidden
            KeyPress::Char('4'),
            KeyPress::Char('2'), // the secret answer, half-typed
        ]);

        match &app.screen {
            Screen::Asking(AskingState {
                engine, history, ..
            }) => {
                assert!(
                    !engine.visible_buffer().is_empty(),
                    "precondition: there must be a half-typed question to protect"
                );
                assert_eq!(history.len(), 1, "precondition: only the greeting so far");
            }
            other => panic!("expected Asking, got {other:?}"),
        }

        feed(&mut app, &[KeyPress::F1, KeyPress::Enter]);

        match &app.screen {
            Screen::Asking(AskingState {
                reply,
                history,
                overlay,
                ..
            }) => {
                assert!(
                    overlay.is_some(),
                    "precondition: the popover is still open, so that Enter was its own"
                );
                assert!(
                    reply.is_none(),
                    "a leaked Enter asked a question nobody finished typing"
                );
                assert_eq!(history.len(), 1, "the transcript grew behind the overlay");
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn backspace_cannot_eat_the_secret_answer_from_behind_the_transcript() {
        // In hidden mode `handle_backspace_key` pops `answer_buffer` and rewinds
        // the decoy cursor, so a leaked Backspace deletes a character of the
        // OPERATOR'S REAL ANSWER — silently, behind an overlay, with no feedback
        // until they hit Enter and SueD says the wrong thing.
        let mut app = drive(&[
            KeyPress::Enter,
            KeyPress::Enter,
            KeyPress::Char(';'),
            KeyPress::Char('4'),
            KeyPress::Char('2'),
        ]);

        let before = match &app.screen {
            Screen::Asking(AskingState { engine, .. }) => {
                assert!(
                    !engine.visible_buffer().is_empty(),
                    "precondition: there must be a character to lose"
                );
                engine.visible_buffer().to_string()
            }
            other => panic!("expected Asking, got {other:?}"),
        };

        feed(&mut app, &[KeyPress::F1, KeyPress::Backspace]);

        match &app.screen {
            Screen::Asking(AskingState { engine, .. }) => assert_eq!(
                engine.visible_buffer(),
                before,
                "Backspace reached the input hidden behind the overlay"
            ),
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn ctrl_c_still_quits_with_the_transcript_open() {
        // The panic button is never locked — same rule as the G8 mid-reveal lock.
        let mut app = after_one_exchange();
        feed(&mut app, &[KeyPress::F1]);

        let flow = app.handle_key(KeyPress::CtrlC);

        assert_eq!(flow, AppFlow::Quit);
    }

    #[test]
    fn f5_shuts_the_transcript_and_burns_the_thread() {
        // F5 is still the hard reset, and it reaches through the popover: a fresh
        // screen has nothing open and nothing to read.
        let mut app = after_one_exchange();

        feed(&mut app, &[KeyPress::F1, KeyPress::F5]);

        assert_eq!(transcript_scroll(&app), None);
        match transcript(&app) {
            [Message::Sued(greeting)] => assert_eq!(greeting, greeting_of(&app)),
            other => panic!("expected a freshly greeted screen, got {other:?}"),
        }
    }

    #[test]
    fn f1_is_locked_out_while_sued_is_still_speaking() {
        // Note the missing `finish_the_reveal`: SueD is mid-crawl, so the G8 lock
        // swallows F1 exactly as it swallows typing. This is what makes recording
        // the reply at Enter safe — the message is in `history` from that moment,
        // but nobody can open the popover to read it until it has actually been
        // spoken on screen.
        let mut app = drive(&ASK_AND_REVEAL);

        feed(&mut app, &[KeyPress::F1]);

        assert_eq!(
            transcript_scroll(&app),
            None,
            "the transcript stays shut until SueD stops talking"
        );
    }

    // ── G19 · Esc confirms before it burns the séance ────────────────────────
    //
    // `history` dies with the screen by design (G12's call — it is a séance, not
    // a log), and `Esc → Menu` destroys it with no warning and no undo. G12 made
    // that loss VISIBLE, which is what created the obligation to guard it.
    //
    // ⚠ THE LAYERING IS THE WHOLE SPEC. `Esc` now means three different things
    // depending on what is up, and the failure mode is that it quietly does two
    // of them at once. Each of the three is pinned separately below.
    //
    // ⚠⚠ THESE OPEN AS COMPILE ERRORS. They name `translation.confirm`, which
    // does not exist yet — deliberately, see `confirm_keys`. Adding it is the
    // first move.

    /// `true` when the leave-confirmation dialog is the overlay standing up.
    ///
    /// ⚠ `ConfirmLeave { .. }` and not `ConfirmLeave` — the `{ .. }` form matches
    /// a unit, tuple OR struct variant, so this helper survives you deciding how
    /// the dialog carries its selection without every test in the block caring.
    fn on_confirm(app: &App) -> bool {
        matches!(
            app.screen(),
            Screen::Asking(AskingState {
                overlay: Some(Overlay::ConfirmLeave { .. }),
                ..
            })
        )
    }

    #[test]
    fn esc_with_the_transcript_open_does_not_also_raise_the_confirm() {
        // Layer 1. Esc closes the transcript and STOPS — it must not spend the
        // same keystroke on the door as well.
        //
        // ⚠ This assertion is the reason this test exists next to
        // `esc_closes_the_transcript_instead_of_leaving_the_oracle`, which looks
        // like it already covers this and does not: that one checks the
        // transcript is shut and we are not on the menu, and BOTH of those stay
        // true if Esc wrongly raises the confirm. It would pass through the bug.
        let mut app = after_one_exchange();

        feed(&mut app, &[KeyPress::F1, KeyPress::Esc]);

        assert_eq!(transcript_scroll(&app), None, "the transcript closed");
        assert!(
            !on_confirm(&app),
            "closing an overlay is not leaving — the confirm must stay down"
        );
    }

    #[test]
    fn esc_after_an_exchange_raises_the_confirm_instead_of_leaving() {
        // Layer 2. Something has been said, so the door asks first.
        let mut app = after_one_exchange();

        feed(&mut app, &[KeyPress::Esc]);

        assert!(on_confirm(&app), "the veil asks before it closes");
        assert!(
            !on_menu(&app),
            "raising the prompt must not ALSO walk out — that is the whole point"
        );
    }

    #[test]
    fn esc_with_nothing_asked_walks_out_without_a_prompt() {
        // Layer 3. Nothing to mourn, so no ceremony: the door behaves exactly as
        // it did before G19 existed.
        //
        // ⚠ The predicate is "has the mark ever spoken", NOT "are there messages"
        // — `history` is always seeded with the greeting, so a length check is
        // never zero and would prompt on a screen where nothing has happened.
        let mut app = drive(&[KeyPress::Enter, KeyPress::Enter]);

        feed(&mut app, &[KeyPress::Esc]);

        assert!(on_menu(&app), "an untouched séance has nothing to lose");
    }

    #[test]
    fn a_question_that_earned_only_a_taunt_still_counts_as_having_spoken() {
        // ⚠ There are TWO ways to put a `Message::User` in the transcript, and a
        // predicate written against the reveal path alone would miss this one: a
        // question with no staged answer is DENIED, and the denial arm pushes the
        // question and the taunt exactly like the reveal arm does.
        //
        // It matters because the taunt path is the one the mark actually walks
        // when the operator has not armed anything yet — losing that exchange is
        // still losing an exchange.
        let mut app = drive(&ask_and_be_denied());
        finish_the_reveal(&mut app);

        feed(&mut app, &[KeyPress::Esc]);

        assert!(
            on_confirm(&app),
            "being refused is still a conversation worth guarding"
        );
    }

    #[test]
    fn a_reflexive_enter_cannot_burn_the_seance() {
        // ⚠⚠ THE SAFETY PROPERTY OF THE WHOLE FEATURE, and the reason the dialog
        // has to carry a selection at all. `Enter` is bound now, and mid-
        // performance the operator hammers it — so the button standing under the
        // cursor when the dialog opens must be the HARMLESS one. Destructive
        // actions do not get to be the default.
        //
        // ⚠ This contradicts `design-refs/03-c-confirm-leave.png`, which
        // highlights QUE ASSIM SEJA. If you keep the mockup's default, this is
        // the test to delete — but delete it deliberately, not by flipping a bool
        // until the bar goes green.
        let mut app = after_one_exchange();

        feed(&mut app, &[KeyPress::Esc, KeyPress::Enter]);

        assert!(
            !on_menu(&app),
            "an untouched Enter must never be the one that closes the veil"
        );
        assert_eq!(
            transcript(&app).len(),
            3,
            "and the séance is still whole behind it"
        );
    }

    #[test]
    fn choosing_the_other_option_and_confirming_burns_the_seance() {
        // The door still works — it just costs a deliberate move first.
        //
        // ⚠ `Left` because the mockup puts the leave option on the LEFT and the
        // safe one on the right. If that order changes, this key changes with it.
        let mut app = after_one_exchange();

        feed(&mut app, &[KeyPress::Esc, KeyPress::Left, KeyPress::Enter]);

        assert!(on_menu(&app), "chosen deliberately, the veil closes");
    }

    #[test]
    fn the_selection_survives_being_moved_back_and_forth() {
        // ←→ is a two-item toggle, not a counter: landing back where you started
        // must mean what it meant when you started. A selection stored as an
        // index that keeps incrementing would pass the test above and fail this
        // one.
        let mut app = after_one_exchange();

        feed(&mut app, &[KeyPress::Esc]);
        assert!(on_confirm(&app), "precondition: the dialog is up");

        feed(
            &mut app,
            &[KeyPress::Left, KeyPress::Right, KeyPress::Enter],
        );

        assert!(
            !on_menu(&app),
            "back on the safe option, Enter must be harmless again"
        );
    }

    #[test]
    fn cancelling_leaves_the_seance_intact() {
        // A refusal has to put you back exactly where you were: overlay down,
        // still in the oracle, and — the part worth pinning — with the transcript
        // still holding every message. A "cancel" that silently wiped the history
        // would pass a naive on-screen check.
        let mut app = after_one_exchange();

        feed(&mut app, &[KeyPress::Esc, KeyPress::Esc]);

        assert!(!on_confirm(&app), "the prompt is dismissed");
        assert!(!on_menu(&app), "and we did NOT leave");
        assert_eq!(
            transcript(&app).len(),
            3,
            "staying must cost the séance nothing"
        );
    }

    #[test]
    fn the_confirm_swallows_every_key_that_would_reach_the_engine() {
        // ✅ TRICK SAFETY, and it is the reason a modal on Esc is acceptable at
        // all. While the dialog is up the operator must not be able to paint
        // decoy characters into a question hidden behind it — the same rule the
        // transcript popover already obeys.
        let mut app = after_one_exchange();

        feed(
            &mut app,
            &[
                KeyPress::Esc,
                KeyPress::Char('x'),
                KeyPress::Char(';'), // the hidden-mode toggle is not exempt
                KeyPress::Char('9'),
                KeyPress::Backspace,
                // ⚠ `Enter` is deliberately NOT in this list. It belongs to the
                // dialog now — it commits the choice — so feeding it here would
                // dismiss the very overlay this test is standing behind. That a
                // leaked `Enter` cannot ask a question is pinned separately, by
                // `a_reflexive_enter_cannot_burn_the_seance`.
            ],
        );

        assert!(
            on_confirm(&app),
            "precondition: the dialog is still up, so those keys were its own"
        );
        match &app.screen {
            Screen::Asking(AskingState {
                engine, history, ..
            }) => {
                assert_eq!(
                    engine.visible_buffer(),
                    "",
                    "not one keystroke may reach the input behind the dialog"
                );
                assert_eq!(
                    history.len(),
                    3,
                    "a leaked Enter would have asked a question"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn f1_cannot_open_the_transcript_from_behind_the_confirm() {
        // ⚠ The allowed-key set is PER-VARIANT, and this is the test that says
        // so. Today's swallow guard lets F1/arrows/PgUp/PgDn through on
        // `overlay.is_some()` because the transcript was the only overlay there
        // was. Left alone, F1 would stack a second overlay on top of the dialog —
        // the exact illegal state `Option<Overlay>` exists to forbid.
        let mut app = after_one_exchange();

        feed(&mut app, &[KeyPress::Esc, KeyPress::F1]);

        assert_eq!(
            transcript_scroll(&app),
            None,
            "the dialog owns the keys while it is up"
        );
        assert!(on_confirm(&app), "and it is still the overlay standing");
    }

    #[test]
    fn ctrl_c_still_quits_from_behind_the_confirm() {
        // The panic buttons are never locked — that is precisely what makes it
        // safe to put a modal on Esc. Esc is deliberate navigation; Ctrl-C is the
        // escape hatch, and an escape hatch that asks a question is not one.
        // ⚠ The precondition is the whole test. Without it this passes TODAY for
        // the wrong reason: Esc currently walks straight out, so Ctrl-C would be
        // quitting from the MENU and the assertion below would never once have
        // seen the dialog. Same trap `assert_popover_is_open` exists to close.
        let mut app = after_one_exchange();
        feed(&mut app, &[KeyPress::Esc]);
        assert!(on_confirm(&app), "precondition: the dialog is up");

        let flow = app.handle_key(KeyPress::CtrlC);

        assert_eq!(flow, AppFlow::Quit);
    }

    #[test]
    fn f5_still_burns_the_seance_from_behind_the_confirm() {
        // The other panic button. F5 reaches through the dialog and hard-resets,
        // dialog and all — no prompt, because F5's entire job is to be instant.
        let mut app = after_one_exchange();

        feed(&mut app, &[KeyPress::Esc, KeyPress::F5]);

        assert!(!on_confirm(&app), "the reset takes the dialog with it");
        match transcript(&app) {
            [Message::Sued(greeting)] => assert_eq!(greeting, greeting_of(&app)),
            other => panic!("expected a freshly greeted screen, got {other:?}"),
        }
    }

    // ── G2 wiring: SUED's words come from the language pools ─────────────────
    // Decoys and denials are drawn from `Language::translation()` with a random
    // roll at the app edge — so these specs assert pool *membership*, never
    // which entry won the draw. Every pin flips idioma to PT-BR first: the
    // language flip is the discriminator. A prefix-length probe alone proved
    // gameable — the PT pool's first decoy grew out of the old constant, so
    // fixing the constant's typo satisfied "prefix of some pool entry" with no
    // wiring at all. No hardcoded English string can pass these.

    /// Drive a fresh app onto the Ask screen with `idioma` flipped to PT-BR
    /// first, then apply `then`. Config is 3 Downs from the top of the menu;
    /// `idioma` is 3 Downs from the top of the config rows; the menu cursor is
    /// still on Config when we Esc back out.
    fn ask_in_portuguese(then: &[KeyPress]) -> App {
        let mut keys = vec![
            KeyPress::Enter, // Intro → Menu
            KeyPress::Down,
            KeyPress::Down,
            KeyPress::Down,  // → Config row
            KeyPress::Enter, // → Screen::Config, cursor on `tema`
            KeyPress::Down,
            KeyPress::Down,
            KeyPress::Down,  // → `idioma`
            KeyPress::Right, // EN-US → PT-BR
            KeyPress::Esc,   // → Menu (cursor on Config)
            KeyPress::Up,
            KeyPress::Up,
            KeyPress::Up,    // → Ask row
            KeyPress::Enter, // → Asking
        ];
        keys.extend_from_slice(then);
        let app = drive(&keys);
        assert_eq!(
            app.config().language(),
            Language::PtBr,
            "precondition: the idioma flip must have landed"
        );
        app
    }

    #[test]
    fn a_new_question_draws_its_decoy_from_the_language_pool() {
        let mut hidden_typing = vec![KeyPress::Char(';')];
        hidden_typing.extend(std::iter::repeat_n(KeyPress::Char('x'), 50));
        let app = ask_in_portuguese(&hidden_typing);

        match &app.screen {
            Screen::Asking(AskingState { engine, .. }) => {
                let visible = engine.visible_buffer();
                assert_eq!(visible.chars().count(), 50, "one decoy char per keystroke");
                assert!(
                    Language::PtBr
                        .translation()
                        .decoys
                        .iter()
                        .any(|d| d.starts_with(visible)),
                    "the painted decoy must be an entry of the active language's pool, \
                     got {visible:?}"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn f5_re_arms_the_decoy_from_the_start_of_a_pool_entry() {
        // F5's re-arm lives app-side (the engine's F5 is inert): the fresh
        // exchange must paint a pool decoy from its first character — a stale
        // decoy cursor would paint mid-string and break the illusion.
        let mut app = ask_in_portuguese(&[
            KeyPress::Char(';'), // Hidden
            KeyPress::Char('4'),
            KeyPress::Char('2'), // secret answer "42"
            KeyPress::Enter,     // reveal
        ]);
        feed(&mut app, &[KeyPress::F5, KeyPress::Char(';')]);
        feed(&mut app, &[KeyPress::Char('x'); 45]);

        match &app.screen {
            Screen::Asking(AskingState { engine, .. }) => {
                let visible = engine.visible_buffer();
                assert_eq!(visible.chars().count(), 45, "one decoy char per keystroke");
                assert!(
                    Language::PtBr
                        .translation()
                        .decoys
                        .iter()
                        .any(|d| d.starts_with(visible)),
                    "after F5 the decoy must restart from the top of a pool entry, \
                     got {visible:?}"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn a_denial_speaks_the_configured_language() {
        // ⚠ The question must clear `SHORT_QUESTION_CHARS` or this stops testing
        // what it claims: a rebuke refuses in the configured language too, so it
        // would pass while covering the wrong branch. The constant is guarded by
        // `the_fixtures_actually_straddle_the_threshold`.
        let mut state = ask_in_portuguese(&typing(DENIED_QUESTION_PT));

        state.handle_key(KeyPress::Enter);

        match state.screen {
            Screen::Asking(AskingState {
                reply: Some(reply), ..
            }) => {
                let taunt = reply.words();

                assert!(
                    Language::PtBr.translation().denials.contains(&taunt),
                    "the oracle must taunt in the configured language, got {taunt:?}"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    // ── G13 · the ponder, and what replaced the cue tests ────────────────────
    //
    // Four tests died with the `pending_cue` seam (`a_reveal_queues_the_jump_scare_cue`,
    // `a_denial_queues_the_jump_scare_cue`, `take_cue_drains_so_the_sound_fires_once`,
    // `plain_typing_queues_no_cue`). Their CLAIMS all survived — the sting still
    // fires, for both outcomes, exactly once, and never before a reply — but the
    // mechanism moved to `main`'s falling-edge check on `is_pondering()`. These
    // pin the app-side half, which is the half that decides WHEN the edge happens.
    //
    // ⚠ Worth remembering why the fourth one had to go rather than being kept:
    // `plain_typing_queues_no_cue` asserted `take_cue() == None` and kept PASSING
    // after the seam was gutted, because a negative assertion goes vacuous once
    // its subject disappears. The three that broke did their job; the one that
    // survived had quietly stopped testing anything.

    #[test]
    fn a_reveal_makes_sued_ponder_before_it_speaks() {
        let app = drive(&ASK_AND_REVEAL);
        assert!(
            app.is_pondering(),
            "the reveal must open with a ponder — without it the sting fires at \
             Enter and SueD answers instantly, which is what G13 exists to stop"
        );
    }

    #[test]
    fn a_denial_makes_sued_ponder_before_it_speaks() {
        // The ponder is NOT reveal-only (Danilo's call 2026-07-27): SueD weighing
        // a mortal before rejecting them sells the seance better than an instant
        // refusal, and it keeps one clock rule instead of two.
        let app = drive(&ask_and_be_denied());
        assert!(
            app.is_pondering(),
            "a denial must ponder too, or denials skip the pause and the sting \
             fires early for exactly one of the two outcomes"
        );
    }

    #[test]
    fn a_rebuke_lands_instantly_because_nothing_was_consulted() {
        // ⚠ THE LINE THIS DRAWS, and it is narrower than "the ponder is for
        // answers". A DENIAL still ponders and must keep doing so (the test above
        // pins it): SueD heard the question, weighed it, and found it beneath him
        // — a consultation happened, so the pause and the spell are earned.
        //
        // A REBUKE is the one refusal that answers the RITUAL rather than the
        // question. You did not flatter him, so he never went looking: there is
        // nothing to dig out of the dark, no incantation to cast, and no reason
        // to wait. Instant contempt is the character.
        //
        // ⚠⚠ AND THE PRECONDITION IS NOT DECORATION. `App::is_pondering()` is
        // `false` whenever there is no reply AT ALL (`app.rs:643`), so the
        // assertion below would pass just as happily on a rebuke that never
        // fired — the most likely way for this to rot into a test of nothing.
        let app = drive(&ask_and_be_rebuked());
        let translation = app.config().language().translation();

        assert_eq!(
            live_reply(&app),
            translation.rebuke.replace("{question}", REBUKED_QUESTION),
            "precondition: a rebuke must actually have been spoken, or the ponder \
             assertion below is vacuously true"
        );

        assert!(
            !app.is_pondering(),
            "a rebuke must land with no pause — SueD consulted nothing, so there \
             is no spell to cast and nothing to wait for"
        );
    }

    #[test]
    fn a_rebuke_fires_no_sting_because_there_is_no_falling_edge() {
        // The consequence of the test above, written down where it can be seen.
        // `main.rs:133` plays `JumpScare` on the FALLING EDGE of `is_pondering()`
        // — `was_pondering && !pondering_now`. A rebuke never ponders, so the
        // edge never happens and no sting plays. That is deliberate: the sting
        // belongs to SueD *arriving with an answer*, not to him brushing you off.
        //
        // Pinned app-side because `main`'s loop cannot be unit-tested — what this
        // guards is the CAUSE (never pondering, therefore never a falling edge),
        // which is the half that lives here.
        let mut app = drive(&ask_and_be_rebuked());

        assert!(
            !app.is_pondering(),
            "frame one: already not pondering, so there is no rising edge either"
        );

        // Wind the clock forward across the window a ponder would have occupied.
        // If it never becomes true, it can never fall.
        for _ in 0..10 {
            app.rewind_reply(Duration::from_millis(500));
            assert!(
                !app.is_pondering(),
                "a rebuke must never enter the pondering state at any point, or a \
                 falling edge appears later and the sting fires on a brush-off"
            );
        }
    }

    #[test]
    fn typing_without_asking_never_ponders() {
        // No reply clock, no ponder, no falling edge — so nothing can play before
        // a question is actually offered.
        let app = drive(&[
            KeyPress::Enter,
            KeyPress::Enter, // → Asking
            KeyPress::Char('o'),
            KeyPress::Char('i'), // typed, but never submitted
        ]);
        assert!(
            !app.is_pondering(),
            "no question asked yet — nothing to ponder"
        );
    }

    #[test]
    fn the_ponder_ends_so_the_sting_has_a_falling_edge() {
        // The other half of the pair: `is_pondering` must eventually go false, or
        // `main`'s `was_pondering && !pondering_now` never fires and the reply is
        // silent forever. "Always pondering" would satisfy the two pins above.
        let mut app = drive(&ASK_AND_REVEAL);
        assert!(app.is_pondering(), "precondition: the ponder started");

        finish_the_reveal(&mut app);

        assert!(
            !app.is_pondering(),
            "the ponder must end — otherwise the sting never gets its edge"
        );
    }

    // ── G14 · thunder at decoy exhaustion ────────────────────────────────────
    //
    // ⚠ THE SEAM IS `pending_cue` REVIVED, AND THAT IS CORRECT HERE FOR THE
    // EXACT REASON IT FAILED FOR G13. G13's ponder→speak transition happens on a
    // TIMER with no keypress, so a queue drained inside `main`'s keypress block
    // could never see it — which is why that seam was deleted. G14's threshold
    // crossing is the opposite: it happens *inside* `handle_key`, caused by a
    // keystroke. A keypress-driven event wants a keypress-drained queue.
    //
    // Shape these pin: `Engine::decoy_chars_remaining()` (pure fact) +
    // `THUNDER_AT_CHARS_REMAINING` (policy) + `thunder_spent` on `Screen::Asking`
    // (per-decoy memory, re-armed wherever `engine.reset` is called) +
    // `App::pending_cue`/`take_pending_cue` (mirrors `pending_save`).

    /// Menu → Asking → Hidden, ready to burn decoy.
    const ASK_IN_HIDDEN: [KeyPress; 3] = [
        KeyPress::Enter,     // Intro → Menu
        KeyPress::Enter,     // Menu → Asking
        KeyPress::Char(';'), // → Hidden
    ];

    /// The ask screen's engine, for tests that need to read the decoy down.
    fn engine_of(app: &App) -> &Engine {
        match &app.screen {
            Screen::Asking(AskingState { engine, .. }) => engine,
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    /// Type hidden chars until the decoy is exactly `remaining` from spent.
    ///
    /// Driven by the engine's own count rather than a fixed keystroke total,
    /// because the decoy is picked at random from a pool of 86–113 char strings
    /// — no constant would land on the threshold for every roll.
    fn type_hidden_until_remaining(app: &mut App, remaining: usize) {
        for _ in 0..500 {
            if engine_of(app).decoy_chars_remaining() <= remaining {
                return;
            }
            app.handle_key(KeyPress::Char('x'));
        }
        panic!("never reached {remaining} remaining — is the app still in Hidden mode?");
    }

    #[test]
    fn crossing_the_threshold_queues_the_thunder() {
        let mut app = drive(&ASK_IN_HIDDEN);

        type_hidden_until_remaining(&mut app, THUNDER_AT_CHARS_REMAINING);

        assert_eq!(
            app.take_pending_cue(),
            Some(AudioCue::Thunder),
            "at {THUNDER_AT_CHARS_REMAINING} chars left the operator must be told \
             to land the answer — today the fake question just stops growing \
             mid-performance and nothing warns anyone"
        );
    }

    #[test]
    fn nothing_is_queued_while_the_decoy_is_still_comfortable() {
        let mut app = drive(&ASK_IN_HIDDEN);

        type_hidden_until_remaining(&mut app, THUNDER_AT_CHARS_REMAINING + 5);

        assert_eq!(
            app.take_pending_cue(),
            None,
            "still five chars of runway beyond the line — warning this early \
             trains the operator to ignore it"
        );
    }

    #[test]
    fn the_thunder_strikes_once_per_decoy_not_once_per_keystroke() {
        // ⚠ EDGE-triggered, not level-triggered. `remaining <= threshold` stays
        // true for every keystroke after the crossing, so a plain level test
        // would fire the sting on all twenty of the remaining keys — a stuck
        // alarm rather than a warning. Same trap as `is_pondering` in G13.
        let mut app = drive(&ASK_IN_HIDDEN);
        type_hidden_until_remaining(&mut app, THUNDER_AT_CHARS_REMAINING);
        assert!(
            app.take_pending_cue().is_some(),
            "precondition: it struck once on the crossing"
        );

        app.handle_key(KeyPress::Char('x'));
        app.handle_key(KeyPress::Char('x'));

        assert_eq!(
            app.take_pending_cue(),
            None,
            "one thunder per decoy — his call, 2026-07-28"
        );
    }

    #[test]
    fn backspacing_back_over_the_line_does_not_re_arm_the_thunder() {
        // Danilo's call 2026-07-28: STAY SPENT. Re-arming would let a nervous
        // operator retrigger the warning repeatedly mid-performance, which is
        // exactly when the room is listening.
        let mut app = drive(&ASK_IN_HIDDEN);
        type_hidden_until_remaining(&mut app, THUNDER_AT_CHARS_REMAINING);
        app.take_pending_cue(); // drain the first strike

        app.handle_key(KeyPress::Backspace); // back above the line
        app.handle_key(KeyPress::Char('x')); // and across it a second time

        assert_eq!(
            app.take_pending_cue(),
            None,
            "the warning is spent for this decoy — only a NEW decoy re-arms it"
        );
    }

    #[test]
    fn f5_starts_a_new_decoy_and_re_arms_the_thunder() {
        let mut app = drive(&ASK_IN_HIDDEN);
        type_hidden_until_remaining(&mut app, THUNDER_AT_CHARS_REMAINING);
        app.take_pending_cue();

        app.handle_key(KeyPress::F5); // panic button → brand-new decoy
        app.handle_key(KeyPress::Char(';')); // reset leaves Mode::Normal, so re-enter Hidden
        type_hidden_until_remaining(&mut app, THUNDER_AT_CHARS_REMAINING);

        assert_eq!(
            app.take_pending_cue(),
            Some(AudioCue::Thunder),
            "a fresh decoy owes a fresh warning — which is why re-arming belongs \
             wherever `engine.reset` is called, not to the keystroke that crossed"
        );
    }

    #[test]
    fn a_new_exchange_re_arms_the_thunder() {
        // The conversation path (G8): SueD finishes, and the next key rotates
        // the reply aside and resets the engine. That is a SECOND `engine.reset`
        // call site, so a fix that only re-arms on F5 leaves every follow-up
        // question in the conversation unwarned.
        //
        // ⚠ THE PRECONDITION IS THE ENTIRE TEST, and the first version of this
        // test did not have it. It went straight to the reveal via
        // `ASK_AND_REVEAL`, which types only two hidden chars — so the line was
        // never crossed, `thunder_played` was still `false` at the reset, and the
        // assertion below passed whether or not the re-arm existed. It proved
        // nothing while looking like it proved the headline claim.
        //
        // Burn the FIRST decoy's thunder before asking, or this test is decor.
        let mut app = drive(&ASK_IN_HIDDEN);
        type_hidden_until_remaining(&mut app, THUNDER_AT_CHARS_REMAINING);
        assert!(
            app.take_pending_cue().is_some(),
            "precondition: the FIRST decoy must actually spend its thunder, \
             otherwise the re-arm below is never exercised"
        );

        app.handle_key(KeyPress::Enter); // ask it — SueD replies
        finish_the_reveal(&mut app);

        app.handle_key(KeyPress::Char('n')); // begins the next exchange → new decoy
        app.handle_key(KeyPress::Char(';')); // → Hidden
        type_hidden_until_remaining(&mut app, THUNDER_AT_CHARS_REMAINING);

        assert_eq!(
            app.take_pending_cue(),
            Some(AudioCue::Thunder),
            "every decoy gets its own warning, including the ones that arrive \
             mid-conversation rather than through F5"
        );
    }

    #[test]
    fn take_pending_cue_drains_so_the_thunder_plays_once() {
        // Mirrors `take_pending_save`. `main` drains this inside the keypress
        // block; if a second look still returned the cue, the tick loop would
        // replay the sting every frame until the next keystroke.
        let mut app = drive(&ASK_IN_HIDDEN);
        type_hidden_until_remaining(&mut app, THUNDER_AT_CHARS_REMAINING);

        assert!(app.take_pending_cue().is_some(), "first look gets the cue");
        assert_eq!(
            app.take_pending_cue(),
            None,
            "second look must be empty — the queue is drained, not merely read"
        );
    }

    // ── G11 · `Option<Reply>` — one Option over the correlated group ─────────
    //
    // ⚠ THIS IS A REFACTOR, SO READ THE RHYTHM DIFFERENTLY FROM G13/G14.
    // Behaviour is FROZEN. These tests pin the NEW SHAPE; the ~48 tests that go
    // through the public surface (`handle_key`, `is_pondering`, `visible_buffer`)
    // must stay BYTE-IDENTICAL through the change — they are the only thing
    // proving the refactor preserved behaviour. If a test has to be edited to
    // compile, its CLAIM must survive verbatim; only the reading mechanism moves.
    //
    // Target shape:
    //   struct Reply { words: String, asked_at: Instant, thinking_for: Duration }
    //   Screen::Asking { engine, reply: Option<Reply>, previous_reply, spell }
    //
    // `words` is a plain `String`, NOT an `Answer(String) | Denial(&'static str)`
    // enum — evidence, not taste: `ask.rs:141` renders a denial through the exact
    // same `typewriter_reveal` as an answer, with no styling difference, and
    // G12's `Message::Sued(String)` flattens the two anyway. Nothing downstream
    // can tell them apart, so the type should not pretend it matters.
    //
    // ⚠ NOTE WHAT IS DELIBERATELY *NOT* TESTED HERE. The old
    // `.expect("a reply clock with no reply words is a bug")` enforced "a reply
    // always has words" at RUNTIME. Once `Reply` owns `words`, that is a
    // COMPILE-TIME guarantee — a `Reply` without words cannot be constructed.
    // Writing a test for it would be writing a test that can never fail, which
    // is the vacuous-assertion trap this codebase has now been bitten by twice.
    // The rung moved from "loud" to "impossible"; impossible needs no test.

    /// A `Reply` whose clock was started `asked_secs` ago — same rewind trick as
    /// `finish_the_reveal`, so the ponder can be observed from either side.
    fn reply_asked_ago(asked_secs: u64, thinking_secs: u64) -> Reply {
        Reply {
            words: "42".to_string(),
            asked_at: Instant::now()
                .checked_sub(Duration::from_secs(asked_secs))
                .expect("the test clock must be able to rewind"),
            thinking_for: Duration::from_secs(thinking_secs),
        }
    }

    #[test]
    fn a_reply_ponders_while_it_is_still_inside_its_thinking_time() {
        let reply = reply_asked_ago(1, 5);

        assert!(
            reply.is_pondering(),
            "asked 1s ago with a 5s ponder — SueD is still weighing the mortal"
        );
    }

    #[test]
    fn a_reply_stops_pondering_once_the_thinking_time_is_spent() {
        let reply = reply_asked_ago(10, 3);

        assert!(
            !reply.is_pondering(),
            "the ponder must END — `main`'s falling-edge check is what fires the \
             reply sting, and an edge needs both sides"
        );
    }

    #[test]
    fn speaking_elapsed_stays_at_zero_for_the_whole_ponder() {
        let reply = reply_asked_ago(1, 5);

        assert_eq!(
            reply.speaking_elapsed(),
            Duration::ZERO,
            "SueD has not begun speaking, so the reveal clock has not started — \
             this is the `saturating_sub` in `reveal_elapsed`, and it must not \
             underflow into a colossal Duration"
        );
    }

    #[test]
    fn speaking_elapsed_counts_from_the_moment_the_ponder_ended() {
        // The whole point of the shifted clock (G13): the typewriter measures
        // time spent SPEAKING, not time since the question was asked.
        let reply = reply_asked_ago(10, 3);

        let speaking = reply.speaking_elapsed();
        assert!(
            speaking >= Duration::from_secs(6) && speaking < Duration::from_secs(8),
            "asked 10s ago minus a 3s ponder ≈ 7s of speaking, got {speaking:?}"
        );
    }

    #[test]
    fn a_fresh_ask_screen_has_no_reply() {
        let app = drive(&[KeyPress::Enter, KeyPress::Enter]);

        match &app.screen {
            Screen::Asking(AskingState { reply, .. }) => assert!(
                reply.is_none(),
                "nothing asked yet — `is_none()` now says what three separate \
                 cleared fields used to say between them"
            ),
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn revealing_an_answer_stamps_a_reply_carrying_that_answer() {
        let app = drive(&ASK_AND_REVEAL);

        match &app.screen {
            Screen::Asking(AskingState { reply, .. }) => {
                let reply = reply
                    .as_ref()
                    .expect("a revealed answer must stamp a reply");
                assert_eq!(
                    reply.words(),
                    "42",
                    "the staged answer becomes SueD's words"
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }

    #[test]
    fn a_denial_stamps_a_reply_too_so_one_field_carries_both_kinds() {
        // ⚠ THE HEADLINE TEST OF G11. Today the words live in two places and a
        // `match denied_message { Some(..) => .., None => engine.revealed()... }`
        // picks between them at every read site. One field carrying BOTH kinds is
        // what dissolves that match — and the `.expect` inside it.
        let app = drive(&ask_and_be_denied());
        let denials = app.config().language().translation().denials;

        match &app.screen {
            Screen::Asking(AskingState {
                reply: Some(reply), ..
            }) => {
                assert!(
                    denials.contains(&reply.words()),
                    "the taunt must come from the active language's denial pool, \
                     got {:?}",
                    reply.words()
                );
            }
            other => panic!("expected Asking, got {other:?}"),
        }
    }
}

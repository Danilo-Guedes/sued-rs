pub const APP_TITLE: &str = " ☠  SueD — O Oráculo  ☠ ";

/// The repository, read from the one place it is already declared.
///
/// ⚠ Not in the three translation tables, and not typed out again here: a URL
/// carries no language, and `Cargo.toml` already has to hold it for crates.io.
/// Three hand-copied strings would be three chances to drift the day the repo
/// moves — the same argument that put `RECOMMENDED_TERMINAL_SIZE` below.
pub const REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");

/// The operator's manual, advertised at the foot of the story popover.
///
/// ⏳ **This flag does not exist yet** (PLAN §G16 · Phase 6). The popover names
/// it because the design does, and the whole point of the story popover is to
/// give the one confused `cargo install` user somewhere to go. Wiring `clap`
/// is the outstanding half of G16 — until it lands, this line advertises a
/// command that errors.
pub const HOW_IT_WORKS_COMMAND: &str = "sued-rs --how-it-works";

/// The terminal size the info screen tells people to use.
///
/// It lives here rather than inside the three translations because it is a fact
/// about the app, not a piece of language — three copies of a number are three
/// chances to drift, and the string it replaced had drifted from reality
/// entirely (it recommended `80×24`, inherited from the VT100 default and never
/// measured; §J.7 found the app actually breaks below ~92×40).
///
/// This is the *comfortable* size — measured as the one where everything
/// renders correctly, and the size the mockups were drawn at. The hard floor is
/// lower (~92×40); G3 may yet compact the design or band the layout, and when it
/// does, this is the one line that changes.
pub const RECOMMENDED_TERMINAL_SIZE: &str = "132×41";

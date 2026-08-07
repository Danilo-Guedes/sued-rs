pub const APP_TITLE: &str = " ☠  SueD — O Oráculo  ☠ ";

/// The repository, read from the one place it is already declared.
///
/// ⚠ Not in the three translation tables, and not typed out again here: a URL
/// carries no language, and `Cargo.toml` already has to hold it for crates.io.
/// Three hand-copied strings would be three chances to drift the day the repo
/// moves — the same argument that put `RECOMMENDED_TERMINAL_SIZE` below.
///
/// 📌 This is the SOURCE, and it belongs with the operator's manual rather than
/// in the story popover: reading the repo is how you learn the trick, which is
/// precisely what `--how-it-works` is for and what the on-screen popover is
/// deliberately not.
pub const REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");

/// The author, as opposed to the project. These are what the story popover
/// shows — someone who read a personal memory wants the person, not the crate.
pub const AUTHOR_GITHUB: &str = "https://github.com/Danilo-Guedes";

pub const AUTHOR_LINKEDIN: &str = "https://linkedin.com/in/danilo-guedes-dev";

/// The operator's manual, advertised at the foot of the story popover.
///
/// ⚠ Keep this in step with `cli::Args::how_it_works`. It is a *string* naming a
/// flag, so nothing in the type system ties the two together — rename the flag
/// and the popover cheerfully goes on advertising the old one. The tests around
/// `cli::how_it_works_text` cover the manual's contents, not its name.
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

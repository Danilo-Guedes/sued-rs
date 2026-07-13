//! CLI args / flags via `clap` derive. Carries the M3 `--no-sound` switch for
//! now; grows in M5 (`--config`, theme/language select).

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "sued-rs",
    about = "SueD, o oráculo — a horror-themed prank oracle for your terminal."
)]
pub struct Args {
    /// Run with no audio at all (overrides the `audio` build feature).
    #[arg(long)]
    pub no_sound: bool,
}

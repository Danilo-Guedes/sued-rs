//! CLI args / flags via `clap` derive. Carries the M3 `--no-sound` switch and
//! the M5 `--config` override.

use std::path::PathBuf;

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

    /// Use a specific config file instead of the platform default
    /// (`~/.config/sued-rs/sued.config.json`).
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
}

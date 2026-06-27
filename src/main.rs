//! sued-rs — a horror-themed terminal recreation of the SueD prank oracle.
//!
//! This is the M0 scaffold. The app proper begins at **M1**: implement the pure
//! prank `Engine` in [`core::engine`]. See `../plan/PLAN.md` for the milestone plan
//! and the working agreement.

mod cli;
mod config;
mod core;
mod ui;

#[cfg(feature = "audio")]
mod audio;

use anyhow::Result;

fn main() -> Result<()> {
    println!(
        "sued-rs scaffold ready (M0). Next: implement the Engine in src/core/engine.rs — see plan/PLAN.md."
    );
    Ok(())
}

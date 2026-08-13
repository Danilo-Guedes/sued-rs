//! Core prank logic — pure, I/O-free, unit-testable.
//!
//! Nothing in here may touch the terminal, audio, or the filesystem. That is
//! what lets the entire illusion run under `cargo test` without opening a TTY.

pub mod engine;

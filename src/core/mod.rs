//! Core prank logic — pure, I/O-free, unit-testable.
//!
//! Nothing in here may touch the terminal, audio, or the filesystem. That keeps
//! the trick logic testable without a TTY (see the working agreement in CLAUDE.md).

pub mod engine;

//! ratatui draw loop, widget layout, and the frame/tick loop (M2).
//!
//! Reminder: this is a **tick loop** — `event::poll(timeout)` → redraw every tick →
//! `event::read()` only when there is input. Blocking on `read()` freezes animations.
//
// TODO(M2): framed oracle panel, input area, title, menu, marquee.

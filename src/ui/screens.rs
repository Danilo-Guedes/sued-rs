//! ratatui draw code — one submodule per screen; `render` dispatches on `App`.

mod about;
mod ask;
mod common;
mod config;
mod info;
mod intro;
mod menu;

use ratatui::Frame;

use crate::app::{App, Screen};

pub fn render(frame: &mut Frame, app: &App) {
    match app.screen() {
        Screen::Intro => intro::render(frame),
        Screen::Menu => menu::render(frame, app.menu()),
        Screen::Asking {
            engine,
            replied_at,
            denied_message,
        } => ask::render(
            frame,
            engine,
            *replied_at,
            *denied_message,
            app.started_at(),
        ),
        Screen::Info => info::render(frame),
        Screen::About => about::render(frame),
        Screen::Config => config::render(frame),
    }
}

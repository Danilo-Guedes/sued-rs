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
        Screen::Intro => intro::render(frame, app.config()),
        Screen::Menu => menu::render(frame, app.menu()),
        Screen::Asking {
            engine,
            replied_at,
            denied_message,
            previous_reply,
        } => ask::render(
            frame,
            engine,
            *replied_at,
            *denied_message,
            app.started_at(),
            app.config(),
            previous_reply,
        ),
        Screen::Info => info::render(frame),
        Screen::About => about::render(frame, app.config()),
        Screen::Config => config::render(frame, app),
    }
}

//! 05 · SOBRE O SUED. (placeholder — content still to be built)

use ratatui::Frame;
use ratatui::widgets::Block;

pub(super) fn render(frame: &mut Frame) {
    frame.render_widget(Block::bordered().title("ABOUT"), frame.area());
}

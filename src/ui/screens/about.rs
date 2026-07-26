//! 05 · SOBRE O SUED.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Borders, Padding, Paragraph, Wrap};

use crate::config::Configuration;
use crate::ui::effects::flicker_intensity;
use crate::ui::screens::common::{
    DEMON_ART, DEMON_ART_HEIGHT, DEMON_ART_WIDTH, NavTab, colorfull_bordered_block,
    create_centered_rect, create_screen_block, hint_line, render_nav_strip, table_row,
};
use crate::ui::template::styled_line;

pub(super) fn render(frame: &mut Frame, config: Configuration) {
    let palette = config.theme().palette();
    let layout = create_screen_block(frame, palette);
    let language = config.language();
    let translation = language.translation();

    let [
        nav_layout,
        _empty_space,
        center_layout,
        footer_layout,
        status_layout,
    ] = Layout::vertical([
        Constraint::Length(4), // nav strip
        Constraint::Fill(1),   //empty space
        Constraint::Fill(3),   // center: two panels
        Constraint::Fill(2),
        Constraint::Length(2), // status bar
    ])
    .areas(layout);

    render_nav_strip(
        frame,
        nav_layout,
        NavTab::About,
        palette,
        language,
        translation,
    );

    let [art_area, text_area, _empty] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
    ])
    .areas(center_layout);

    let art_rect = create_centered_rect(
        art_area,
        Constraint::Length(DEMON_ART_WIDTH),
        Constraint::Length(DEMON_ART_HEIGHT),
    );

    let random_flicker_value = flicker_intensity(rand::random(), config.animations());

    frame.render_widget(
        Paragraph::new(DEMON_ART).fg(palette.glow(random_flicker_value)),
        art_rect,
    );

    let mut lore_rows = vec![
        Line::from(translation.about.title.fg(palette.accent).bold()),
        Line::from(" "),
    ];

    lore_rows.extend(
        translation
            .about
            .lore
            .lines()
            .map(|row| styled_line(row, Style::default().white(), palette.accent)),
    );

    let lore = Paragraph::new(Text::from(lore_rows))
        .left_aligned()
        .wrap(Wrap { trim: false });

    const KEY_WIDTH: usize = 12;

    let spec_rows: Vec<_> = translation
        .about
        .table
        .iter()
        .map(|(label, value)| table_row(label, value, KEY_WIDTH, palette))
        .collect();

    let text_h = lore.line_count(text_area.width) as u16;
    let [_, text_block, _gap, table_block, _] = Layout::vertical([
        Constraint::Fill(1),                        // top spacer
        Constraint::Length(text_h),                 // lore, sized to its wrapped height
        Constraint::Length(2),                      // breathing space between text + table
        Constraint::Length(spec_rows.len() as u16), // the spec table (one row each)
        Constraint::Fill(1),                        // bottom spacer
    ])
    .areas(text_area);

    frame.render_widget(lore, text_block);
    frame.render_widget(Paragraph::new(spec_rows), table_block);

    let status_block =
        colorfull_bordered_block(Some(Borders::TOP), palette).padding(Padding::horizontal(2));
    let status_inner = status_block.inner(status_layout);
    frame.render_widget(status_block, status_layout);

    let [_, bottom_footer_layout] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).areas(footer_layout);

    let footer_text = Paragraph::new(translation.about.footer).dim().centered();

    frame.render_widget(footer_text, bottom_footer_layout);

    let [hints_area, page_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(14)]).areas(status_inner);

    let hints = hint_line(translation.about.hints, palette);
    frame.render_widget(Paragraph::new(hints), hints_area);
    frame.render_widget(
        Paragraph::new(NavTab::About.label(language).to_uppercase())
            .dim()
            .right_aligned(),
        page_area,
    );
}

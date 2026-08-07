//! 05 · SOBRE O SUED.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Borders, Padding, Paragraph, Wrap};

use crate::app::AboutState;
use crate::config::Configuration;
use crate::ui::effects::flicker_intensity;
use crate::ui::screens::common::{
    DEMON_ART, DEMON_ART_HEIGHT, DEMON_ART_WIDTH, NavTab, colorfull_bordered_block,
    create_centered_rect, create_screen_block, hint_line, render_nav_strip, table_row,
};
use crate::ui::screens::story;
use crate::ui::template::styled_line;

pub(super) fn render(frame: &mut Frame, config: Configuration, about_state: &AboutState) {
    let palette = config.theme().palette();
    let layout = create_screen_block(frame, palette);
    let language = config.language();
    let translation = language.translation();

    let [
        nav_layout,
        empty_space,
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

    // The strip describes the keys as they behave RIGHT NOW, so it swaps with
    // the overlay — same idiom as `ask.rs`, and here it is not merely tidy:
    // with the popover up `Esc` closes it rather than going back, so a strip
    // that kept saying "voltar ao menu" would be actively lying.
    //
    // 📌 Deliberately the screen's own strip and NOT a second one drawn inside
    // the popover, which is where the mockup put it. Two strips would be a
    // hand-maintained duplicate of each other — the exact complaint G20 exists
    // to fix on the Ritual screen — and the two rows it saves are worth having
    // at the 80×24 floor, where the prose viewport is single digits.
    let (current_hints, current_page) = match about_state.story() {
        Some(_) => (translation.about.story.hints, NavTab::Story),
        None => (translation.about.hints, NavTab::About),
    };

    let hints = hint_line(current_hints, palette);
    frame.render_widget(Paragraph::new(hints), hints_area);
    frame.render_widget(
        Paragraph::new(current_page.label(language).to_uppercase())
            .dim()
            .right_aligned(),
        page_area,
    );

    // ⚠ LAST, so it lands on top of everything above it — a `Clear` only clears
    // what has already been drawn.
    //
    // The band deliberately excludes the nav strip and the status bar: the strip
    // says which screen you are on and the bar now carries the popover's own
    // keys, so covering either would hide the only thing telling the reader how
    // to get out. ⚠ `union` is the bounding box of the two rects, so this spans
    // empty→center→footer only while those three stay ADJACENT in the vertical
    // layout above. Reorder that stack and the band silently changes meaning —
    // it still compiles, still draws, and covers the wrong thing.
    if let Some(story_view) = about_state.story() {
        story::render(
            frame,
            empty_space.union(footer_layout),
            story_view,
            palette,
            translation,
        );
    }
}

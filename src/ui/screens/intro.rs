//! 01 · INTRO / Invocação.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Borders, Padding, Paragraph, Wrap};

use crate::app::App;
use crate::ui::effects::flicker_intensity;
use crate::ui::screens::common::{
    NavTab, SUED_BANNER, SUED_BANNER_HEIGHT, SUED_BANNER_WIDTH, colorfull_bordered_block,
    create_centered_rect, create_screen_block, hint_line, render_nav_strip,
};
use crate::ui::screens::confirm;
use crate::ui::template::styled_line;

pub(super) fn render(frame: &mut Frame, app: &App) {
    let config = app.config();

    let palette = config.theme().palette();

    let language = config.language();

    let translation = language.translation();

    let layout = create_screen_block(frame, palette);

    let [
        nav_layout,
        _,
        page_title_and_sub_layout,
        intro_text_layout,
        _,
        status_layout,
    ] = Layout::vertical([
        Constraint::Length(4),  // nav strip
        Constraint::Fill(1),    // empty
        Constraint::Fill(3),    // page_title_and_sub
        Constraint::Length(18), // intro_text_layout
        Constraint::Fill(1),    // empty
        Constraint::Length(2),  // status bar
    ])
    .areas(layout);

    render_nav_strip(
        frame,
        nav_layout,
        NavTab::Intro,
        palette,
        language,
        translation,
    );

    let [banner_area, _gap, subtitle_area] = Layout::vertical([
        Constraint::Length(SUED_BANNER_HEIGHT),
        Constraint::Length(1), // breathing space
        Constraint::Length(1), // subtitle line
    ])
    .flex(Flex::Center)
    .areas(page_title_and_sub_layout);

    let banner_rect = create_centered_rect(
        banner_area,
        Constraint::Length(SUED_BANNER_WIDTH),
        Constraint::Length(SUED_BANNER_HEIGHT),
    );

    let random_flicker_value = flicker_intensity(rand::random(), config.animations());

    frame.render_widget(
        Paragraph::new(SUED_BANNER)
            .fg(palette.glow(random_flicker_value))
            .bold(),
        banner_rect,
    );

    frame.render_widget(
        Paragraph::new(translation.intro.subtitle.dim()).centered(),
        subtitle_area,
    );

    // Red rule + breathing space above the ATENÇÃO block (per the design). Split a
    // small strip off the top for the rule; the warning text fills the rest.
    let [divider_area, atencao_area] = Layout::vertical([
        Constraint::Length(3), // red rule (row 0) + a two-row gap below it
        Constraint::Fill(1),   // the warning text block
    ])
    .areas(intro_text_layout);

    // Match the rule to the same centred 50% band the warning text uses.
    let rule_band = divider_area.centered_horizontally(Constraint::Percentage(50));
    frame.render_widget(
        Paragraph::new("─".repeat(rule_band.width as usize)).fg(palette.accent),
        rule_band,
    );

    // A `Line` is a single row, so the multi-row blocks are split on `\n` here:
    // the translation owns where the sentences break, the render turns each one
    // into its own row.
    let mut warning_rows = vec![
        Line::from(translation.intro.attention.fg(palette.accent).bold()),
        Line::from(""), // blank row for breathing space
    ];
    warning_rows.extend(
        translation
            .intro
            .welcome
            .lines()
            .map(|row| styled_line(row, Style::default().white(), palette.accent)),
    );
    warning_rows.extend(translation.intro.disclaimer.lines().map(Line::from));
    warning_rows.extend([
        Line::from(""),
        Line::from(""),
        Line::from(
            format!("   {} ▸   ", translation.intro.continue_btn)
                .fg(palette.on_accent)
                .bg(palette.accent)
                .bold(),
        ),
    ]);

    let intro_texts = Text::from(warning_rows);

    frame.render_widget(
        Paragraph::new(intro_texts)
            .white()
            .centered()
            .wrap(Wrap { trim: false }),
        atencao_area.centered_horizontally(Constraint::Percentage(50)),
    );

    // The strip belongs to whatever owns the keys. While the quit-confirm is up
    // that is the dialog, so its hints replace the intro's — the same swap
    // `ask.rs` makes for the transcript and the séance confirm.
    let current_hint_slice = match app.confirm_quit() {
        Some(_) => translation.quit.hints,
        None => translation.intro.hints,
    };

    let status_texts = hint_line(current_hint_slice, palette);

    frame.render_widget(
        Paragraph::new(status_texts).block(
            colorfull_bordered_block(Some(Borders::TOP), palette).padding(Padding::new(2, 0, 0, 0)),
        ),
        status_layout,
    );

    // Drawn LAST so it lands on top of everything it covers.
    //
    // The band is everything BETWEEN the two strips — spelled out from their
    // edges rather than by unioning the middle areas, because at 80×24 the three
    // `Fill`s above collapse to zero height and a union would then depend on
    // where ratatui parks an empty rect. Reading `nav_layout.bottom()` and
    // `status_layout.y` cannot go wrong at any size, and it says what it means:
    // the dialog centres in the screen, exactly as it does on the menu.
    //
    // Both strips stay visible on purpose. The status one is now the dialog's own
    // hint line (it swapped just above), so covering it would hide the only thing
    // saying how to get out.
    //
    // ⚠⚠ `Cover::TheWholeRow` IS LOAD-BEARING HERE — it is what lets the box be
    // centred at all. This screen centres its rule and its warning text at
    // `Percentage(50)` of the terminal, so past ~124 columns both are WIDER than
    // the 62-column box and their ends surface either side of it: the rule as two
    // stray dashes welded to the border, the warning as orphaned words ("A",
    // "Para", "com"). Blanking the band's full width across the box's rows is the
    // only fix that holds at every size — a wider box does not, because the
    // content scales with the terminal.
    //
    // 📌 Found by dumping the buffer at 132×41, the recommended size.
    // At 92 and 80 the content is narrower than the box and nothing shows, so
    // this defect is invisible at two of the three sizes `SIZES` tests — which is
    // exactly why the dump happened and why the tests alone were not enough.
    //
    // 📌 The nav strip deliberately keeps highlighting `Invocação` rather than
    // swapping to `NavTab::Confirm`: `NavTab::ALL` is the five real tabs, so a
    // sixth would highlight none of them and read as a rendering fault. `ask.rs`
    // swaps its bottom-right *page label* instead, which is a slot the intro
    // does not have.
    if let Some(choice) = app.confirm_quit() {
        let band = Rect {
            x: layout.x,
            y: nav_layout.bottom(),
            width: layout.width,
            height: status_layout.y.saturating_sub(nav_layout.bottom()),
        };

        confirm::render(
            frame,
            band,
            choice,
            palette,
            translation.quit,
            confirm::Cover::TheWholeRow,
        );
    }
}

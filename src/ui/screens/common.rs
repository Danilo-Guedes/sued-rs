//! Small rendering helpers shared across screens.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::merge::MergeStrategy;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

use crate::constants::APP_TITLE;

/// The demon face — verbatim quadrant-block art (11 rows), shown on the
/// Pergunta + Sobre screens. Centre it by carving a Rect the *exact* size of the
/// art and rendering LEFT-aligned into it (see `create_centered_rect`) so the rows
/// stay locked together — never `.centered()`, which shears each row apart.
pub(super) const DEMON_ART: &str = r"  ▄▄▖                    ▗▄▄
  ▜██▄▄                ▄▄██▛
    ▀▜███▄▄▄▄▄▄▄▄▄▄███▜▀
        ▟█▀▘        ▝▀█▙
      ▄█▀   ▄▄    ▄▄   ▀█▄
     ██▌   ▐█▌    ▐█▌   ▐██
      ▀█▖   ▀▘    ▀▘   ▗█▀
        ▜█▄    ▄▄▄▄    ▄█▛
         ▝█▖ ▝▌▐▌▝▌ ▗█▘
          ▜█▄▖▙▟▟▙▗▄█▛
            ▀▀██████▀▀";

pub(super) const DEMON_ART_WIDTH: u16 = 28;
pub(super) const DEMON_ART_HEIGHT: u16 = 11;

/// The SUED word-mark — verbatim block-font banner, shown on the Intro screen.
/// Same rendering as `DEMON_ART`: centre a Rect to its exact size, then render
/// LEFT-aligned with the style on the `Paragraph` (never `.red()` on the `&str`,
/// which would collapse it into a single `Span`/line).
pub(super) const SUED_BANNER: &str = r"███████╗██╗   ██╗███████╗██████╗
██╔════╝██║   ██║██╔════╝██╔══██╗
███████╗██║   ██║█████╗  ██║  ██║
╚════██║██║   ██║██╔══╝  ██║  ██║
███████║╚██████╔╝███████╗██████╔╝
╚══════╝ ╚═════╝ ╚══════╝╚═════╝";

pub(super) const SUED_BANNER_WIDTH: u16 = 33;
pub(super) const SUED_BANNER_HEIGHT: u16 = 6;

/// Center `area` down to `width` × `height`, discarding the surrounding space.
pub(super) fn create_centered_rect(area: Rect, width: Constraint, height: Constraint) -> Rect {
    let [a] = Layout::horizontal([width]).flex(Flex::Center).areas(area);
    let [a] = Layout::vertical([height]).flex(Flex::Center).areas(a);
    a
}

/// The shared accent-red panel frame, so every screen's border colour lives in
/// one place (M5/M6 can swap the theme palette here).
pub(super) fn colorfull_bordered_block(border_type: Option<Borders>) -> Block<'static> {
    let final_border_style = match border_type {
        Some(border) => border,
        None => Borders::all(),
    };

    Block::new()
        .borders(final_border_style)
        .border_style(Style::default().fg(Color::Red))
        .merge_borders(MergeStrategy::Exact)
}

/// Accent "chip" for a step number: black glyphs on the accent colour.
pub(super) fn step_badge(n: u8) -> Span<'static> {
    Span::from(format!(" {n} ")).black().on_red().bold()
}

/// One aligned `key   description` row. `key_width` pads the key so the
/// descriptions line up into a column.
pub(super) fn table_row(key: &str, desc: &str, key_width: usize) -> Line<'static> {
    Line::from(vec![
        Span::from(format!("{:<width$}", key, width = key_width))
            .red()
            .bold(),
        Span::from(desc.to_string()).dim(),
    ])
}

/// A tab in the decorative top-nav strip. This is *orientation only* — it maps a
/// screen to its label; it does not decide which screen you're on. The set is the
/// four "destinations" (note: not the same as `MenuItem`, which also has `Exit`).
#[derive(Clone, Copy, PartialEq)]
pub(super) enum NavTab {
    Intro,
    Ask,
    Info,
    About,
    Config,
}

impl NavTab {
    const ALL: [NavTab; 5] = [
        NavTab::Intro,
        NavTab::Ask,
        NavTab::Info,
        NavTab::About,
        NavTab::Config,
    ];

    fn label(self) -> &'static str {
        match self {
            NavTab::Intro => "Invocação",
            NavTab::Ask => "Pergunta",
            NavTab::Info => "Informações",
            NavTab::About => "Sobre",
            NavTab::Config => "Config",
        }
    }
}

pub(super) fn render_nav_strip(frame: &mut Frame, area: Rect, active: NavTab) {
    // `area` must now be 2 rows tall: one row of tabs plus the red underline.
    let block = Block::new()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::Red))
        .padding(Padding::new(2, 2, 1, 0));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [tabs_area, session_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(24)]).areas(inner);

    // Build the tab row as one Line: label, four spaces, next label, ...
    let mut spans: Vec<Span<'static>> = Vec::new();
    for (i, tab) in NavTab::ALL.iter().enumerate() {
        if i > 0 {
            spans.push("    ".into()); // gap between tabs
        }

        if active == *tab {
            // Active: uppercased, black-on-red chip (leading/trailing space = padding).
            spans.push(
                Span::from(format!(" {} ", tab.label().to_uppercase()))
                    .black()
                    .on_red()
                    .bold(),
            );
        } else {
            spans.push(tab.label().dim());
        }
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), tabs_area);

    // Session badge — static placeholder for now; wire it to real state later.
    let session = Line::from(vec![
        "sessão #999 ".dim(),
        "· ".dim(),
        "●".red(),
        " online".dim(),
    ]);
    frame.render_widget(Paragraph::new(session).right_aligned(), session_area);
}

pub(super) fn create_screen_block(frame: &mut Frame) -> Rect {
    let outer_layout = colorfull_bordered_block(None).title(APP_TITLE);
    let inner_layout = outer_layout.inner(frame.area());

    frame.render_widget(outer_layout, frame.area());

    inner_layout
}

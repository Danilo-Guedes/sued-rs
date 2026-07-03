//! ratatui draw code — reads the `App` state each frame and renders it.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Padding, Paragraph, Wrap};

use crate::app::{App, Menu, Screen};
use crate::contants::APP_TITLE;
use crate::core::engine::Engine;

pub fn render(frame: &mut Frame, app: &App) {
    match app.screen() {
        Screen::Intro => {
            render_intro_screen(frame);
        }
        Screen::Menu => {
            render_menu_screen(frame, app.menu());
        }
        Screen::Asking(engine) => {
            render_ask_screen(frame, engine);
        }

        Screen::Info => {
            render_info_screen(frame);
        }
        Screen::About => {
            render_about_screen(frame);
        }
    }
}

fn render_intro_screen(frame: &mut Frame) {
    let [
        title_bar_layout,
        page_title_and_sub_layout,
        intro_text_layout,
        status_layout,
    ] = Layout::vertical([
        Constraint::Length(2), // title bar,
        Constraint::Fill(2),   // page_title_and_sub
        Constraint::Fill(3),   // intro_text_layout
        Constraint::Length(3), // status bar
    ])
    .areas(frame.area());

    frame.render_widget(
        Paragraph::new(APP_TITLE).red().bold().left_aligned(),
        title_bar_layout,
    );

    let page_title_and_sub_texts = Text::from(vec![
        Line::from("SUED".red().bold()),
        Line::from(""), // blank row for breathing space
        Line::from("SUA ÚLTIMA ESPERANÇA DIVINA".dim()),
    ]);

    frame.render_widget(
        Paragraph::new(page_title_and_sub_texts).centered(),
        page_title_and_sub_layout,
    );

    let intro_texts = Text::from(vec![
        Line::from("ATENÇÃO".bold()),
        Line::from(""), // blank row for breathing space
        Line::from("Você está prestes a abrir uma porta para o desconhecido."),
        Line::from(""),
        Line::from("Aconselho acender uma vela e apagar as luzes antes de executar o programa."),
        Line::from(""),
        Line::from(
            "Para que SUED responda, você deve elogiá-lo e em seguida perguntar de forma clara.",
        ),
        Line::from(""),
        Line::from("Pessoas fracas e sensíveis não devem utilizar o programa."),
        Line::from(""),
        Line::from("Tenha muito cuidado com o que você irá perguntar..."),
    ]);

    frame.render_widget(
        Paragraph::new(intro_texts)
            .red()
            .centered()
            .wrap(Wrap { trim: false }),
        intro_text_layout.centered_horizontally(Constraint::Percentage(50)),
    );

    let status_texts = Line::from(vec![
        "[Enter]".red().bold(),
        " ".into(),
        "continuar".dim(),
        " ".into(),
        "[Esc]".red().bold(),
        " ".into(),
        "sair".dim(),
    ]);

    frame.render_widget(
        Paragraph::new(status_texts).block(Block::bordered()),
        status_layout,
    );
}

fn render_menu_screen(frame: &mut Frame, menu: &Menu) {
    let [title_bar_layout, center_layout, status_layout] = Layout::vertical([
        Constraint::Length(2), // title title_bar_layout,
        Constraint::Fill(1),   // center_layout
        Constraint::Length(3), // status_layout
    ])
    .areas(frame.area());

    frame.render_widget(
        Paragraph::new(APP_TITLE).red().bold().left_aligned(),
        title_bar_layout,
    );

    let [menu_left_layout, disclaim_right_layout] =
        Layout::horizontal([Constraint::Fill(6), Constraint::Fill(4)]).areas(center_layout);

    let mut menu_items_lines: Vec<Line> = vec![];

    for (idx, menu_item) in Menu::ALL.iter().enumerate() {
        let label = if idx == menu.index() {
            menu_item.label().to_string().black().on_red()
        } else {
            menu_item.label().to_string().red()
        };

        menu_items_lines.push(Line::from(label));
    }

    frame.render_widget(Paragraph::new(menu_items_lines), menu_left_layout);

    let displaimer_texts = Text::from(vec![
        Line::from("ATENÇÃO").red(),
        Line::from(" "),
        Line::from("Pessoas fracas e sensíveis não devem utilizar o programa.").red(),
        Line::from(" "),
        Line::from("Acenda uma vela. Apague as luzes.").red(),
        Line::from(" "),
        Line::from("Tenha cuidado com o que irá perguntar...").red(),
    ]);

    frame.render_widget(Paragraph::new(displaimer_texts), disclaim_right_layout);

    let status_texts = Line::from(vec![
        "[↑↓]".red().bold(),
        " ".into(),
        "navegar".dim(),
        " ".into(),
        "[Enter]".red().bold(),
        " ".into(),
        "selecionar".dim(),
        "[Esc]".red().bold(),
        " ".into(),
        "sair".dim(),
    ]);

    frame.render_widget(
        Paragraph::new(status_texts).block(Block::bordered()),
        status_layout,
    );
}

fn render_ask_screen(frame: &mut Frame, engine: &Engine) {
    let [
        title_bar_layout,
        sued_art_layout,
        sued_says_layout,
        sued_logs_layout,
        input_layout,
        status_layout,
    ] = Layout::vertical([
        Constraint::Length(2), // title bar,
        Constraint::Fill(2),   // sued_art
        Constraint::Fill(2),   // sued_says
        Constraint::Fill(3),   // sued_logs
        Constraint::Length(4), // input box
        Constraint::Length(3), // status bar
    ])
    .areas(frame.area());

    let [title_bar_left, title_bar_right] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(22)]).areas(title_bar_layout);

    frame.render_widget(
        Paragraph::new(APP_TITLE).red().bold(),
        // .style(Style::new().red().rapid_blink()),
        title_bar_left,
    );

    let session = Line::from(vec![
        Span::raw("sessão #666  "),
        Span::raw("*").red(), // the "online" dot in its own color
        Span::raw(" online").red(),
    ]);

    frame.render_widget(Paragraph::new(session).right_aligned(), title_bar_right);
    frame.render_widget(Block::bordered().title("sued_art"), sued_art_layout);

    let speak_layout = create_centered_rect(
        sued_says_layout,
        Constraint::Length(60),
        Constraint::Length(8),
    );

    let default_sued_text = Text::from(vec![
        Line::from("Pergunte-me o que deseja saber, humano..."),
        Line::from(""), // blank row for breathing space
        Line::from(vec![
            Span::raw("— elogie-me antes da pergunta, e ").dim(),
            Span::raw("talvez").red(),
            Span::raw(" eu responda.").dim(),
        ]),
    ]);

    let final_sued_words = match engine.revealed() {
        Some(answer) => Text::from(answer),
        None => default_sued_text,
    };

    let speak_widget = Paragraph::new(final_sued_words)
        .block(
            Block::bordered()
                .title("SUED FALA")
                .padding(Padding::new(2, 2, 1, 1)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(speak_widget, speak_layout);

    let default_logs_text = Text::from(vec![
        Line::from(vec![
            Span::raw(">").red(),
            Span::raw(" "),
            Span::raw("conexão com o além estabelecida.").dim(),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(">").red(),
            Span::raw(" "),
            Span::raw("aguardando oferenda do mortal_").dim(),
        ]),
    ]);

    frame.render_widget(
        Paragraph::new(default_logs_text).block(
            Block::bordered()
                .title("sued_logs")
                .padding(Padding::new(2, 2, 1, 1)),
        ),
        sued_logs_layout,
    );

    let typed = Paragraph::new(engine.visible_buffer())
        .block(Block::bordered().title("input").on_light_red());

    frame.render_widget(typed, input_layout);

    frame.render_widget(
        Block::bordered().title("status_bar").on_red(),
        status_layout,
    );
}

fn render_info_screen(frame: &mut Frame) {
    let [title_bar_layout, center_layout, status_layout] = Layout::vertical([
        Constraint::Length(2), // title bar
        Constraint::Fill(1),   // center: two panels
        Constraint::Length(3), // status bar
    ])
    .areas(frame.area());

    frame.render_widget(
        Paragraph::new(APP_TITLE).red().bold().left_aligned(),
        title_bar_layout,
    );

    // The body is two side-by-side panels. Each panel is its own fn that takes
    // only its `Rect`, so it owns its internal layout — the screen fn just hands
    // out areas. That is the pattern to reuse on every complex screen.
    let [ritual_area, shortcuts_area] =
        Layout::horizontal([Constraint::Fill(6), Constraint::Fill(4)]).areas(center_layout);

    render_ritual_panel(frame, ritual_area);
    render_shortcuts_panel(frame, shortcuts_area);

    // Status bar: split the *inside* of one border into left hints + right page tag.
    let status_block = Block::bordered();
    let status_inner = status_block.inner(status_layout);
    frame.render_widget(status_block, status_layout);

    let [hints_area, page_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(14)]).areas(status_inner);

    let hints = Line::from(vec![
        "[Esc]".red().bold(),
        " ".into(),
        "voltar ao menu".dim(),
    ]);
    frame.render_widget(Paragraph::new(hints), hints_area);
    frame.render_widget(
        Paragraph::new("INFORMAÇÕES".dim()).right_aligned(),
        page_area,
    );
}

/// Left panel — the 4-step ritual.
fn render_ritual_panel(frame: &mut Frame, area: Rect) {
    // The reusable move for a bordered panel with content:
    //   1. build the frame `Block`,
    //   2. ask it for the `inner` content rect (border + padding removed),
    //   3. draw the frame,
    //   4. lay the content out inside `inner`.
    let block = Block::bordered()
        .title(Line::from("▚ O RITUAL ▞").red().bold())
        .padding(Padding::new(2, 2, 1, 1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [steps_area, divider_area, example_area] = Layout::vertical([
        Constraint::Fill(1),   // numbered steps
        Constraint::Length(1), // divider
        Constraint::Length(2), // example
    ])
    .areas(inner);

    let steps = vec![
        Line::from(vec![
            step_badge(1),
            " ".into(),
            "Acenda uma vela e apague as luzes do recinto.".into(),
        ]),
        Line::from(""),
        Line::from(vec![
            step_badge(2),
            " ".into(),
            "Elogie".red().bold(),
            " o Sued antes de qualquer coisa — ele é vaidoso.".into(),
        ]),
        Line::from(""),
        Line::from(vec![
            step_badge(3),
            " ".into(),
            "Faça ".into(),
            "uma".red().bold(),
            " pergunta por vez, de forma clara e objetiva.".into(),
        ]),
        Line::from(""),
        Line::from(vec![
            step_badge(4),
            " ".into(),
            "Aguarde em silêncio. A resposta virá do além.".into(),
        ]),
    ];
    frame.render_widget(Paragraph::new(steps), steps_area);

    // Divider stretches to fill the content width — sized from the rect, not hard-coded.
    let divider = "─".repeat(inner.width as usize);
    frame.render_widget(Paragraph::new(divider).dim(), divider_area);

    let example = Line::from("» Ex.: \"Sued, o mais sábio de todos, o que me aguarda amanhã?\"")
        .dim()
        .italic();
    frame.render_widget(
        Paragraph::new(example).wrap(Wrap { trim: false }),
        example_area,
    );
}

/// Right panel — the keyboard shortcuts table.
fn render_shortcuts_panel(frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .title(Line::from("⌨ ATALHOS").red().bold())
        .padding(Padding::new(2, 2, 1, 1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [rows_area, footer_area] = Layout::vertical([
        Constraint::Fill(1),   // key/desc rows
        Constraint::Length(1), // bottom-pinned footer
    ])
    .areas(inner);

    // A "table" here is just aligned lines: pad the key column to a fixed width
    // so every description starts at the same column. No table widget needed.
    const KEY_WIDTH: usize = 9;
    let rows = vec![
        table_row("Enter", "perguntar / confirmar", KEY_WIDTH),
        table_row("↑ ↓", "navegar o menu", KEY_WIDTH),
        table_row("Tab", "alternar menu", KEY_WIDTH),
        table_row("Esc", "voltar", KEY_WIDTH),
        table_row("Ctrl-C", "encerrar sessão", KEY_WIDTH),
    ];
    frame.render_widget(Paragraph::new(rows), rows_area);

    frame.render_widget(
        Paragraph::new(Line::from("⌁ terminal 80×24 recomendado").dim()),
        footer_area,
    );
}

/// Accent "chip" for a step number: black glyphs on the accent colour.
fn step_badge(n: u8) -> Span<'static> {
    Span::from(format!(" {n} ")).black().on_red().bold()
}

/// One aligned `key   description` row. `key_width` pads the key so the
/// descriptions line up into a column.
fn table_row(key: &str, desc: &str, key_width: usize) -> Line<'static> {
    Line::from(vec![
        Span::from(format!("{:<width$}", key, width = key_width))
            .red()
            .bold(),
        Span::from(desc.to_string()).dim(),
    ])
}

fn render_about_screen(frame: &mut Frame) {
    frame.render_widget(Block::bordered().title("ABOUT"), frame.area());
}

fn create_centered_rect(area: Rect, width: Constraint, height: Constraint) -> Rect {
    let [a] = Layout::horizontal([width]).flex(Flex::Center).areas(area);
    let [a] = Layout::vertical([height]).flex(Flex::Center).areas(a);
    a
}

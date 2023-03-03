use crate::app::App;
use crate::styles;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs, Wrap},
    Frame,
};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Length(3), // Tab bar
                Constraint::Min(0),    // Tab content
                Constraint::Length(2), // Status bar
            ]
            .as_ref(),
        )
        .split(f.size());
    draw_tab_bar(f, app, chunks[0]);
    match app.tab_index {
        0 => draw_console_tab(f, app, chunks[1]),
        1 => draw_debug_tab(f, app, chunks[1]),
        _ => {}
    };
    draw_feedback_text(f, app, chunks[2]);
}

fn draw_tab_bar<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let titles = vec!["Console", "Debug"]
        .iter()
        .map(|t| Spans::from(Span::styled(*t, styles::tab())))
        .collect();
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .title(Span::styled(app.title, styles::title()))
                .borders(Borders::ALL)
                .border_style(styles::border()),
        )
        .highlight_style(styles::active_tab())
        .select(app.tab_index);
    f.render_widget(tabs, chunk);
}

fn draw_console_tab<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)])
        .split(chunk);
    let inner_chunks = Layout::default()
        .constraints([Constraint::Min(5), Constraint::Min(2)])
        .split(chunks[0]);
    draw_tasks(f, app, chunks[1]);
    draw_help(f, app, inner_chunks[0]);
    draw_prompt(f, app, inner_chunks[1]);
}

fn draw_tasks<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let spans: Vec<Spans> = app
        .task_list
        .iter()
        .map(|t| Spans::from(format!("- {}", t.desc)))
        .collect();
    let block = Block::default()
        .title(Span::styled("Tasks", styles::title()))
        .borders(Borders::ALL)
        .border_style(styles::border());
    let paragraph = Paragraph::new(spans).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunk);
}

fn draw_help<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let spans = multiline_to_spans(&app.help_text);
    let block = Block::default()
        .title(Span::styled("Help", styles::title()))
        .borders(Borders::ALL)
        .border_style(styles::border());
    let paragraph = Paragraph::new(spans).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunk);
}

fn draw_prompt<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let chunks = Layout::default()
        .constraints([Constraint::Max(1), Constraint::Max(1)])
        .split(chunk);
    let style = match app.prompt_handler {
        Some(_) => styles::prompt(),
        None => styles::dim(),
    };
    app.textarea.set_style(style);
    let cursor_style = match app.prompt_handler {
        Some(_) => Style::default().bg(Color::Magenta),
        None => Style::default().bg(Color::Black),
    };
    let prompt_text: String;
    match &app.prompt_handler {
        Some(handler) => prompt_text = format!("{}:", handler.to_string()),
        None => prompt_text = "".to_string(),
    }
    app.textarea.set_cursor_style(cursor_style);
    f.render_widget(
        Paragraph::new(Spans::from(Span::styled(prompt_text, styles::highlight()))),
        chunks[0],
    );
    f.render_widget(app.textarea.widget(), chunks[1])
}

fn draw_feedback_text<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let text = Span::styled(format!(">> {}", app.user_feedback_text), styles::dim());
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(styles::border());
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunk);
}

fn draw_debug_tab<B>(f: &mut Frame<B>, _app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0)])
        .split(area);
    let colors = [
        // Color::Reset,
        // Color::Black,
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::Gray,
        Color::DarkGray,
        Color::LightRed,
        Color::LightGreen,
        Color::LightYellow,
        Color::LightBlue,
        Color::LightMagenta,
        Color::LightCyan,
        Color::White,
    ];
    let items: Vec<Row> = colors
        .iter()
        .map(|c| {
            let cells = vec![
                Cell::from(Span::styled(format!("{:?}: ", c), styles::text())),
                Cell::from(Span::styled(
                    "Foreground",
                    Style::default().bg(Color::Black).fg(*c),
                )),
                Cell::from(Span::styled(
                    "Background",
                    Style::default().bg(*c).fg(Color::Black),
                )),
            ];
            Row::new(cells)
        })
        .collect();
    let table = Table::new(items)
        .block(
            Block::default()
                .title(Span::styled("Colors", styles::title()))
                .borders(Borders::ALL)
                .border_style(styles::border()),
        )
        .widths(&[
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ]);
    f.render_widget(table, chunks[0]);
}

fn multiline_to_spans(text: &str) -> Vec<Spans> {
    text.split("\n").map(|l| Spans::from(l.trim())).collect()
}

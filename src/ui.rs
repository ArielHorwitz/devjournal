use crate::app::{App, GitOutput};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
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
                Constraint::Length(1), // Console
                Constraint::Length(2), // Status bar
            ]
            .as_ref(),
        )
        .split(f.size());
    draw_tab_bar(f, app, chunks[0]);
    match app.tab_index {
        0 => draw_first_tab(f, app, chunks[1]),
        1 => draw_second_tab(f, app, chunks[1]),
        2 => draw_third_tab(f, app, chunks[1]),
        _ => {}
    };
    draw_text_area(f, app, chunks[2]);
    draw_feedback_text(f, app, chunks[3]);
}

fn draw_tab_bar<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let titles = vec!["Console", "Overview", "Debug"]
        .iter()
        .map(|t| Spans::from(Span::styled(*t, Style::default().fg(Color::Green))))
        .collect();
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .title(app.title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .highlight_style(Style::default().fg(Color::LightMagenta))
        .select(app.tab_index);
    f.render_widget(tabs, chunk);
}

fn draw_first_tab<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let spans = multiline_to_spans(&app.console_text);
    let block = Block::default().borders(Borders::ALL).title(Span::styled(
        "Console",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ));
    let paragraph = Paragraph::new(spans).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunk);
}

fn draw_second_tab<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let mut spans = multiline_to_spans(app.git_text.get(&GitOutput::Files).unwrap());
    spans.append(&mut multiline_to_spans(
        app.git_text.get(&GitOutput::Status).unwrap(),
    ));
    let block = Block::default().borders(Borders::ALL).title(Span::styled(
        "Git files and status",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ));
    let paragraph = Paragraph::new(spans).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunk);
}

fn draw_third_tab<B>(f: &mut Frame<B>, _app: &mut App, area: Rect)
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
                Cell::from(Span::raw(format!("{:?}: ", c))),
                Cell::from(Span::styled("Foreground", Style::default().fg(*c))),
                Cell::from(Span::styled(
                    "Background",
                    Style::default().bg(*c).fg(Color::Black),
                )),
            ];
            Row::new(cells)
        })
        .collect();
    let table = Table::new(items)
        .block(Block::default().title("Colors").borders(Borders::ALL))
        .widths(&[
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ]);
    f.render_widget(table, chunks[0]);
}

fn draw_text_area<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let style = match app.focus_text {
        true => Style::default().bg(Color::Black).fg(Color::Green),
        false => Style::default().bg(Color::Black).fg(Color::DarkGray),
    };
    app.textarea.set_style(style);
    let cursor_style = match app.focus_text {
        true => Style::default().bg(Color::LightMagenta).fg(Color::Black),
        false => Style::default().bg(Color::Black).fg(Color::DarkGray),
    };
    app.textarea.set_cursor_style(cursor_style);
    f.render_widget(app.textarea.widget(), chunk)
}

fn draw_feedback_text<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let text = Span::styled(
        app.feedback_text.clone(),
        Style::default().fg(Color::DarkGray),
    );
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::Magenta));
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunk);
}

fn multiline_to_spans(text: &str) -> Vec<Spans> {
    text.split("\n").map(|l| Spans::from(l.trim())).collect()
}

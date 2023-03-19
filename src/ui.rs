use crate::app::App;
mod styles;
pub mod widgets;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Row, Table, Tabs},
    Frame,
};

fn center_rect(width: u16, height: u16, chunk: Rect) -> Rect {
    Rect::new(
        chunk
            .x
            .saturating_add(1)
            .max((chunk.width.saturating_sub(width)) / 2),
        chunk
            .y
            .saturating_add(1)
            .max((chunk.height.saturating_sub(height)) / 2),
        width.min(chunk.width.saturating_sub(2)),
        height.min(chunk.height.saturating_sub(2)),
    )
}

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Length(3),                                 // Tab bar
                Constraint::Length(f.size().height.saturating_sub(5)), // Tab content
                Constraint::Length(2),                                 // Status bar
            ]
            .as_ref(),
        )
        .split(f.size());
    draw_tab_bar(f, app, chunks[0]);
    match app.tab_index {
        0 => draw_main_content(f, app, chunks[1]),
        1 => draw_debug_tab(f, app, chunks[1]),
        _ => {}
    };
    widgets::draw_feedback_text(f, app, chunks[2]);
    if let crate::app::AppFocus::Prompt(handler) = app.focus {
        widgets::draw_prompt(f, app, center_rect(80, 3, chunks[1]), &handler.to_string());
    };
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

fn draw_main_content<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)])
        .split(chunk);
    widgets::draw_sidebar(f, app, chunks[0]);
    widgets::draw_tasks(f, app, chunks[1]);
}

pub fn draw_debug_tab<B>(f: &mut Frame<B>, _app: &mut App, area: Rect)
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

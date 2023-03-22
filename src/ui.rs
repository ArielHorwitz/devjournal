use crate::app::App;
mod styles;
pub mod widgets;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs},
    Frame,
};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Length(2),                                 // Tab bar
                Constraint::Length(f.size().height.saturating_sub(4)), // Tab content
                Constraint::Length(2),                                 // Status bar
            ]
            .as_ref(),
        )
        .split(f.size());
    draw_tab_bar(f, app, chunks[0]);
    match app.tab_index {
        0 => app.project_widget.draw(f, chunks[1]),
        1 => draw_debug_tab(f, app, chunks[1]),
        _ => {}
    };
    draw_feedback_text(f, app, chunks[2]);
}

fn draw_tab_bar<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(styles::border());
    let inner = block.inner(chunk);
    f.render_widget(block, chunk);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Length(1),
            Constraint::Length(29),
            Constraint::Length(chunk.width.saturating_sub(30)),
        ])
        .split(inner);
    let project_name = Paragraph::new(Span::styled(
        format!("Project: {}", app.project_widget.project_name()),
        styles::title(),
    ));
    f.render_widget(project_name, chunks[1]);
    let titles = vec!["Project", "Debug"]
        .iter()
        .map(|t| Spans::from(Span::styled(*t, styles::tab_dim())))
        .collect();
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::LEFT))
        .highlight_style(styles::tab())
        .select(app.tab_index);
    f.render_widget(tabs, chunks[2]);
}

fn draw_feedback_text<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(styles::border());
    let inner = block.inner(chunk);
    f.render_widget(block, chunk);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(3, 4), Constraint::Ratio(1, 4)])
        .split(inner);
    f.render_widget(Paragraph::new(app.user_feedback_text.clone()), chunks[0]);
    let text = Span::styled(
        format!("( terminal: {}×{} )", f.size().width, f.size().height),
        styles::text_dim(),
    );
    let paragraph = Paragraph::new(text).alignment(tui::layout::Alignment::Right);
    f.render_widget(paragraph, chunks[1]);
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
                .title(Span::styled("Colors", styles::title_dim()))
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

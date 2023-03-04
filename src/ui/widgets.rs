use super::styles;
use crate::app::App;
use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn draw_tasks<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let enumerated_tasks = (0..app.task_list.len()).zip(app.task_list.iter());
    let spans: Vec<Spans> = enumerated_tasks
        .map(|(i, t)| Spans::from(format!("{i}. {}", t.desc)))
        .collect();
    let block = Block::default()
        .title(Span::styled("Tasks", styles::title()))
        .borders(Borders::ALL)
        .border_style(styles::border());
    let paragraph = Paragraph::new(spans).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunk);
}

pub fn draw_help<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let spans = multiline_to_spans(&app.help_text);
    let block = Block::default()
        .title(Span::styled("Help", styles::title()))
        .borders(Borders::ALL)
        .border_style(styles::border());
    let paragraph = Paragraph::new(spans).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunk);
}

pub fn draw_prompt<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect) {
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

pub fn draw_feedback_text<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let text = Span::styled(format!(">> {}", app.user_feedback_text), styles::dim());
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(styles::border());
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunk);
}

fn multiline_to_spans(text: &str) -> Vec<Spans> {
    text.split("\n").map(|l| Spans::from(l.trim())).collect()
}
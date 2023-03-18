use super::styles;
use crate::app::{App, AppFocus};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
pub mod list;

pub fn draw_tasks<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let title = format!("Tasks - {}", app.get_active_filename());
    let border_style: Style;
    let title_style: Style;
    let highlight_style: Style;
    match app.focus {
        AppFocus::TaskList => {
            border_style = styles::border_highlighted();
            title_style = styles::title_highlighted();
            highlight_style = styles::list_highlight();
        }
        _ => {
            border_style = styles::border();
            title_style = styles::title();
            highlight_style = styles::list_normal();
        }
    };
    let test = app
        .task_list
        .widget()
        .block(
            Block::default()
                .title(Span::styled(title, title_style))
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .style(styles::list_normal())
        .highlight_style(highlight_style);
    f.render_widget(test, chunk);
}

pub fn draw_sidebar<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Percentage(100)])
        .split(chunk);
    // File list
    let border_style: Style;
    let title_style: Style;
    let highlight_style: Style;
    match app.focus {
        AppFocus::FileList => {
            border_style = styles::border_highlighted();
            title_style = styles::title_highlighted();
            highlight_style = styles::list_highlight();
        }
        _ => {
            border_style = styles::border();
            title_style = styles::title();
            highlight_style = styles::list_normal();
        }
    };
    let file_list = app
        .file_list
        .widget()
        .block(
            Block::default()
                .title(Span::styled("Files", title_style))
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .style(styles::list_normal())
        .highlight_style(highlight_style);
    f.render_widget(file_list, chunks[0]);
    // Help text
    let spans = multiline_to_spans(&app.help_text);
    let block = Block::default()
        .title(Span::styled("Help", styles::title()))
        .borders(Borders::ALL)
        .border_style(styles::border());
    let paragraph = Paragraph::new(spans).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunks[1]);
}

pub fn draw_prompt<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect, prompt_text: &str) {
    f.render_widget(Clear, chunk);
    app.textarea.set_cursor_line_style(styles::prompt());
    app.textarea
        .set_cursor_style(Style::default().bg(Color::Magenta));
    let block = Block::default()
        .title(Span::styled(prompt_text, styles::highlight()))
        .borders(Borders::ALL)
        .border_style(styles::border_highlighted());
    let inner = block.inner(chunk);
    f.render_widget(block, chunk);
    f.render_widget(app.textarea.widget(), inner)
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

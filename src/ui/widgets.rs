use self::list::ListWidget;
use super::styles;
use crate::app::App;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
pub mod list;
pub mod project;
pub mod prompt;

pub fn draw_sidebar<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Percentage(100)])
        .split(chunk);
    // File list
    let border_style: Style;
    let title_style: Style;
    let highlight_style: Style;
    // match app.focus {
    //     AppFocus::FileList => {
    //         border_style = styles::border_highlighted();
    //         title_style = styles::title_highlighted();
    //         highlight_style = styles::list_highlight();
    //     }
    //     _ => {
    //         border_style = styles::border();
    //         title_style = styles::title();
    //         highlight_style = styles::list_normal();
    //     }
    // };
    border_style = styles::border();
    title_style = styles::title();
    highlight_style = styles::list_normal();
    let file_list = ListWidget::new(app.file_list.as_strings(), app.file_list.selected())
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

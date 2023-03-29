use crate::app::appstate::AppState;
pub mod events;
mod styles;
pub mod widgets;
use self::widgets::{center_rect, list::ListWidget};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs},
    Frame,
};

pub fn draw<B: Backend>(frame: &mut Frame<B>, state: &AppState, debug: bool) {
    let chunks = Layout::default()
        .constraints(vec![
            Constraint::Length(2),
            Constraint::Length(frame.size().height.saturating_sub(4)),
            Constraint::Length(2),
        ])
        .split(frame.size());
    draw_tab_bar(frame, state, chunks[0]);
    match debug {
        false => draw_project(frame, state, chunks[1]),
        true => draw_debug_tab(frame, state, chunks[1]),
    };
    draw_feedback_text(frame, state, chunks[2]);
}

fn draw_tab_bar<B: Backend>(frame: &mut Frame<B>, state: &AppState, chunk: Rect) {
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(styles::border());
    let inner = block.inner(chunk);
    frame.render_widget(block, chunk);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Length(1),
            Constraint::Length(29),
            Constraint::Length(chunk.width.saturating_sub(30)),
        ])
        .split(inner);
    let project_name = Paragraph::new(Span::styled(
        format!("Project: {}", state.project.name),
        styles::title(),
    ));
    frame.render_widget(project_name, chunks[1]);
    let titles = vec!["Project", "Debug"]
        .iter()
        .map(|t| Spans::from(Span::styled(*t, styles::tab_dim())))
        .collect();
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::LEFT))
        .highlight_style(styles::tab())
        .select(state.tab_index);
    frame.render_widget(tabs, chunks[2]);
}

fn draw_feedback_text<B: Backend>(frame: &mut Frame<B>, state: &AppState, chunk: Rect) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(styles::border());
    let inner = block.inner(chunk);
    frame.render_widget(block, chunk);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(3, 4), Constraint::Ratio(1, 4)])
        .split(inner);
    frame.render_widget(Paragraph::new(state.user_feedback_text.clone()), chunks[0]);
    let text = Span::styled(
        format!(
            "( terminal: {}×{} )",
            frame.size().width,
            frame.size().height
        ),
        styles::text_dim(),
    );
    let paragraph = Paragraph::new(text).alignment(tui::layout::Alignment::Right);
    frame.render_widget(paragraph, chunks[1]);
}

pub fn draw_debug_tab<B>(frame: &mut Frame<B>, _state: &AppState, area: Rect)
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
    frame.render_widget(table, chunks[0]);
}

fn draw_project<B: Backend>(frame: &mut Frame<B>, state: &AppState, rect: Rect) {
    draw_subprojects(frame, state, rect);
    if state.file_request.is_some() {
        state.filelist.draw(frame, center_rect(35, 20, rect, 1));
    } else if state.project_state.prompt_request.is_some() {
        state.project_state.prompt.draw(frame, rect);
    };
}

fn draw_subprojects<B: Backend>(frame: &mut Frame<B>, state: &AppState, rect: Rect) {
    let subproject_count = state.project.len() as u16;
    let percent_unfocus = ((100. - state.project_state.focused_width_percent as f32)
        / (subproject_count as f32 - 1.).floor()) as u16;
    let constraints: Vec<Constraint> = (0..subproject_count)
        .map(|i| {
            if i == state.project.subprojects.selected().unwrap() as u16 {
                Constraint::Percentage(state.project_state.focused_width_percent)
            } else {
                Constraint::Percentage(percent_unfocus)
            }
        })
        .collect();
    let chunks = Layout::default()
        .direction(state.project_state.split_orientation.clone())
        .constraints(constraints)
        .split(rect);
    for (index, subproject) in state.project.subprojects.iter().enumerate() {
        let mut border_style = styles::border();
        let mut title_style = styles::title_dim();
        let mut focus = false;
        if Some(index) == state.project.subprojects.selected() {
            border_style = styles::border_highlighted();
            title_style = styles::title();
            focus = true;
        }
        let widget = ListWidget::new(subproject.tasks.as_strings(), subproject.tasks.selected())
            .block(
                Block::default()
                    .title(Span::styled(&subproject.name, title_style))
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .focus(focus);
        frame.render_widget(widget, chunks[index]);
    }
}

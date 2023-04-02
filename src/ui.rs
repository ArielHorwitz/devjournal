use crate::app::data::{App, Project};
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

pub fn draw<B: Backend>(frame: &mut Frame<B>, state: &App, debug: bool) {
    let chunks = Layout::default()
        .constraints(vec![
            Constraint::Length(2),
            Constraint::Length(frame.size().height.saturating_sub(4)),
            Constraint::Length(2),
        ])
        .split(frame.size());
    draw_tab_bar(frame, state, chunks[0]);
    if debug {
        draw_debug_tab(frame, state, chunks[1]);
    } else {
        if let Some(project) = state.journal.projects.get_item(None) {
            draw_project(frame, project, chunks[1]);
        }
        if state.file_request.is_some() {
            state
                .filelist
                .draw(frame, center_rect(40, 20, chunks[1], 1));
        }
    };
    if state.prompt_request.is_some() {
        state.prompt.draw(frame, chunks[1]);
    }
    draw_feedback_text(frame, state, chunks[2]);
}

fn draw_tab_bar<B: Backend>(frame: &mut Frame<B>, state: &App, chunk: Rect) {
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
    frame.render_widget(
        Paragraph::new(Span::styled(&state.journal.name, styles::title())),
        chunks[1],
    );
    let titles = state
        .journal
        .projects
        .iter()
        .map(|t| Spans::from(Span::styled(&t.name, styles::tab_dim())))
        .collect();
    let mut tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::LEFT))
        .highlight_style(styles::tab_dim());
    if let Some(selected) = state.journal.projects.selected() {
        tabs = tabs.select(selected).highlight_style(styles::tab());
    }
    frame.render_widget(tabs, chunks[2]);
}

fn draw_feedback_text<B: Backend>(frame: &mut Frame<B>, state: &App, chunk: Rect) {
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

pub fn draw_debug_tab<B>(frame: &mut Frame<B>, _state: &App, area: Rect)
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
                Cell::from(Span::styled(format!("{c:?}: "), styles::text())),
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

fn draw_project<B: Backend>(frame: &mut Frame<B>, project: &Project, rect: Rect) {
    draw_subprojects(frame, project, rect);
    if project.prompt_request.is_some() {
        project.prompt.draw(frame, rect);
    };
}

fn draw_subprojects<B: Backend>(frame: &mut Frame<B>, project: &Project, rect: Rect) {
    let subproject_count = project.subprojects.items().len() as u16;
    let percent_unfocus = if subproject_count > 1 {
        let remainder = 100. - project.focused_width_percent as f32;
        (remainder / (subproject_count as f32 - 1.).floor()) as u16
    } else {
        100
    };
    let subproject_index = project.subprojects.selected();
    let constraints: Vec<Constraint> = (0..subproject_count)
        .map(|i| {
            if subproject_index == Some(i as usize) {
                Constraint::Percentage(project.focused_width_percent)
            } else {
                Constraint::Percentage(percent_unfocus)
            }
        })
        .collect();
    let direction = match project.split_vertical {
        true => Direction::Vertical,
        false => Direction::Horizontal,
    };
    let chunks = Layout::default()
        .direction(direction)
        .constraints(constraints)
        .split(rect);
    for (index, subproject) in project.subprojects.iter().enumerate() {
        let mut border_style = styles::border();
        let mut title_style = styles::title_dim();
        let mut focus = false;
        if Some(index) == project.subprojects.selected() {
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

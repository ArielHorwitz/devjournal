use super::{
    list::ListWidget,
    prompt::{PromptEvent, PromptWidget},
};
use crate::{
    app::project::{Project, SubProject, Task},
    ui::styles,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    text::Span,
    widgets::{Block, Borders},
    Frame,
};

pub struct ProjectWidget<'a> {
    prompt: PromptWidget<'a>,
    subproject_focus: usize,
    prompt_request: Option<PromptRequest>,
}

impl<'a> ProjectWidget<'a> {
    pub fn default() -> ProjectWidget<'a> {
        ProjectWidget {
            prompt: PromptWidget::default(),
            subproject_focus: 0,
            prompt_request: None,
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, chunk: Rect, project: &Project) {
        self.draw_tasks(f, chunk, project);
        if self.prompt_request.is_some() {
            self.prompt.draw(f, chunk);
        };
    }

    pub fn draw_tasks<B: Backend>(&mut self, f: &mut Frame<B>, chunk: Rect, project: &Project) {
        let subproject_count = project.subprojects.len();
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Ratio(1, subproject_count as u32);
                subproject_count
            ])
            .split(chunk);
        for (index, subproject) in project.subprojects.iter().enumerate() {
            let mut border_style = styles::border();
            let mut title_style = styles::title();
            let mut highlight_style = styles::list_normal();
            if index == self.subproject_focus {
                border_style = styles::border_highlighted();
                title_style = styles::title_highlighted();
                highlight_style = styles::list_highlight();
            }
            let widget =
                ListWidget::new(subproject.tasks.as_strings(), subproject.tasks.selected())
                    .block(
                        Block::default()
                            .title(Span::styled(&subproject.name, title_style))
                            .borders(Borders::ALL)
                            .border_style(border_style),
                    )
                    .style(styles::list_normal())
                    .highlight_style(highlight_style);
            f.render_widget(widget, chunks[index]);
        }
    }

    pub fn handle_event(&mut self, key: KeyEvent, project: &mut Project) {
        if self.prompt_request.is_some() {
            self.handle_prompt_event(key, project);
        } else {
            self.handle_subproject_event(key, project);
        }
    }

    fn handle_prompt_event(&mut self, key: KeyEvent, project: &mut Project) {
        if let Some(pr) = &self.prompt_request {
            match self.prompt.handle_event(key) {
                PromptEvent::Cancelled => self.prompt_request = None,
                PromptEvent::AwaitingResult(_) => (),
                PromptEvent::Result(result_text) => {
                    self.prompt.set_text("");
                    match pr {
                        PromptRequest::AddSubProject => {
                            project.subprojects.push(SubProject::new(&result_text));
                        }
                        PromptRequest::AddTask(index) => {
                            project.subprojects[index.clone()]
                                .tasks
                                .add_item(Task::new(&result_text));
                        }
                    };
                    self.prompt_request = None;
                }
            };
        }
    }

    fn handle_subproject_event(&mut self, key: KeyEvent, project: &mut Project) {
        match (key.code, key.modifiers) {
            (KeyCode::Char('n'), KeyModifiers::NONE) => {
                self.prompt_request = Some(PromptRequest::AddTask(self.subproject_focus));
                self.prompt.set_prompt_text("New task description: ");
            }
            (KeyCode::Char('='), KeyModifiers::ALT) => {
                self.prompt_request = Some(PromptRequest::AddSubProject);
                self.prompt.set_prompt_text("New subproject name: ");
            }
            (KeyCode::Char('h'), KeyModifiers::NONE) => {
                if self.subproject_focus == 0 {
                    self.subproject_focus = project.subprojects.len() - 1;
                } else {
                    self.subproject_focus -= 1;
                }
            }
            (KeyCode::Char('l'), KeyModifiers::NONE) => {
                self.subproject_focus = (self.subproject_focus + 1) % project.subprojects.len();
            }
            (KeyCode::Char('k'), KeyModifiers::NONE) => {
                project.subprojects[self.subproject_focus]
                    .tasks
                    .select_prev();
            }
            (KeyCode::Char('j'), KeyModifiers::NONE) => {
                project.subprojects[self.subproject_focus]
                    .tasks
                    .select_next();
            }
            (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                project.subprojects[self.subproject_focus]
                    .tasks
                    .move_up()
                    .ok();
            }
            (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
                project.subprojects[self.subproject_focus]
                    .tasks
                    .move_down()
                    .ok();
            }
            _ => (),
        };
    }
}

enum PromptRequest {
    AddSubProject,
    AddTask(usize),
}

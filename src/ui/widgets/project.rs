use super::{
    center_rect,
    files::{FileListResult, FileListWidget},
    list::ListWidget,
    prompt::{PromptEvent, PromptWidget},
};
use crate::{
    app::project::{Project, SubProject, Task},
    ui::styles,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use pathdiff::diff_paths;
use std::{
    fs::{remove_file, File},
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

const CREATE_CHAR: char = '⁕';
const LOAD_CHAR: char = '★';
const SAVE_CHAR: char = '☑';
const DELETE_CHAR: char = '☒';

enum PromptRequest {
    AddSubProject,
    AddTask,
    RenameTask,
}

enum FileRequest {
    Save,
    Load,
}

pub struct ProjectWidget<'a> {
    datadir: PathBuf,
    project: Project,
    prompt: PromptWidget<'a>,
    subproject_focus: usize,
    prompt_request: Option<PromptRequest>,
    filelist: FileListWidget<'a>,
    file_request: Option<FileRequest>,
}

impl<'a> ProjectWidget<'a> {
    pub fn new(datadir: &str) -> ProjectWidget<'a> {
        ProjectWidget {
            datadir: PathBuf::from(datadir.clone()),
            project: Project::new("New Project", "Tasks"),
            prompt: PromptWidget::default(),
            subproject_focus: 0,
            prompt_request: None,
            filelist: FileListWidget::new(datadir),
            file_request: None,
        }
    }

    pub fn project_name(&self) -> &str {
        &self.project.name
    }

    pub fn draw<B: Backend>(&self, f: &mut Frame<B>, chunk: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)])
            .split(chunk);
        self.draw_sidebar(f, chunks[0]);
        self.draw_subprojects(f, chunks[1]);
        if self.file_request.is_some() {
            self.filelist.draw(f, center_rect(25, 20, chunk));
        } else if self.prompt_request.is_some() {
            self.prompt.draw(f, chunk);
        };
    }

    fn draw_sidebar<B: Backend>(&self, f: &mut Frame<B>, chunk: Rect) {
        let block = Block::default()
            .title(Span::styled(&self.project.name, styles::title()))
            .borders(Borders::ALL)
            .border_style(styles::border());
        let paragraph = Paragraph::new(Spans::from(format!("Project: {}", &self.project.name)))
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, chunk);
    }

    fn draw_subprojects<B: Backend>(&self, f: &mut Frame<B>, chunk: Rect) {
        let subproject_count = self.project.subprojects.len();
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Ratio(1, subproject_count as u32);
                subproject_count
            ])
            .split(chunk);
        for (index, subproject) in self.project.subprojects.iter().enumerate() {
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

    pub fn handle_event(&mut self, key: KeyEvent) {
        if self.file_request.is_some() {
            self.handle_filelist_event(key);
        } else if self.prompt_request.is_some() {
            self.handle_prompt_event(key);
        } else {
            self.handle_subproject_event(key);
        }
    }

    fn handle_subproject_event(&mut self, key: KeyEvent) {
        match (key.code, key.modifiers) {
            // Project operations
            (KeyCode::Char('='), KeyModifiers::ALT) => {
                self.prompt_request = Some(PromptRequest::AddSubProject);
                self.prompt.set_prompt_text("New Subproject Name: ");
            }
            (KeyCode::Char('-'), KeyModifiers::ALT) => {
                if self.project.subprojects.len() > 1 {
                    self.project.subprojects.remove(self.subproject_focus);
                    self.subproject_focus = self.subproject_focus.saturating_sub(1);
                }
            }
            (KeyCode::Char('l'), KeyModifiers::NONE) => {
                self.subproject_focus =
                    (self.subproject_focus + 1) % self.project.subprojects.len();
            }
            (KeyCode::Char('h'), KeyModifiers::NONE) => {
                if self.subproject_focus == 0 {
                    self.subproject_focus = self.project.subprojects.len() - 1;
                } else {
                    self.subproject_focus -= 1;
                }
            }
            // Subproject operations
            (KeyCode::Char('n'), KeyModifiers::NONE) => {
                self.prompt_request = Some(PromptRequest::AddTask);
                self.prompt.set_prompt_text(&format!(
                    "New Task For {}: ",
                    self.project.subprojects[self.subproject_focus].name
                ));
            }
            (KeyCode::Char('d'), KeyModifiers::NONE) => {
                self.project.subprojects[self.subproject_focus]
                    .tasks
                    .pop_selected();
            }
            (KeyCode::Char('r'), KeyModifiers::NONE) => {
                self.prompt_request = Some(PromptRequest::RenameTask);
                self.prompt.set_prompt_text(&format!(
                    "Rename Task From: `{}`: ",
                    self.project.subprojects[self.subproject_focus].name
                ));
            }
            // Subproject navigation
            (KeyCode::Char('j'), KeyModifiers::NONE) => {
                self.project.subprojects[self.subproject_focus]
                    .tasks
                    .select_next();
            }
            (KeyCode::Char('k'), KeyModifiers::NONE) => {
                self.project.subprojects[self.subproject_focus]
                    .tasks
                    .select_prev();
            }
            (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
                self.project.subprojects[self.subproject_focus]
                    .tasks
                    .move_down()
                    .ok();
            }
            (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                self.project.subprojects[self.subproject_focus]
                    .tasks
                    .move_up()
                    .ok();
            }
            // File operations
            (KeyCode::Char('o'), KeyModifiers::CONTROL) => {
                self.file_request = Some(FileRequest::Load);
                self.filelist.refresh_filelist();
            }
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                self.project.save_file(&self.datadir);
                self.filelist.refresh_filelist();
            }
            (KeyCode::Char('s'), KeyModifiers::ALT) => {
                self.file_request = Some(FileRequest::Save);
                self.filelist.refresh_filelist();
            }
            _ => (),
        };
    }

    fn handle_prompt_event(&mut self, key: KeyEvent) {
        if let Some(pr) = &self.prompt_request {
            match self.prompt.handle_event(key) {
                PromptEvent::Cancelled => self.prompt_request = None,
                PromptEvent::AwaitingResult(_) => (),
                PromptEvent::Result(result_text) => {
                    self.prompt.set_text("");
                    match pr {
                        PromptRequest::AddSubProject => {
                            self.project
                                .subprojects
                                .insert(self.subproject_focus + 1, SubProject::new(&result_text));
                            self.subproject_focus += 1;
                        }
                        PromptRequest::AddTask => {
                            self.project.subprojects[self.subproject_focus]
                                .tasks
                                .add_item(Task::new(&result_text));
                        }
                        PromptRequest::RenameTask => {
                            let subproject = &mut self.project.subprojects[self.subproject_focus];
                            if let Some(task) = subproject.tasks.selected_value() {
                                let new_task = Task {
                                    desc: result_text,
                                    ..task.clone()
                                };
                                subproject.tasks.replace_selected(new_task);
                            }
                        }
                    };
                    self.prompt_request = None;
                }
            };
        }
    }

    fn handle_filelist_event(&mut self, key: KeyEvent) {
        match self.filelist.handle_event(key) {
            FileListResult::AwaitingResult => (),
            FileListResult::Cancelled => self.file_request = None,
            FileListResult::Feedback(_message) => (),
            FileListResult::Result(name) => {
                if let Some(fr) = &self.file_request {
                    match fr {
                        FileRequest::Load => {
                            self.project = Project::from_file(&self.datadir.join(&name));
                        }
                        FileRequest::Save => {
                            self.project.save_file(&self.datadir);
                        }
                    }
                    self.file_request = None;
                }
            }
        }
    }
}

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
use std::path::PathBuf;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    text::Span,
    widgets::{Block, Borders},
    Frame,
};

#[derive(Clone)]
enum PromptRequest {
    SetProjectPassword,
    GetLoadPassword(String),
    RenameProject,
    RenameSubProject,
    AddSubProject,
    AddTask,
    RenameTask,
}

#[derive(Clone, Copy)]
enum FileRequest {
    Save,
    Load,
}

pub struct ProjectWidget<'a> {
    datadir: PathBuf,
    project: Project,
    prompt: PromptWidget<'a>,
    prompt_request: Option<PromptRequest>,
    filelist: FileListWidget<'a>,
    file_request: Option<FileRequest>,
    project_password: String,
    project_filepath: PathBuf,
}

impl<'a> ProjectWidget<'a> {
    pub fn new(datadir: &str) -> ProjectWidget<'a> {
        let datadir_path = PathBuf::from(datadir);
        let project = Project::default();
        ProjectWidget {
            datadir: datadir_path.clone(),
            project,
            prompt: PromptWidget::default().width_hint(0.7),
            prompt_request: None,
            filelist: FileListWidget::new(datadir),
            file_request: None,
            project_password: "".to_string(),
            project_filepath: datadir_path.join("new_project"),
        }
    }

    pub fn project_name(&self) -> &str {
        &self.project.name
    }

    fn save_project(&mut self, filepath: Option<&PathBuf>) {
        let filepath = filepath.unwrap_or(&self.project_filepath);
        self.project
            .save_file_encrypted(filepath, &self.project_password)
            .expect("failed to save");
        self.filelist.refresh_filelist();
    }

    fn load_project(&mut self, name: &str, key: &str) {
        let filepath = self.datadir.join(name);
        self.project = Project::from_file_encrypted(&filepath, key).expect("failed to load");
        self.project_password = key.to_string();
        self.project_filepath = filepath;
        self.filelist.refresh_filelist();
    }

    pub fn draw<B: Backend>(&self, f: &mut Frame<B>, chunk: Rect) {
        self.draw_subprojects(f, chunk);
        if self.file_request.is_some() {
            self.filelist.draw(f, center_rect(35, 20, chunk, 1));
        } else if self.prompt_request.is_some() {
            self.prompt.draw(f, chunk);
        };
    }

    fn draw_subprojects<B: Backend>(&self, f: &mut Frame<B>, chunk: Rect) {
        let subproject_count = self.project.len() as u32;
        let small: u32 = 1;
        let large: u32 = 2;
        let total: u32 = small * (subproject_count - 1) + large;
        let constraints: Vec<Constraint> = (0..subproject_count)
            .map(|i| {
                if i == self.project.subprojects.selected().unwrap() as u32 {
                    Constraint::Ratio(large, total)
                } else {
                    Constraint::Ratio(small, total)
                }
            })
            .collect();
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(chunk);
        for (index, subproject) in self.project.subprojects.iter().enumerate() {
            let mut border_style = styles::border();
            let mut title_style = styles::title_dim();
            let mut focus = false;
            if Some(index) == self.project.subprojects.selected() {
                border_style = styles::border_highlighted();
                title_style = styles::title();
                focus = true;
            }
            let widget =
                ListWidget::new(subproject.tasks.as_strings(), subproject.tasks.selected())
                    .block(
                        Block::default()
                            .title(Span::styled(&subproject.name, title_style))
                            .borders(Borders::ALL)
                            .border_style(border_style),
                    )
                    .focus(focus);
            f.render_widget(widget, chunks[index]);
        }
    }

    pub fn handle_event(&mut self, key: KeyEvent) -> Option<String> {
        if self.file_request.is_some() {
            self.handle_filelist_event(key)
        } else if self.prompt_request.is_some() {
            self.handle_prompt_event(key)
        } else {
            self.handle_subproject_event(key)
        }
    }

    fn handle_subproject_event(&mut self, key: KeyEvent) -> Option<String> {
        let selected_subproject = self.project.subprojects.selected_value();
        match (key.code, key.modifiers) {
            // Project operations
            (KeyCode::Char('p'), KeyModifiers::ALT) => {
                self.set_prompt_extra(
                    PromptRequest::SetProjectPassword,
                    &format!("Set new password for `{}`:", self.project.name),
                    "",
                    true,
                );
            }
            (KeyCode::Char('r'), KeyModifiers::ALT) => {
                self.set_prompt(PromptRequest::RenameProject, "New Project Name:");
            }
            (KeyCode::Char('R'), KeyModifiers::SHIFT) => {
                if let Some(subproject) = selected_subproject {
                    let name = subproject.name.clone();
                    self.set_prompt_extra(
                        PromptRequest::RenameSubProject,
                        "New Subproject Name:",
                        &name,
                        false,
                    );
                }
            }
            (KeyCode::Char('='), KeyModifiers::ALT) => {
                self.set_prompt(PromptRequest::AddSubProject, "New Subproject Name:");
            }
            (KeyCode::Char('-'), KeyModifiers::ALT) => {
                self.project.subprojects.pop_selected();
            }
            (KeyCode::Char('l'), KeyModifiers::NONE) => self.project.subprojects.select_next(),
            (KeyCode::Char('h'), KeyModifiers::NONE) => self.project.subprojects.select_prev(),
            (KeyCode::Char('l'), KeyModifiers::ALT) => {
                self.project.subprojects.move_down().ok();
            }
            (KeyCode::Char('h'), KeyModifiers::ALT) => {
                self.project.subprojects.move_up().ok();
            }
            (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
                if let Some(subproject) = selected_subproject {
                    if let Some(task) = subproject.tasks.pop_selected() {
                        let target_subproject = self.project.subprojects.next_value().unwrap();
                        target_subproject.tasks.insert_item(
                            target_subproject.tasks.selected(),
                            task,
                            true,
                        );
                        self.project.subprojects.select_next();
                    }
                }
            }
            (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                if let Some(subproject) = selected_subproject {
                    if let Some(task) = subproject.tasks.pop_selected() {
                        let target_subproject = self.project.subprojects.prev_value().unwrap();
                        target_subproject.tasks.insert_item(
                            target_subproject.tasks.selected(),
                            task,
                            true,
                        );
                        self.project.subprojects.select_prev()
                    }
                }
            }
            // Subproject operations
            (KeyCode::Char('n'), KeyModifiers::NONE) => {
                self.set_prompt(PromptRequest::AddTask, "New Task:");
            }
            (KeyCode::Char('d'), KeyModifiers::NONE) => {
                if let Some(subproject) = selected_subproject {
                    subproject.tasks.pop_selected();
                }
            }
            (KeyCode::Char('r'), KeyModifiers::NONE) => {
                if let Some(subproject) = selected_subproject {
                    if let Some(task) = subproject.tasks.selected_value() {
                        let desc = task.desc.clone();
                        self.set_prompt_extra(
                            PromptRequest::RenameTask,
                            "Rename Task:",
                            &desc,
                            false,
                        );
                    }
                }
            }
            // Subproject navigation
            (KeyCode::Esc, KeyModifiers::NONE) => {
                if let Some(subproject) = selected_subproject {
                    subproject.tasks.deselect();
                }
            }
            (KeyCode::Char('j'), KeyModifiers::NONE) => {
                if let Some(subproject) = selected_subproject {
                    subproject.tasks.select_next();
                }
            }
            (KeyCode::Char('k'), KeyModifiers::NONE) => {
                if let Some(subproject) = selected_subproject {
                    subproject.tasks.select_prev();
                }
            }
            (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
                if let Some(subproject) = selected_subproject {
                    subproject.tasks.move_down().ok();
                }
            }
            (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                if let Some(subproject) = selected_subproject {
                    subproject.tasks.move_up().ok();
                }
            }
            // File operations
            (KeyCode::Char('o'), KeyModifiers::CONTROL) => {
                self.file_request = Some(FileRequest::Load);
                self.filelist.refresh_filelist();
                self.filelist.set_title_text("Open Project:");
                self.filelist.set_prompt_text("Create New File:");
            }
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                self.save_project(None);
                return Some(format!("Saved project: {:?}", self.project_filepath));
            }
            (KeyCode::Char('s'), KeyModifiers::ALT) => {
                self.file_request = Some(FileRequest::Save);
                self.filelist.refresh_filelist();
                self.filelist.set_title_text("Save Project:");
                self.filelist.set_prompt_text("Save File As:");
            }
            _ => (),
        };
        None
    }

    fn handle_prompt_event(&mut self, key: KeyEvent) -> Option<String> {
        if let Some(pr) = self.prompt_request.clone() {
            match self.prompt.handle_event(key) {
                PromptEvent::Cancelled => self.prompt_request = None,
                PromptEvent::AwaitingResult(_) => (),
                PromptEvent::Result(result_text) => {
                    self.clear_prompt();
                    let subproject = self.project.subprojects.selected_value();
                    match pr {
                        PromptRequest::SetProjectPassword => {
                            self.project_password = result_text;
                            return Some("Reset project password".to_string());
                        }
                        PromptRequest::GetLoadPassword(name) => {
                            self.load_project(&name, &result_text);
                            return Some(format!("Loaded project: {:?}", self.project_filepath));
                        }
                        PromptRequest::RenameProject => {
                            self.project.name = result_text;
                            return Some(format!("Renamed project: {}", self.project.name));
                        }
                        PromptRequest::RenameSubProject => {
                            if let Some(subproject) = subproject {
                                subproject.name = result_text;
                            }
                        }
                        PromptRequest::AddSubProject => {
                            self.project.subprojects.insert_item(
                                self.project.subprojects.next_index(),
                                SubProject::new(&result_text),
                                true,
                            );
                        }
                        PromptRequest::AddTask => {
                            if let Some(subproject) = subproject {
                                subproject.tasks.add_item(Task::new(&result_text));
                            };
                        }
                        PromptRequest::RenameTask => {
                            if let Some(subproject) = subproject {
                                if let Some(task) = subproject.tasks.selected_value() {
                                    let new_task = Task {
                                        desc: result_text.clone(),
                                        ..task.clone()
                                    };
                                    subproject.tasks.replace_selected(new_task);
                                }
                            }
                        }
                    };
                    self.prompt_request = None;
                }
            };
        }
        None
    }

    fn handle_filelist_event(&mut self, key: KeyEvent) -> Option<String> {
        match self.filelist.handle_event(key) {
            FileListResult::AwaitingResult => (),
            FileListResult::Cancelled => self.file_request = None,
            FileListResult::Feedback(_message) => (),
            FileListResult::Result(name) => {
                if let Some(fr) = &self.file_request {
                    match fr {
                        FileRequest::Load => self.set_prompt_extra(
                            PromptRequest::GetLoadPassword(name.clone()),
                            &format!("Password for `{}`:", name),
                            "",
                            true,
                        ),
                        FileRequest::Save => {
                            self.save_project(Some(&self.datadir.join(name)));
                            return Some(format!("Saved project {:?}", self.project_filepath));
                        }
                    }
                    self.file_request = None;
                }
            }
        }
        None
    }

    fn clear_prompt(&mut self) {
        self.prompt.set_prompt_text("Input:");
        self.prompt.set_text("");
        self.prompt_request = None;
        self.prompt.set_password(false);
    }

    fn set_prompt(&mut self, request: PromptRequest, prompt_text: &str) {
        self.set_prompt_extra(request, prompt_text, "", false)
    }

    fn set_prompt_extra(
        &mut self,
        request: PromptRequest,
        prompt_text: &str,
        prefill_text: &str,
        password: bool,
    ) {
        self.prompt.set_prompt_text(prompt_text);
        self.prompt.set_text(prefill_text);
        self.prompt_request = Some(request);
        self.prompt.set_password(password);
    }
}

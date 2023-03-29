use crate::ui::widgets::{files::FileListWidget, prompt::PromptWidget};

use super::project::Project;
use std::path::PathBuf;
use tui::layout::Direction;

pub const DEFAULT_WIDTH_PERCENT: u16 = 40;
pub const DEFAULT_PROJECT_FILENAME: &'static str = "new_project";

#[derive(Clone)]
pub enum PromptRequest {
    SetProjectPassword,
    GetLoadPassword(String),
    RenameProject,
    RenameSubProject,
    AddSubProject,
    AddTask,
    RenameTask,
}

#[derive(Clone, Copy)]
pub enum FileRequest {
    Save,
    Load,
}

pub struct AppState<'a> {
    pub title: &'a str,
    pub datadir: PathBuf,
    pub tab_index: usize,
    pub user_feedback_text: String,
    pub filelist: FileListWidget<'a>,
    pub file_request: Option<FileRequest>,
    pub project_filepath: PathBuf,
    pub project: Project,
    pub project_state: ProjectState<'a>,
}

impl<'a> AppState<'a> {
    pub fn new(title: &'a str, datadir: PathBuf) -> AppState<'a> {
        AppState {
            title,
            datadir: datadir.clone(),
            tab_index: 0,
            user_feedback_text: format!("Welcome to {title}."),
            filelist: FileListWidget::new(datadir.to_string_lossy().to_string().as_str()),
            file_request: None,
            project_filepath: datadir.join(DEFAULT_PROJECT_FILENAME),
            project: Project::default(),
            project_state: ProjectState::default(),
        }
    }
}

pub struct ProjectState<'a> {
    pub prompt: PromptWidget<'a>,
    pub prompt_request: Option<PromptRequest>,
    pub project_password: String,
    pub focused_width_percent: u16,
    pub split_orientation: Direction,
}

impl<'a> ProjectState<'a> {
    pub fn default() -> ProjectState<'a> {
        ProjectState {
            prompt: PromptWidget::default().width_hint(0.7),
            prompt_request: None,
            project_password: "".to_string(),
            focused_width_percent: DEFAULT_WIDTH_PERCENT,
            split_orientation: Direction::Horizontal,
        }
    }
}

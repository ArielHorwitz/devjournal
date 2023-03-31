use super::list::SelectionList;
use crate::crypto::{decrypt, encrypt};
use crate::ui::widgets::{files::FileListWidget, prompt::PromptWidget};
use serde::{Deserialize, Serialize};
use std::{fmt, fs, path::PathBuf};

pub const DEFAULT_WIDTH_PERCENT: u16 = 40;
pub const DEFAULT_PROJECT_FILENAME: &str = "new_project";

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

pub struct App<'a> {
    pub title: &'a str,
    pub datadir: PathBuf,
    pub tab_index: usize,
    pub user_feedback_text: String,
    pub filelist: FileListWidget<'a>,
    pub file_request: Option<FileRequest>,
    pub filepath: PathBuf,
    pub project: Project<'a>,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, datadir: PathBuf) -> App<'a> {
        App {
            title,
            datadir: datadir.clone(),
            tab_index: 0,
            user_feedback_text: format!("Welcome to {title}."),
            filelist: FileListWidget::new(datadir.to_string_lossy().to_string().as_str()),
            file_request: None,
            filepath: datadir.join(DEFAULT_PROJECT_FILENAME),
            project: Project::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Project<'a> {
    pub name: String,
    pub subprojects: SelectionList<SubProject>,
    #[serde(skip)]
    pub prompt: PromptWidget<'a>,
    #[serde(skip)]
    pub prompt_request: Option<PromptRequest>,
    pub project_password: String,
    pub focused_width_percent: u16,
    pub split_vertical: bool,
}

impl<'a> Default for Project<'a> {
    fn default() -> Project<'a> {
        Project {
            name: "New Project".to_owned(),
            subprojects: SelectionList::from_vec(vec![SubProject::default()]),
            prompt: PromptWidget::default().width_hint(0.7),
            prompt_request: None,
            project_password: "".to_owned(),
            focused_width_percent: DEFAULT_WIDTH_PERCENT,
            split_vertical: false,
        }
    }
}

impl<'a> Project<'a> {
    pub fn from_file(filepath: &PathBuf, key: &str) -> Result<Project<'a>, String> {
        if !filepath.exists() {
            Project::default().save_file(filepath, key)?;
        }
        let encrypted = fs::read(filepath).map_err(|e| format!("failed to read file [{e}]"))?;
        let encoded = decrypt(&encrypted, key)?;
        bincode::deserialize::<Project>(encoded.as_slice())
            .map_err(|e| format!("failed to deserialize [{e}]"))
    }

    pub fn save_file(&self, filepath: &PathBuf, key: &str) -> Result<(), String> {
        let encoded =
            bincode::serialize(&self).map_err(|e| format!("failed to serialize [{e}]"))?;
        let encrypted = encrypt(&encoded, key)?;
        fs::write(filepath, encrypted).map_err(|e| format!("failed to write file [{e}]"))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubProject {
    pub name: String,
    pub tasks: SelectionList<Task>,
}

impl Default for SubProject {
    fn default() -> SubProject {
        SubProject {
            name: "Tasks".to_owned(),
            tasks: SelectionList::default(),
        }
    }
}

impl SubProject {
    pub fn new(name: &str) -> SubProject {
        SubProject {
            name: name.to_owned(),
            tasks: SelectionList::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task {
    pub desc: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

impl Task {
    pub fn new(desc: &str) -> Task {
        Task {
            desc: desc.to_owned(),
            created_at: "2020-02-02 12:00:00".to_owned(),
            completed_at: None,
        }
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.desc.to_owned())
    }
}

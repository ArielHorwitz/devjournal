use super::list::SelectionList;
use crate::crypto::{decrypt, encrypt};
use crate::ui::widgets::{files::FileListWidget, prompt::PromptWidget};
use serde::{self, Deserialize, Serialize};
use std::ops::Add;
use std::{fmt, fs, path::PathBuf};

pub const DEFAULT_WIDTH_PERCENT: u16 = 40;
pub const DEFAULT_PROJECT_FILENAME: &str = "new_project";

pub fn load_decrypt<T>(filepath: &PathBuf, key: &str) -> Result<T, String>
where
    T: DataDeserialize<T>,
{
    let encrypted = fs::read(filepath).map_err(|e| format!("failed to read file [{e}]"))?;
    let decrypted = decrypt(&encrypted, key)?;
    T::deserialize(decrypted)
}

pub fn save_encrypt<T>(object: &T, filepath: &PathBuf, key: &str) -> Result<(), String>
where
    T: DataSerialize<T>,
{
    let encoded = T::serialize(object)?;
    let encrypted = encrypt(&encoded, key)?;
    fs::write(filepath, encrypted).map_err(|e| format!("failed to write file [{e}]"))
}

pub trait DataSerialize<T> {
    fn serialize(decoded: &T) -> Result<Vec<u8>, String>;
}

pub trait DataDeserialize<T> {
    fn deserialize(encoded: Vec<u8>) -> Result<T, String>;
}

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

    pub fn project(&mut self) -> Option<&mut Project<'a>> {
        Some(&mut self.project)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Project<'a> {
    pub name: String,
    pub password: String,
    pub subprojects: SelectionList<SubProject>,
    #[serde(skip)]
    pub prompt: PromptWidget<'a>,
    #[serde(skip)]
    pub prompt_request: Option<PromptRequest>,
    pub focused_width_percent: u16,
    pub split_vertical: bool,
}

impl<'a> Project<'a> {
    pub fn subproject(&mut self) -> Option<&mut SubProject> {
        self.subprojects.get_item_mut(None)
    }
}

impl<'a> Clone for Project<'a> {
    fn clone(&self) -> Self {
        Project {
            name: self.name.clone(),
            password: self.password.clone(),
            subprojects: self.subprojects.clone(),
            split_vertical: self.split_vertical,
            focused_width_percent: self.focused_width_percent,
            ..Project::default()
        }
    }
}

impl<'a> Default for Project<'a> {
    fn default() -> Project<'a> {
        Project {
            name: "New Project".to_owned(),
            password: "".to_owned(),
            subprojects: SelectionList::from_vec(vec![SubProject::default()]),
            prompt: PromptWidget::default().width_hint(0.7),
            prompt_request: None,
            focused_width_percent: DEFAULT_WIDTH_PERCENT,
            split_vertical: false,
        }
    }
}

impl<'a> Add<Project<'a>> for Project<'a> {
    type Output = Project<'a>;

    fn add(self, rhs: Project<'a>) -> Self::Output {
        Project {
            name: self.name.clone(),
            password: self.password.clone(),
            subprojects: self.subprojects + rhs.subprojects,
            split_vertical: self.split_vertical,
            focused_width_percent: self.focused_width_percent,
            ..Default::default()
        }
    }
}

impl<'a> DataSerialize<Project<'a>> for Project<'a> {
    fn serialize(project: &Project) -> Result<Vec<u8>, String> {
        bincode::serialize(&project).map_err(|e| format!("failed to serialize [{e}]"))
    }
}

impl<'a> DataDeserialize<Project<'a>> for Project<'a> {
    fn deserialize(encoded: Vec<u8>) -> Result<Project<'a>, String> {
        bincode::deserialize::<Project>(encoded.as_slice())
            .map_err(|e| format!("failed to deserialize [{e}]"))
    }
}

#[derive(Serialize, Deserialize, Clone)]
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

    pub fn task(&mut self) -> Option<&mut Task> {
        self.tasks.get_item_mut(None)
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

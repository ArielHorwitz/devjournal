use super::list::SelectionList;
use crate::crypto::{decrypt, encrypt};
use crate::ui::widgets::{files::FileListWidget, prompt::PromptWidget};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};
use tui::layout::Direction;

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
    pub project_filepath: PathBuf,
    pub project: Project,
    pub project_state: ProjectState<'a>,
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
            project_password: "".to_owned(),
            focused_width_percent: DEFAULT_WIDTH_PERCENT,
            split_orientation: Direction::Horizontal,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub name: String,
    pub subprojects: SelectionList<SubProject>,
}

impl Project {
    pub fn default() -> Project {
        let mut project = Project {
            name: "New Project".to_owned(),
            subprojects: SelectionList::from_vec(vec![SubProject::default()]),
        };
        project.subprojects.select_next();
        project
    }

    pub fn from_file_encrypted(filepath: &PathBuf, key: &str) -> Result<Project, String> {
        let mut encoded: Vec<u8> = Vec::new();
        if !filepath.exists() {
            Project::default().save_file_encrypted(filepath, key)?;
        }
        File::open(filepath)
            .map_err(|e| format!("Failed to create new file [{e}]"))?
            .read_to_end(&mut encoded)
            .map_err(|e| format!("Failed to read from file [{e}]"))?;
        if !key.is_empty() {
            encoded = decrypt(&encoded, key)?;
        }
        let mut project = bincode::deserialize::<Project>(encoded.as_slice()).map_err(|e| {
            format!("Failed to deserialize - wrong password or corrupted file [{e}]")
        })?;
        for index in 0..project.len() {
            if let Some(subproject) = project.subprojects.get_item_mut(Some(index)) {
                subproject.tasks.deselect();
                subproject.tasks.select_next();
            }
        }
        project.subprojects.deselect();
        project.subprojects.select_next();
        Ok(project)
    }

    pub fn save_file_encrypted(&self, filepath: &PathBuf, key: &str) -> Result<(), String> {
        let mut encoded =
            bincode::serialize(&self).map_err(|e| format!("Failed to serialize [{e}]"))?;
        if !key.is_empty() {
            encoded = encrypt(&encoded, key)?;
        }
        let mut file =
            File::create(filepath).map_err(|e| format!("Failed to create file [{e}]"))?;
        file.write_all(&encoded)
            .map_err(|e| format!("Failed to write to file [{e}]"))?;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.subprojects.items().len()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubProject {
    pub name: String,
    pub tasks: SelectionList<Task>,
}

impl SubProject {
    pub fn default() -> SubProject {
        SubProject {
            name: "Tasks".to_owned(),
            tasks: SelectionList::new(),
        }
    }

    pub fn new(name: &str) -> SubProject {
        SubProject {
            name: name.to_owned(),
            tasks: SelectionList::new(),
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

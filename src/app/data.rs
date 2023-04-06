use super::list::SelectionList;
use crate::crypto::{decrypt, encrypt};
use crate::ui::widgets::{files::FileListWidget, prompt::PromptWidget};
use serde::{self, Deserialize, Serialize};
use std::fmt::Display;
use std::ops::Add;
use std::path::Path;
use std::time::{Duration, Instant};
use std::{fmt, fs, path::PathBuf};

pub const DEFAULT_WIDTH_PERCENT: u16 = 40;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    message: String,
    cause: Option<Box<Error>>,
}

impl std::error::Error for Error {}

impl Error {
    pub fn from_cause(message: &str, cause: Error) -> Self {
        Self {
            message: message.to_owned(),
            cause: Some(Box::new(cause)),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.cause {
            None => write!(f, "{}", &self.message),
            Some(cause) => write!(f, "{} [cause: ({})]", &self.message, cause),
        }
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self {
            message: value,
            cause: None,
        }
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self {
            message: value.to_owned(),
            cause: None,
        }
    }
}

impl From<Error> for String {
    fn from(value: Error) -> Self {
        value.message
    }
}

impl<T> From<Error> for Result<T> {
    fn from(value: Error) -> Result<T> {
        Err(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self {
            message: value.to_string(),
            cause: Some(Box::new(Error::from(value.to_string()))),
        }
    }
}

impl From<Box<bincode::ErrorKind>> for Error {
    fn from(value: Box<bincode::ErrorKind>) -> Self {
        Self {
            message: value.to_string(),
            cause: Some(Box::new(Error::from(value.to_string()))),
        }
    }
}

pub trait DataSerialize<T>
where
    Self: Serialize,
{
    fn save_encrypt(&self, filepath: &PathBuf, key: &str) -> Result<()> {
        let encoded = bincode::serialize(&self)?;
        let encrypted = encrypt(&encoded, key)?;
        fs::write(filepath, encrypted)?;
        Ok(())
    }
}

pub trait DataDeserialize<T>
where
    T: for<'a> Deserialize<'a>,
{
    fn load_decrypt(filepath: &PathBuf, key: &str) -> Result<T> {
        let encrypted = fs::read(filepath)?;
        let decrypted = decrypt(&encrypted, key)?;
        let decoded = bincode::deserialize::<T>(decrypted.as_slice())?;
        Ok(decoded)
    }
}

#[derive(Clone)]
pub enum JournalPrompt {
    SetPassword,
    RenameJournal,
    AddProject,
    RenameProject,
    AddSubProject,
    RenameSubProject,
    AddTask,
    RenameTask,
}

#[derive(Clone, Copy)]
pub enum FileRequest {
    Save,
    Load,
    LoadMerge,
}

#[derive(Clone)]
pub enum AppPrompt {
    NewJournal,
    LoadFile(String),
    MergeFile(String),
}

pub enum FeedbackKind {
    Nominal,
    Error,
}

pub struct Feedback {
    pub message: String,
    pub kind: FeedbackKind,
    pub instant: Instant,
}

impl Feedback {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_owned(),
            kind: FeedbackKind::Nominal,
            instant: Instant::now(),
        }
    }
}

impl From<Error> for Feedback {
    fn from(value: Error) -> Self {
        Self {
            message: value.message,
            kind: FeedbackKind::Error,
            instant: Instant::now(),
        }
    }
}

pub fn filename(filepath: &Path) -> String {
    filepath
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or("/missing_filename/".into())
}

pub struct App<'a> {
    pub title: &'a str,
    pub datadir: PathBuf,
    pub user_feedback_text: String,
    feedback_stack: Vec<Feedback>,
    pub filelist: FileListWidget<'a>,
    pub file_request: Option<FileRequest>,
    pub prompt: PromptWidget<'a>,
    pub prompt_request: Option<AppPrompt>,
    pub filepath: PathBuf,
    pub journal: Journal<'a>,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, datadir: PathBuf) -> App<'a> {
        App {
            title,
            datadir: datadir.clone(),
            user_feedback_text: format!("Welcome to {title}."),
            feedback_stack: vec![Feedback::new(&format!("Welcome to {title}."))],
            filelist: FileListWidget::new(datadir.to_string_lossy().to_string().as_str()),
            file_request: None,
            prompt: PromptWidget::default(),
            prompt_request: None,
            filepath: datadir.join("new_project"),
            journal: Default::default(),
        }
    }

    pub fn feedback(&self) -> Option<&Feedback> {
        if let Some(feedback) = self.feedback_stack.get(0) {
            if Instant::now() - feedback.instant <= Duration::from_millis(2000) {
                return Some(feedback);
            }
        }
        None
    }

    pub fn add_feedback(&mut self, feedback: Feedback) {
        self.feedback_stack.insert(0, feedback);
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Journal<'a> {
    pub name: String,
    pub password: String,
    pub projects: SelectionList<Project<'a>>,
}

impl<'a> Journal<'a> {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            ..Default::default()
        }
    }

    pub fn project(&mut self) -> Option<&mut Project<'a>> {
        self.projects.get_item_mut(None)
    }
}

impl<'a> Default for Journal<'a> {
    fn default() -> Self {
        let mut projects = SelectionList::from(vec![Project::default()]);
        projects.select_next();
        Journal {
            name: "New Journal".to_owned(),
            password: "".to_owned(),
            projects,
        }
    }
}

impl<'a> DataSerialize<Journal<'a>> for Journal<'a> {}

impl<'a> DataDeserialize<Journal<'a>> for Journal<'a> {}

impl<'a> From<Project<'a>> for Journal<'a> {
    fn from(project: Project<'a>) -> Self {
        Self {
            name: project.name.clone(),
            password: project.password.clone(),
            projects: SelectionList::from(vec![project]),
        }
    }
}

impl<'a> Add<Journal<'a>> for Journal<'a> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            name: self.name,
            password: self.password,
            projects: self.projects + rhs.projects,
        }
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
    pub prompt_request: Option<JournalPrompt>,
    pub focused_width_percent: u16,
    pub split_vertical: bool,
}

impl<'a> Project<'a> {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            ..Default::default()
        }
    }

    pub fn subproject(&mut self) -> Option<&mut SubProject> {
        self.subprojects.get_item_mut(None)
    }
}

impl<'a> Clone for Project<'a> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            password: self.password.clone(),
            subprojects: self.subprojects.clone(),
            split_vertical: self.split_vertical,
            focused_width_percent: self.focused_width_percent,
            ..Default::default()
        }
    }
}

impl<'a> Default for Project<'a> {
    fn default() -> Self {
        Self {
            name: "New Project".to_owned(),
            password: "".to_owned(),
            subprojects: SelectionList::from(vec![SubProject::default()]),
            prompt: PromptWidget::default().width_hint(0.7),
            prompt_request: None,
            focused_width_percent: DEFAULT_WIDTH_PERCENT,
            split_vertical: false,
        }
    }
}

impl<'a> Add<Project<'a>> for Project<'a> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            name: self.name.clone(),
            password: self.password.clone(),
            subprojects: self.subprojects + rhs.subprojects,
            split_vertical: self.split_vertical,
            focused_width_percent: self.focused_width_percent,
            ..Default::default()
        }
    }
}

impl<'a> DataSerialize<Project<'a>> for Project<'a> {}

impl<'a> DataDeserialize<Project<'a>> for Project<'a> {}

#[derive(Serialize, Deserialize, Clone)]
pub struct SubProject {
    pub name: String,
    pub tasks: SelectionList<Task>,
}

impl Default for SubProject {
    fn default() -> Self {
        Self {
            name: "Tasks".to_owned(),
            tasks: SelectionList::default(),
        }
    }
}

impl SubProject {
    pub fn new(name: &str) -> Self {
        Self {
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
    pub fn new(desc: &str) -> Self {
        Self {
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

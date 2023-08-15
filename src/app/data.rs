use super::list::SelectionList;
use crate::crypto::{decrypt, encrypt};
use crate::ui::widgets::{files::FileListWidget, prompt::PromptWidget};
use anyhow::Result;
use geckopanda::prelude::Storage;
use serde::{self, Deserialize, Serialize};
use std::fmt;
use std::ops::Add;
use std::time::{Duration, Instant};

pub const DEFAULT_WIDTH_PERCENT: u16 = 40;

pub trait SerializeEncrypt<T>
where
    Self: Serialize,
{
    fn encrypt(&self, key: &str) -> Result<Vec<u8>> {
        let encoded = bincode::serialize(&self)?;
        encrypt(&encoded, key)
    }
}

pub trait DeserializeDecrypt<T>
where
    T: for<'a> Deserialize<'a>,
{
    fn decrypt(data: &[u8], key: &str) -> Result<T> {
        let decrypted = decrypt(data, key)?;
        Ok(bincode::deserialize::<T>(&decrypted)?)
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

impl From<Box<dyn std::error::Error>> for Feedback {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Self {
            message: value.to_string(),
            kind: FeedbackKind::Error,
            instant: Instant::now(),
        }
    }
}

impl From<String> for Feedback {
    fn from(value: String) -> Self {
        Self::new(&value)
    }
}

impl From<&str> for Feedback {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

pub struct App<'a> {
    pub storage: &'a dyn Storage,
    feedback_stack: Vec<Feedback>,
    pub filelist: FileListWidget<'a>,
    pub file_request: Option<FileRequest>,
    pub prompt: PromptWidget<'a>,
    pub prompt_request: Option<AppPrompt>,
    pub filename: String,
    pub journal: Journal<'a>,
}

impl<'a> App<'a> {
    pub fn new(storage: &'a dyn Storage) -> App<'a> {
        App {
            storage,
            feedback_stack: vec![Feedback::new("Welcome to Dev Journal")],
            filelist: FileListWidget::new(storage),
            file_request: None,
            prompt: PromptWidget::default(),
            prompt_request: None,
            filename: "new_journal".to_owned(),
            journal: Default::default(),
        }
    }

    pub fn feedback(&self) -> Option<&Feedback> {
        if let Some(feedback) = self.feedback_stack.first() {
            let show_duration = match feedback.kind {
                FeedbackKind::Nominal => 1250,
                FeedbackKind::Error => 5000,
            };
            if Instant::now() - feedback.instant <= Duration::from_millis(show_duration) {
                return Some(feedback);
            }
        };
        None
    }

    pub fn add_feedback<F>(&mut self, feedback: F)
    where
        F: Into<Feedback>,
    {
        self.feedback_stack.insert(0, feedback.into());
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

impl<'a> SerializeEncrypt<Journal<'a>> for Journal<'a> {}

impl<'a> DeserializeDecrypt<Journal<'a>> for Journal<'a> {}

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

impl<'a> SerializeEncrypt<Project<'a>> for Project<'a> {}

impl<'a> DeserializeDecrypt<Project<'a>> for Project<'a> {}

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

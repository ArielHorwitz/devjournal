use crate::crypto::{decrypt, encrypt};
use crate::ui::widgets::{files::FileListWidget, prompt::PromptWidget};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub const DEFAULT_WIDTH_PERCENT: u16 = 40;

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

pub fn filename(filepath: &Path) -> String {
    filepath
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or("/missing_filename/".into())
}

pub struct App<'a> {
    pub datadir: PathBuf,
    feedback_stack: Vec<Feedback>,
    pub filelist: FileListWidget<'a>,
    pub file_request: Option<FileRequest>,
    pub prompt: PromptWidget<'a>,
    pub prompt_request: Option<AppPrompt>,
    pub filepath: PathBuf,
    pub journal: Journal<'a>,
}

impl<'a> App<'a> {
    pub fn new(datadir: PathBuf) -> App<'a> {
        App {
            datadir: datadir.clone(),
            feedback_stack: vec![Feedback::new("Welcome to Dev Journal")],
            filelist: FileListWidget::new(datadir.to_string_lossy().to_string().as_str()),
            file_request: None,
            prompt: PromptWidget::default(),
            prompt_request: None,
            filepath: datadir.join("new_journal"),
            journal: Default::default(),
        }
    }

    pub fn feedback(&self) -> Option<&Feedback> {
        if let Some(feedback) = self.feedback_stack.get(0) {
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


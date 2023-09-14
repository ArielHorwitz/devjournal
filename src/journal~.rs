use crate::app::list::SelectionList;
use std::ops::Add;
use std::{fmt, fs, path::PathBuf};

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

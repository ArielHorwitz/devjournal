use super::list::InteractiveList;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task {
    pub desc: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

impl Task {
    pub fn new(desc: &str) -> Task {
        Task {
            desc: desc.to_string(),
            created_at: "2020-02-02 12:00:00".to_string(),
            completed_at: None,
        }
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&format!("{}", self.desc))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubProject {
    pub name: String,
    pub tasks: InteractiveList<Task>,
}

impl SubProject {
    pub fn default() -> SubProject {
        SubProject {
            name: "Tasks".to_string(),
            tasks: InteractiveList::new(),
        }
    }

    pub fn new(name: &str) -> SubProject {
        SubProject {
            name: name.to_string(),
            tasks: InteractiveList::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub name: String,
    pub subprojects: InteractiveList<SubProject>,
}

impl Project {
    pub fn default() -> Project {
        let mut project = Project {
            name: "New Project".to_string(),
            subprojects: InteractiveList::from_vec(vec![SubProject::default()]),
        };
        project.subprojects.select_next();
        project
    }

    pub fn new(name: &str, subproject_name: &str) -> Project {
        let mut project = Project {
            name: name.to_string(),
            subprojects: InteractiveList::from_vec(vec![SubProject::new(subproject_name)]),
        };
        project.subprojects.select_next();
        project
    }

    pub fn from_file(filepath: &PathBuf) -> Project {
        let mut encoded: Vec<u8> = Vec::new();
        if !filepath.exists() {
            Project::default().save_file(filepath);
        }
        File::open(filepath)
            .unwrap()
            .read_to_end(&mut encoded)
            .unwrap();
        let mut project = bincode::deserialize::<Project>(encoded.as_slice()).unwrap();
        for index in 0..project.len() {
            project.subprojects.get_value(index).tasks.deselect();
        }
        project.subprojects.deselect();
        project.subprojects.select_next();
        project
    }

    pub fn save_file(&self, filepath: &PathBuf) {
        let encoded = bincode::serialize(&self).unwrap();
        let mut file = File::create(&filepath).unwrap();
        file.write_all(&encoded).unwrap();
    }

    pub fn len(&self) -> usize {
        self.subprojects.len()
    }
}
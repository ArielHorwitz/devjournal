use super::list::SelectionList;
use crate::crypto::{decrypt, encrypt};
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
    pub tasks: SelectionList<Task>,
}

impl SubProject {
    pub fn default() -> SubProject {
        SubProject {
            name: "Tasks".to_string(),
            tasks: SelectionList::new(),
        }
    }

    pub fn new(name: &str) -> SubProject {
        SubProject {
            name: name.to_string(),
            tasks: SelectionList::new(),
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
            name: "New Project".to_string(),
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
            .map_err(|e| format!("Failed to create new file [{}]", e.to_string()))?
            .read_to_end(&mut encoded)
            .map_err(|e| format!("Failed to read from file [{}]", e.to_string()))?;
        if key.len() > 0 {
            encoded = decrypt(&encoded, key)?;
        }
        let mut project = bincode::deserialize::<Project>(encoded.as_slice()).map_err(|e| {
            format!(
                "Failed to deserialize - wrong password or corrupted file [{}]",
                e.to_string()
            )
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
        let mut encoded = bincode::serialize(&self)
            .map_err(|e| format!("Failed to serialize [{}]", e.to_string()))?;
        if key.len() > 0 {
            encoded = encrypt(&encoded, key)?;
        }
        let mut file = File::create(&filepath)
            .map_err(|e| format!("Failed to create file [{}]", e.to_string()))?;
        file.write_all(&encoded)
            .map_err(|e| format!("Failed to write to file [{}]", e.to_string()))?;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.subprojects.items().len()
    }
}

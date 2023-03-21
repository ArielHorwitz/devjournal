use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
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
    pub tasks: List<Task>,
}

impl SubProject {
    pub fn new(name: &str) -> SubProject {
        SubProject {
            name: name.to_string(),
            tasks: List::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub name: String,
    pub subprojects: Vec<SubProject>,
}

impl Project {
    pub fn new(name: &str, subproject_name: &str) -> Project {
        Project {
            name: name.to_string(),
            subprojects: vec![SubProject::new(subproject_name)],
        }
    }

    pub fn from_file(filepath: &PathBuf) -> Project {
        let mut encoded: Vec<u8> = Vec::new();
        File::open(filepath)
            .unwrap()
            .read_to_end(&mut encoded)
            .unwrap();
        bincode::deserialize::<Project>(encoded.as_slice()).unwrap()
    }

    pub fn save_file(&self, filepath: &PathBuf) {
        let encoded = bincode::serialize(&self).unwrap();
        let mut file = File::create(&filepath).unwrap();
        file.write_all(&encoded).unwrap();
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct List<T>
where
    T: Clone + Display,
{
    items: Vec<T>,
    selection: Option<usize>,
}

impl<T> List<T>
where
    T: Clone + Display,
{
    pub fn new() -> List<T> {
        List {
            items: Vec::default(),
            selection: None,
        }
    }

    pub fn as_strings(&self) -> Vec<String> {
        self.items.iter().map(|t| t.to_string()).collect()
    }

    pub fn add_item(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn clear_items(&mut self) {
        self.items = Vec::default();
    }

    pub fn selected(&self) -> Option<usize> {
        self.selection
    }

    pub fn selected_value(&self) -> Option<&T> {
        match self.selection {
            None => None,
            Some(index) => Some(&self.items[index]),
        }
    }

    pub fn deselect(&mut self) {
        self.selection = None;
    }

    pub fn select_next(&mut self) {
        self.selection = match self.selection {
            None => {
                if self.items.len() == 0 {
                    None
                } else {
                    Some(0)
                }
            }
            Some(index) => {
                if self.items.len() == index + 1 {
                    Some(0)
                } else {
                    Some(index + 1)
                }
            }
        }
    }

    pub fn select_prev(&mut self) {
        self.selection = match self.selection {
            None => {
                if self.items.len() == 0 {
                    None
                } else {
                    Some(self.items.len() - 1)
                }
            }
            Some(i) => {
                if i == 0 {
                    Some(self.items.len() - 1)
                } else {
                    Some(i - 1)
                }
            }
        }
    }

    pub fn move_up(&mut self) -> Result<(), String> {
        if let Some(selected) = self.selection {
            if selected > 0 {
                self.items.swap(selected, selected - 1);
                self.selection = Some(selected - 1);
            } else {
                let element = self.items.remove(selected);
                self.items.push(element);
                self.selection = Some(self.items.len() - 1);
            }
            Ok(())
        } else {
            Err("no item selected".to_string())
        }
    }

    pub fn move_down(&mut self) -> Result<(), String> {
        if let Some(selected) = self.selection {
            if selected < self.items.len() - 1 {
                self.items.swap(selected, selected + 1);
                self.selection = Some(selected + 1);
            } else {
                let element = self.items.pop().unwrap();
                self.items.insert(0, element);
                self.selection = Some(0);
            }
            Ok(())
        } else {
            Err("no item selected".to_string())
        }
    }

    pub fn replace_selected(&mut self, new: T) -> Option<T> {
        if let Some(index) = self.selection {
            let element = self.items.remove(index);
            self.items.insert(index, new);
            Some(element)
        } else {
            None
        }
    }

    pub fn pop_selected(&mut self) -> Option<T> {
        match self.selection {
            None => None,
            Some(index) => {
                let result = self.items.remove(index);
                if self.items.len() == 0 {
                    self.selection = None;
                } else if index >= self.items.len() {
                    self.selection = Some(self.items.len() - 1);
                }
                Some(result)
            }
        }
    }
}

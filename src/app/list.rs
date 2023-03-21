use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InteractiveList<T> {
    items: Vec<T>,
    selection: Option<usize>,
}

impl<T> InteractiveList<T> {
    pub fn new() -> InteractiveList<T> {
        InteractiveList {
            items: Vec::default(),
            selection: None,
        }
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

impl<T> InteractiveList<T>
where
    T: Display,
{
    pub fn as_strings(&self) -> Vec<String> {
        self.items.iter().map(|t| t.to_string()).collect()
    }
}

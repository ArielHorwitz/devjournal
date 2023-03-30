use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SelectionList<T> {
    items: Vec<T>,
    selection: Option<usize>,
}

impl<T> SelectionList<T> {
    pub fn new() -> SelectionList<T> {
        SelectionList {
            items: Vec::default(),
            selection: None,
        }
    }

    pub fn from_vec(vec: Vec<T>) -> SelectionList<T> {
        SelectionList {
            items: vec,
            selection: None,
        }
    }

    pub fn items(&self) -> &Vec<T> {
        &self.items
    }

    pub fn get_item(&self, index: Option<usize>) -> Option<&T> {
        match index {
            Some(i) => {
                if i < self.items.len() {
                    Some(&self.items[i])
                } else {
                    None
                }
            }
            None => match self.selection {
                Some(i) => Some(&self.items[i]),
                None => None,
            },
        }
    }

    pub fn get_item_mut(&mut self, index: Option<usize>) -> Option<&mut T> {
        match index {
            Some(i) => {
                if i < self.items.len() {
                    Some(&mut self.items[i])
                } else {
                    None
                }
            }
            None => match self.selection {
                Some(i) => Some(&mut self.items[i]),
                None => None,
            },
        }
    }

    pub fn add_item(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn insert_item(&mut self, index: Option<usize>, item: T, select: bool) {
        let index = index.unwrap_or(0);
        self.items.insert(index, item);
        if select {
            self.selection = Some(index);
        }
    }

    pub fn clear_items(&mut self) {
        self.items = Vec::default();
    }

    pub fn selected(&self) -> Option<usize> {
        self.selection
    }

    pub fn prev_index(&self) -> Option<usize> {
        if let Some(index) = self.selection {
            if index == 0 {
                Some(self.items.len() - 1)
            } else {
                Some(index - 1)
            }
        } else if !self.items.is_empty() {
            Some(self.items.len() - 1)
        } else {
            None
        }
    }

    pub fn next_index(&self) -> Option<usize> {
        if let Some(index) = self.selection {
            if self.items.len() == index + 1 {
                Some(0)
            } else {
                Some(index + 1)
            }
        } else if !self.items.is_empty() {
            Some(0)
        } else {
            None
        }
    }

    pub fn next_item_mut(&mut self) -> Option<&mut T> {
        self.get_item_mut(self.next_index())
    }

    pub fn prev_item_mut(&mut self) -> Option<&mut T> {
        self.get_item_mut(self.prev_index())
    }

    pub fn deselect(&mut self) {
        self.selection = None;
    }

    pub fn select_next(&mut self) {
        self.selection = self.next_index()
    }

    pub fn select_prev(&mut self) {
        self.selection = self.prev_index()
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
            Err("no item selected".to_owned())
        }
    }

    pub fn move_down(&mut self) -> Result<(), String> {
        if let Some(selected) = self.selection {
            if selected < self.items.len() - 1 {
                self.items.swap(selected, selected + 1);
                self.selection = Some(selected + 1);
            } else {
                let element = self
                    .items
                    .pop()
                    .ok_or("selection is Some, should have at least one item")?;
                self.items.insert(0, element);
                self.selection = Some(0);
            }
            Ok(())
        } else {
            Err("no item selected".to_owned())
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
                if self.items.is_empty() {
                    self.selection = None;
                } else if index >= self.items.len() {
                    self.selection = Some(self.items.len() - 1);
                }
                Some(result)
            }
        }
    }
}

impl<T> SelectionList<T>
where
    T: Display,
{
    pub fn as_strings(&self) -> Vec<String> {
        self.items.iter().map(|t| t.to_string()).collect()
    }
}

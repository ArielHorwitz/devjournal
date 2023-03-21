use serde::{Deserialize, Serialize};
use std::{fmt::Display, slice::Iter};

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

    pub fn from_vec(vec: Vec<T>) -> InteractiveList<T> {
        InteractiveList {
            items: vec,
            selection: None,
        }
    }

    pub fn iter(&self) -> Iter<T> {
        self.items.iter()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn add_item(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn insert_item(&mut self, index: Option<usize>, item: T, select: bool) {
        let index = match index {
            Some(i) => i,
            None => 0,
        };
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
        } else if self.items.len() > 0 {
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
        } else if self.items.len() > 0 {
            Some(0)
        } else {
            None
        }
    }

    pub fn selected_value(&mut self) -> Option<&mut T> {
        match self.selection {
            None => None,
            Some(index) => Some(&mut self.items[index]),
        }
    }

    pub fn next_value(&mut self) -> Option<&mut T> {
        if let Some(index) = self.next_index() {
            return Some(&mut self.items[index]);
        }
        None
    }

    pub fn prev_value(&mut self) -> Option<&mut T> {
        if let Some(index) = self.prev_index() {
            return Some(&mut self.items[index]);
        }
        None
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

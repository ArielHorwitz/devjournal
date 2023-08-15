use crate::app::data::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops::Add, slice::Iter};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SelectionList<T> {
    items: Vec<T>,
    selection: Option<usize>,
}

impl<T> Default for SelectionList<T> {
    fn default() -> SelectionList<T> {
        SelectionList {
            items: Vec::default(),
            selection: None,
        }
    }
}

impl<T> From<Vec<T>> for SelectionList<T> {
    fn from(vec: Vec<T>) -> Self {
        SelectionList {
            items: vec,
            selection: None,
        }
    }
}

impl<T> SelectionList<T> {
    pub fn iter(&self) -> Iter<'_, T> {
        self.items.iter()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn get_item(&self, index: Option<usize>) -> Option<&T> {
        match index {
            Some(i) => self.items.get(i),
            None => match self.selection {
                Some(i) => self.items.get(i),
                None => None,
            },
        }
    }

    pub fn get_item_mut(&mut self, index: Option<usize>) -> Option<&mut T> {
        match index {
            Some(i) => self.items.get_mut(i),
            None => match self.selection {
                Some(i) => self.items.get_mut(i),
                None => None,
            },
        }
    }

    pub fn push_item(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn add_item(&mut self, item: T, select: bool) {
        match self.selection {
            Some(index) => match index >= self.len() - 1 {
                true => {
                    self.push_item(item);
                    if select {
                        self.selection = Some(self.len() - 1);
                    };
                }
                false => {
                    self.insert_item(Some(index + 1), item, false);
                    if select {
                        self.selection = Some(index + 1);
                    }
                }
            },
            None => {
                self.push_item(item);
                if select {
                    self.selection = Some(self.len() - 1);
                }
            }
        }
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

    pub fn selected(&self) -> Option<&T> {
        self.get_item(None)
    }

    pub fn selection(&self) -> Option<usize> {
        self.selection
    }

    pub fn select(&mut self, index: usize) -> Result<()> {
        if index >= self.items.len() {
            return Err(Error::from("index out of range"));
        };
        self.selection = Some(index);
        Ok(())
    }

    pub fn deselect(&mut self) {
        self.selection = None;
    }

    pub fn next_index(&self) -> Option<usize> {
        match self.selection {
            None => match self.items.is_empty() {
                true => None,
                false => Some(0),
            },
            Some(index) => match index + 1 >= self.len() {
                true => Some(0),
                false => Some(index + 1),
            },
        }
    }

    pub fn prev_index(&self) -> Option<usize> {
        match self.selection {
            None => match self.items.is_empty() {
                true => None,
                false => Some(self.len() - 1),
            },
            Some(index) => match index == 0 {
                true => Some(self.len() - 1),
                false => Some(index - 1),
            },
        }
    }

    pub fn select_next(&mut self) {
        self.selection = self.next_index()
    }

    pub fn select_prev(&mut self) {
        self.selection = self.prev_index()
    }

    pub fn shift_next(&mut self) -> Result<usize> {
        if let Some(selected) = self.selection {
            if selected < self.items.len() - 1 {
                let new_index = selected + 1;
                self.items.swap(selected, new_index);
                self.selection = Some(new_index);
                Ok(new_index)
            } else {
                let element = self.items.pop().ok_or(Error::from(
                    "selection is Some, should have at least one item",
                ))?;
                self.items.insert(0, element);
                self.selection = Some(0);
                Ok(0)
            }
        } else {
            Err(Error::from("no item selected"))
        }
    }

    pub fn shift_prev(&mut self) -> Result<usize> {
        if let Some(selected) = self.selection {
            if selected > 0 {
                let new_index = selected - 1;
                self.items.swap(selected, new_index);
                self.selection = Some(new_index);
                Ok(new_index)
            } else {
                let element = self.items.remove(selected);
                self.items.push(element);
                self.selection = Some(self.items.len() - 1);
                Ok(self.items.len() - 1)
            }
        } else {
            Err(Error::from("no item selected"))
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

impl<T> Add<SelectionList<T>> for SelectionList<T>
where
    T: Clone,
{
    type Output = SelectionList<T>;

    fn add(self, rhs: SelectionList<T>) -> Self::Output {
        let mut items = self.items;
        let mut rhs_items = rhs.items;
        items.append(&mut rhs_items);
        SelectionList::from(items)
    }
}

impl<T> SelectionList<T>
where
    T: Display,
{
    pub fn as_strings(&self) -> Vec<String> {
        self.iter().map(|t| t.to_string()).collect()
    }
}

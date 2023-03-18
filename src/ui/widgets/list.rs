use std::fmt::Display;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Spans,
    widgets::{Block, Widget},
};

#[derive(Serialize, Deserialize)]
pub struct List<T>
where
    T: Display,
{
    items: Vec<T>,
    selection: Option<usize>,
}

impl<T> List<T>
where
    T: Display,
{
    pub fn default() -> List<T> {
        List {
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

    pub fn widget(&self) -> ListWidget {
        ListWidget::new(
            self.items.iter().map(|x| x.to_string()).collect(),
            self.selection,
        )
    }

    pub fn handle_event(&mut self, key: KeyEvent) -> Result<(), ()> {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => self.deselect(),
            (KeyCode::Up, KeyModifiers::NONE) => self.select_prev(),
            (KeyCode::Down, KeyModifiers::NONE) => self.select_next(),
            (KeyCode::Up, KeyModifiers::CONTROL) => self.move_up().unwrap_or(()),
            (KeyCode::Down, KeyModifiers::CONTROL) => self.move_down().unwrap_or(()),
            (KeyCode::Char('k'), KeyModifiers::NONE) => self.select_prev(),
            (KeyCode::Char('j'), KeyModifiers::NONE) => self.select_next(),
            (KeyCode::Char('k'), KeyModifiers::CONTROL) => self.move_up().unwrap_or(()),
            (KeyCode::Char('j'), KeyModifiers::CONTROL) => self.move_down().unwrap_or(()),
            (KeyCode::Delete, KeyModifiers::NONE) => {
                self.pop_selected();
            }
            (KeyCode::Delete, KeyModifiers::CONTROL) => self.clear_items(),
            _ => return Err(()),
        };
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ListWidget<'a> {
    /// A block to wrap this widget in
    block: Option<Block<'a>>,
    /// Items
    items: Vec<String>,
    /// Item to highlight
    selected: Option<usize>,
    /// Style to for item text
    style: Style,
    /// Style for selected item text
    style_selected: Style,
    /// Bullet point for items
    bullet: char,
    /// Bullet point for selected item
    bullet_selected: char,
}

#[allow(dead_code)]
impl<'a> ListWidget<'a> {
    pub fn new(items: Vec<String>, highlighted: Option<usize>) -> ListWidget<'a> {
        ListWidget {
            block: None,
            style: Default::default(),
            items,
            selected: highlighted,
            style_selected: Default::default(),
            bullet: '•',
            bullet_selected: '►',
        }
    }

    pub fn block(mut self, block: Block<'a>) -> ListWidget<'a> {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> ListWidget<'a> {
        self.style = style;
        self
    }

    pub fn highlight_style(mut self, style: Style) -> ListWidget<'a> {
        self.style_selected = style;
        self
    }

    pub fn bullet(mut self, bullet: char) -> ListWidget<'a> {
        self.bullet = bullet;
        self
    }

    pub fn bullet_selected(mut self, bullet: char) -> ListWidget<'a> {
        self.bullet_selected = bullet;
        self
    }
}

impl<'a> Widget for ListWidget<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);
        let area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        if area.height < 1 {
            return;
        }

        let x = area.left();
        let mut y = area.top();
        let width = area.width;
        for (i, text) in self.items.iter().enumerate() {
            let mut style = self.style;
            let mut text = text.clone();
            if self.selected == Some(i) {
                style = self.style_selected;
                text = format!("{} {}", self.bullet_selected, text);
            } else {
                text = format!("{} {}", self.bullet, text);
            }
            buf.set_spans(x, y, &Spans::from(text), width);
            buf.set_style(Rect::new(x, y, width, 1), style);
            y += 1;
        }
    }
}

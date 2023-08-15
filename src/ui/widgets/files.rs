use super::{list::ListWidget, prompt::PromptWidget};
use crate::{app::list::SelectionList, ui::styles};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use geckopanda::prelude::Storage;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::Span,
    widgets::{Block, Borders, Clear},
    Frame,
};

pub enum FileListResult {
    AwaitingResult,
    Feedback(String),
    Result(String),
    Cancelled,
}

enum Focus {
    FileList,
    Prompt,
}

pub struct FileListWidget<'a> {
    prompt: PromptWidget<'a>,
    storage: &'a dyn Storage,
    filelist: SelectionList<String>,
    focus: Focus,
    title: String,
    style_title: Style,
    style_border: Style,
}

impl<'a> FileListWidget<'a> {
    pub fn new(storage: &'a dyn Storage) -> FileListWidget<'a> {
        let mut widget = FileListWidget {
            prompt: PromptWidget::default().focus(false).margin(0),
            storage,
            filelist: SelectionList::default(),
            focus: Focus::FileList,
            title: "Files".to_owned(),
            style_title: styles::title(),
            style_border: styles::border_highlighted(),
        };
        widget.reset();
        widget.filelist.select_next();
        widget
    }

    pub fn set_title_text(&mut self, text: &str) {
        self.title = text.to_owned();
    }

    pub fn reset(&mut self) {
        self.set_focus(Focus::FileList);
        self.filelist.clear_items();
        let mut files = self.storage.list_blocking().expect("failed to list files");
        files.sort_by_key(|metadata| metadata.last_modified.clone());
        for file in files.iter() {
            self.filelist.push_item(file.name.clone());
        }
    }

    pub fn set_prompt_text(&mut self, text: &str) {
        self.prompt.set_prompt_text(text);
    }

    pub fn draw<B: Backend>(&self, f: &mut Frame<B>, chunk: Rect) {
        f.render_widget(Clear, chunk);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(chunk.height.saturating_sub(3)),
                Constraint::Length(3),
            ])
            .split(chunk);
        let file_list = ListWidget::new(self.filelist.as_strings(), self.filelist.selection())
            .block(
                Block::default()
                    .title(Span::styled(&self.title, self.style_title))
                    .borders(Borders::ALL)
                    .border_style(self.style_border),
            )
            .focus(matches!(&self.focus, Focus::FileList));
        f.render_widget(file_list, *chunks.first().expect("missing chunk"));
        self.prompt.draw(f, *chunks.get(1).expect("missing chunk"));
    }

    pub fn handle_event(&mut self, key: KeyEvent) -> FileListResult {
        match self.handle_event_globals(key) {
            FileListResult::AwaitingResult => match self.focus {
                Focus::FileList => self.handle_event_list(key),
                Focus::Prompt => self.handle_event_prompt(key),
            },
            result => result,
        }
    }

    fn handle_event_globals(&mut self, key: KeyEvent) -> FileListResult {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, KeyModifiers::NONE) => FileListResult::Cancelled,
            (KeyCode::F(5), KeyModifiers::NONE) => {
                self.reset();
                FileListResult::AwaitingResult
            }
            _ => FileListResult::AwaitingResult,
        }
    }

    fn set_focus(&mut self, focus: Focus) {
        self.focus = focus;
        self.style_title = match &self.focus {
            Focus::FileList => styles::title(),
            _ => styles::title_dim(),
        };
        self.style_border = match &self.focus {
            Focus::FileList => styles::border_highlighted(),
            _ => styles::border(),
        };
        self.prompt.set_focus(matches!(&self.focus, Focus::Prompt));
    }

    fn handle_event_list(&mut self, key: KeyEvent) -> FileListResult {
        match (key.code, key.modifiers) {
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.set_focus(Focus::Prompt);
                return FileListResult::AwaitingResult;
            }
            (KeyCode::Down, KeyModifiers::NONE) => self.filelist.select_next(),
            (KeyCode::Up, KeyModifiers::NONE) => self.filelist.select_prev(),
            (KeyCode::Char('d'), KeyModifiers::NONE) => {
                if let Some(name) = self.filelist.pop_selected() {
                    if let Err(e) = self.storage.delete_blocking(name.as_str()) {
                        self.filelist.push_item(name);
                        return FileListResult::Feedback(format!("failed to remove file: {e}"));
                    };
                    self.reset();
                    return FileListResult::Feedback(format!("Deleted project file: {name}"));
                }
            }
            (KeyCode::Enter, KeyModifiers::NONE) => {
                if let Some(filename) = self.filelist.selected() {
                    return FileListResult::Result(filename.clone());
                }
            }
            _ => return FileListResult::AwaitingResult,
        }
        FileListResult::AwaitingResult
    }

    fn handle_event_prompt(&mut self, key: KeyEvent) -> FileListResult {
        match (key.code, key.modifiers) {
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.set_focus(Focus::FileList);
                FileListResult::AwaitingResult
            }
            (KeyCode::Enter, KeyModifiers::NONE) => {
                let result_text = self.prompt.get_text();
                self.prompt.set_text("");
                FileListResult::Result(result_text)
            }
            _ => {
                self.prompt.handle_event(key);
                FileListResult::AwaitingResult
            }
        }
    }
}

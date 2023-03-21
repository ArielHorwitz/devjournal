use super::{list::ListWidget, prompt::PromptWidget};
use crate::{app::project::List, ui::styles};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use pathdiff::diff_paths;
use std::{
    fs::{read_dir, remove_file},
    io,
    path::PathBuf,
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
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
    datadir: String,
    filelist: List<String>,
    focus: Focus,
    title: String,
}

impl<'a> FileListWidget<'a> {
    pub fn new(datadir: &str) -> FileListWidget<'a> {
        let mut widget = FileListWidget {
            prompt: PromptWidget::default().width_hint(1.).margin(0),
            datadir: datadir.to_string(),
            filelist: List::new(),
            focus: Focus::FileList,
            title: "Files".to_string(),
        };
        widget.refresh_filelist();
        widget.filelist.select_next();
        widget
    }

    pub fn set_title_text(&mut self, text: &str) {
        self.title = text.to_string();
    }

    pub fn refresh_filelist(&mut self) {
        let mut entries = read_dir(&self.datadir)
            .unwrap()
            .map(|res| {
                res.map(|e| {
                    diff_paths(e.path(), &self.datadir)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string()
                })
            })
            .filter(|x| match x {
                Err(_) => false,
                Ok(s) => !s.ends_with(".config"),
            })
            .collect::<Result<Vec<String>, io::Error>>()
            .unwrap();
        entries.sort();
        self.filelist.clear_items();
        for file in entries {
            self.filelist.add_item(file);
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
        let files_title_style = match self.focus {
            Focus::FileList => styles::title_highlighted(),
            _ => styles::title(),
        };
        let file_list = ListWidget::new(self.filelist.as_strings(), self.filelist.selected())
            .block(
                Block::default()
                    .title(Span::styled(&self.title, files_title_style))
                    .borders(Borders::ALL)
                    .border_style(styles::border_highlighted()),
            )
            .style(styles::list_normal())
            .highlight_style(styles::list_highlight());
        f.render_widget(file_list, chunks[0]);
        self.prompt.draw(f, chunks[1]);
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

    pub fn handle_event_globals(&mut self, key: KeyEvent) -> FileListResult {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, KeyModifiers::NONE) => {
                return FileListResult::Cancelled;
            }
            (KeyCode::F(5), KeyModifiers::NONE) => {
                self.refresh_filelist();
                FileListResult::AwaitingResult
            }
            _ => return FileListResult::AwaitingResult,
        }
    }

    pub fn handle_event_list(&mut self, key: KeyEvent) -> FileListResult {
        match (key.code, key.modifiers) {
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.focus = Focus::Prompt;
                return FileListResult::AwaitingResult;
            }
            (KeyCode::Char('j'), KeyModifiers::NONE) => self.filelist.select_next(),
            (KeyCode::Char('k'), KeyModifiers::NONE) => self.filelist.select_prev(),
            (KeyCode::Char('d'), KeyModifiers::NONE) => {
                if let Some(name) = self.filelist.pop_selected() {
                    remove_file(PathBuf::from(&self.datadir).join(&name)).unwrap();
                    self.refresh_filelist();
                    return FileListResult::Feedback(format!("Deleted project file: {name}"));
                }
            }
            (KeyCode::Enter, KeyModifiers::NONE) => {
                if let Some(filename) = self.filelist.selected_value() {
                    return FileListResult::Result(filename.clone());
                };
            }
            _ => return FileListResult::AwaitingResult,
        }
        FileListResult::AwaitingResult
    }

    pub fn handle_event_prompt(&mut self, key: KeyEvent) -> FileListResult {
        match (key.code, key.modifiers) {
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.focus = Focus::FileList;
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

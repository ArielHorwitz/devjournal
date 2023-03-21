use super::{list::ListWidget, prompt::PromptWidget};
use crate::{app::list::InteractiveList, ui::styles};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{
    fs::{self, read_dir, remove_file},
    path::PathBuf,
};
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
    datadir: String,
    filelist: InteractiveList<String>,
    focus: Focus,
    title: String,
    style_title: Style,
    style_border: Style,
}

impl<'a> FileListWidget<'a> {
    pub fn new(datadir: &str) -> FileListWidget<'a> {
        let mut widget = FileListWidget {
            prompt: PromptWidget::default().focus(false).margin(0),
            datadir: datadir.to_string(),
            filelist: InteractiveList::new(),
            focus: Focus::FileList,
            title: "Files".to_string(),
            style_title: styles::title(),
            style_border: styles::border_highlighted(),
        };
        widget.refresh_filelist();
        widget.filelist.select_next();
        widget
    }

    pub fn set_title_text(&mut self, text: &str) {
        self.title = text.to_string();
    }

    pub fn refresh_filelist(&mut self) {
        let mut entries: Vec<PathBuf> = read_dir(&self.datadir)
            .unwrap()
            .map(|res| res.unwrap().path())
            .filter(|x| x.is_file() && !x.ends_with(".config"))
            .collect();
        entries.sort_by_key(|file| {
            fs::metadata(file)
                .unwrap()
                .modified()
                .unwrap()
                .elapsed()
                .unwrap()
        });
        self.filelist.clear_items();
        for file in entries {
            self.filelist
                .add_item(file.file_name().unwrap().to_str().unwrap().to_string());
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
        let file_list = ListWidget::new(self.filelist.as_strings(), self.filelist.selected())
            .block(
                Block::default()
                    .title(Span::styled(&self.title, self.style_title))
                    .borders(Borders::ALL)
                    .border_style(self.style_border),
            )
            .focus(match &self.focus {
                Focus::FileList => true,
                _ => false,
            });
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
        self.prompt.set_focus(match &self.focus {
            Focus::Prompt => true,
            _ => false,
        });
    }

    pub fn handle_event_list(&mut self, key: KeyEvent) -> FileListResult {
        match (key.code, key.modifiers) {
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.set_focus(Focus::Prompt);
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

use super::{list::ListWidget, prompt::PromptWidget};
use crate::{app::project::List, ui::styles};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use pathdiff::diff_paths;
use std::{fs, io};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    text::Span,
    widgets::{Block, Borders, Clear},
    Frame,
};

pub struct FileListWidget<'a> {
    prompt: PromptWidget<'a>,
    datadir: String,
    filelist: List<String>,
    focus: Focus,
}

impl<'a> FileListWidget<'a> {
    pub fn new(datadir: &str) -> FileListWidget<'a> {
        FileListWidget {
            prompt: PromptWidget::default(),
            datadir: datadir.to_string(),
            filelist: List::new(),
            focus: Focus::FileList,
        }
    }

    pub fn refresh_filelist(&mut self) {
        let mut entries = fs::read_dir(&self.datadir)
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

    pub fn draw<B: Backend>(&self, f: &mut Frame<B>, chunk: Rect) {
        f.render_widget(Clear, chunk);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100), Constraint::Length(3)])
            .split(chunk);
        let file_list = ListWidget::new(self.filelist.as_strings(), self.filelist.selected())
            .block(
                Block::default()
                    .title(Span::styled("Files", styles::title_highlighted()))
                    .borders(Borders::ALL)
                    .border_style(styles::border_highlighted()),
            )
            .style(styles::list_normal())
            .highlight_style(styles::list_highlight());
        f.render_widget(file_list, chunks[0]);
        self.prompt.draw(f, chunks[1]);
    }

    pub fn handle_event(&mut self, key: KeyEvent) -> FileListResult {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, KeyModifiers::NONE) => {
                return FileListResult::Cancelled;
            }
            (KeyCode::F(5), KeyModifiers::NONE) => {
                self.refresh_filelist();
                return FileListResult::Feedback("Refreshed file list.".to_string());
            }
            (KeyCode::Char('j'), KeyModifiers::NONE) => self.filelist.select_next(),
            (KeyCode::Char('k'), KeyModifiers::NONE) => self.filelist.select_prev(),
            (KeyCode::Char('d'), KeyModifiers::NONE) => {
                if let Some(filename) = self.filelist.pop_selected() {
                    // TODO delete file
                    return FileListResult::Feedback(format!("Deleted file: {filename}"));
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
}

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

fn get_filelist(datadir: &str) -> io::Result<Vec<String>> {
    let mut entries = fs::read_dir(datadir)?
        .map(|res| {
            res.map(|e| {
                diff_paths(e.path(), datadir)
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
        .collect::<Result<Vec<_>, io::Error>>()?;
    entries.sort();
    Ok(entries)
}

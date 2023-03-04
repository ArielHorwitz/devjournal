/// App state and logic
use crate::ui;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use pathdiff::diff_paths;
use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::{
    fs::File,
    io::{self, Read},
    time::{Duration, Instant},
};
use tui::widgets::ListState;
use tui::{backend::Backend, Terminal};
use tui_textarea::{CursorMove, TextArea};

const TICK_RATE_MS: u64 = 25;
const LAST_TAB_INDEX: usize = 1;

#[derive(Debug, Clone)]
pub enum PromptHandler {
    AddTask,
    SaveFile,
    LoadFile,
}

impl fmt::Display for PromptHandler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Serialize, Deserialize, Debug)]
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

pub struct App<'a> {
    pub title: &'a str,
    pub datadir: PathBuf,
    pub should_quit: bool,
    pub tab_index: usize,
    pub tick: i32,
    pub textarea: TextArea<'a>,
    pub prompt_handler: Option<PromptHandler>,
    pub user_feedback_text: String,
    pub help_text: String,
    pub task_list: Vec<Task>,
    pub task_selection: ListState,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, datadir: PathBuf) -> App<'a> {
        App {
            title,
            datadir,
            should_quit: false,
            tab_index: 0,
            tick: 0,
            textarea: TextArea::default(),
            prompt_handler: None,
            user_feedback_text: "".to_string(),
            help_text: "".to_string(),
            task_list: Vec::new(),
            task_selection: ListState::default(),
        }
    }

    pub fn set_feedback_text(&mut self, text: &str) {
        self.user_feedback_text = text.to_string();
    }

    pub fn on_tick(&mut self) {
        self.tick += 1;
    }

    pub fn handle_events(&mut self, timeout: Duration) -> io::Result<()> {
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = crossterm::event::read()? {
                let handler = self.prompt_handler.clone();
                match handler {
                    None => self.handle_event_normal(key),
                    Some(handlerkind) => self.handle_event_prompt(key, &handlerkind),
                };
            }
        }
        Ok(())
    }

    fn handle_event_normal(&mut self, key: KeyEvent) {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => self.task_selection.select(None),
            (KeyCode::Up, _) => self.prev_task(),
            (KeyCode::Down, _) => self.next_task(),
            (KeyCode::Tab, _) => self.increment_tab(),
            (KeyCode::BackTab, _) => self.decrement_tab(),
            (KeyCode::F(5), _) => {
                if let Err(e) = self.print_file_list() {
                    self.set_feedback_text(&e.to_string());
                };
            }
            (KeyCode::Delete, _) => {
                self.task_list = Vec::new();
            }
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                self.prompt_handler = Some(PromptHandler::SaveFile);
            }
            (KeyCode::Char('o'), KeyModifiers::CONTROL) => {
                self.prompt_handler = Some(PromptHandler::LoadFile);
            }
            (KeyCode::Char('j'), KeyModifiers::CONTROL) => self.move_task_next(),
            (KeyCode::Char('k'), KeyModifiers::CONTROL) => self.move_task_prev(),
            (KeyCode::Char(c), KeyModifiers::NONE) => match c {
                'q' => self.should_quit = true,
                'j' => self.next_task(),
                'k' => self.prev_task(),
                'a' => self.prompt_handler = Some(PromptHandler::AddTask),
                'd' => {
                    if let Some(index) = self.task_selection.selected() {
                        self.remove_task(index);
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }

    fn handle_event_prompt(&mut self, key: KeyEvent, handlerkind: &PromptHandler) {
        match key.code {
            KeyCode::Esc => {
                self.prompt_handler = None;
            }
            KeyCode::Enter => {
                let prompt_text = self.get_prompt_text(true);
                self.handle_prompt(handlerkind, &prompt_text);
                self.prompt_handler = None;
            }
            KeyCode::Tab => self.increment_tab(),
            KeyCode::BackTab => self.decrement_tab(),
            _ => {
                self.textarea.input(key);
            }
        }
    }

    fn handle_prompt(&mut self, handlerkind: &PromptHandler, prompt_text: &str) {
        match handlerkind {
            PromptHandler::AddTask => self.add_task(prompt_text),
            PromptHandler::SaveFile => {
                if let Err(e) = self.save_file(prompt_text) {
                    self.set_feedback_text(&e.to_string());
                }
            }
            PromptHandler::LoadFile => {
                if let Err(e) = self.load_file(prompt_text) {
                    self.set_feedback_text(&e.to_string());
                }
            }
        };
    }

    fn get_prompt_text(&mut self, clear: bool) -> String {
        let text = self.textarea.lines()[0].to_string();
        if clear == true {
            self.textarea.move_cursor(CursorMove::Top);
            self.textarea.move_cursor(CursorMove::Head);
            while self.textarea.lines().len() > 1 {
                self.textarea.delete_line_by_end();
            }
            self.textarea.delete_line_by_end();
        };
        text
    }

    fn add_task(&mut self, desc: &str) {
        self.task_list.push(Task::new(desc));
        self.set_feedback_text(&format!("Added task: {desc}"));
    }

    fn remove_task(&mut self, index: usize) {
        if index < self.task_list.len() {
            let desc = self.task_list.get(index.clone()).unwrap().desc.clone();
            self.task_list.remove(index);
            self.set_feedback_text(&format!("Deleted task: {}", desc));
        }
    }

    fn save_file(&mut self, file_name: &str) -> Result<(), io::Error> {
        match bincode::serialize(&self.task_list) {
            Ok(encoded) => {
                let mut file = File::create(&Path::join(&self.datadir, file_name))?;
                file.write_all(&encoded)?;
                self.set_feedback_text(&format!("Saved file: {file_name}"));
                Ok(())
            }
            Err(e) => Err(io::Error::new(ErrorKind::InvalidData, e.to_string())),
        }
    }

    fn load_file(&mut self, file_name: &str) -> Result<(), io::Error> {
        let mut file = File::open(&Path::join(&self.datadir, file_name))?;
        let mut encoded: Vec<u8> = Vec::new();
        file.read_to_end(&mut encoded)?;
        match bincode::deserialize(encoded.as_slice()) {
            Ok(decoded) => {
                self.task_list = decoded;
                self.set_feedback_text(&format!("Loaded file: {file_name}"));
                return Ok(());
            }
            Err(e) => Err(io::Error::new(ErrorKind::InvalidData, e.to_string())),
        }
    }

    fn print_file_list(&mut self) -> Result<(), io::Error> {
        let mut entries = fs::read_dir(&self.datadir)?
            .map(|res| {
                res.map(|e| {
                    diff_paths(e.path(), &self.datadir)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string()
                })
            })
            .collect::<Result<Vec<_>, io::Error>>()?;
        entries.sort();
        self.help_text = format!("Available files:\n{}", entries.join("\n"));
        self.set_feedback_text("Refreshed file list.");
        Ok(())
    }

    fn get_next_task(&self) -> usize {
        match self.task_selection.selected() {
            Some(selected) => {
                if selected >= self.task_list.len() - 1 {
                    0
                } else {
                    selected + 1
                }
            }
            None => 0,
        }
    }

    fn get_prev_task(&self) -> usize {
        match self.task_selection.selected() {
            Some(selected) => {
                if selected == 0 {
                    self.task_list.len() - 1
                } else {
                    selected - 1
                }
            }
            None => self.task_list.len().max(1) - 1,
        }
    }

    fn get_task_index(&self) -> usize {
        match self.task_selection.selected() {
            Some(selected) => selected,
            None => 0,
        }
    }

    fn next_task(&mut self) {
        self.task_selection.select(Some(self.get_next_task()))
    }

    fn prev_task(&mut self) {
        self.task_selection.select(Some(self.get_prev_task()))
    }

    fn move_task_next(&mut self) {
        let index = self.get_task_index();
        if index < self.task_list.len() - 1 {
            self.task_list.swap(index, index + 1);
        } else {
            let element = self.task_list.remove(index);
            self.task_list.insert(0, element);
        }
        self.next_task();
    }

    fn move_task_prev(&mut self) {
        let index = self.get_task_index();
        if index > 0 {
            self.task_list.swap(index, index - 1);
        } else {
            let last = self.task_list.len() - 1;
            let element = self.task_list.remove(index);
            self.task_list.insert(last, element);
        }
        self.prev_task();
    }

    fn decrement_tab(&mut self) {
        if self.tab_index > 0 {
            self.tab_index -= 1;
        } else {
            self.tab_index = LAST_TAB_INDEX;
        }
    }

    fn increment_tab(&mut self) {
        if self.tab_index < LAST_TAB_INDEX {
            self.tab_index += LAST_TAB_INDEX;
        } else {
            self.tab_index = 0;
        }
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let datadir = AppDirs::new(Some("devboard"), false).unwrap().data_dir;
    fs::create_dir_all(&datadir).unwrap();
    let tick_rate = Duration::from_millis(TICK_RATE_MS);
    let mut app = App::new("Dev Board", datadir);
    app.print_file_list().unwrap();
    app.set_feedback_text("Welcome to DevBoard.");
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        app.handle_events(timeout)?;
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
        if app.should_quit {
            return Ok(());
        }
    }
}

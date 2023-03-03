/// App state and logic
use crate::ui;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{ErrorKind, Write};
use std::{
    fs::File,
    io::{self, Read},
    time::{Duration, Instant},
};
use tui::{backend::Backend, Terminal};
use tui_textarea::{CursorMove, TextArea};

const TICK_RATE_MS: u64 = 25;

#[derive(Clone)]
pub enum PromptHandler {
    AddTask,
    RemoveTask,
    SaveFile,
    LoadFile,
}

pub enum LogMessage {
    Status(String),
    Command(String),
    Response(String),
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

const LAST_TAB_INDEX: usize = 1;

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub tab_index: usize,
    pub tick: i32,
    pub textarea: TextArea<'a>,
    pub prompt_handler: Option<PromptHandler>,
    pub console_log: Vec<LogMessage>,
    pub overview_text: String,
    pub task_list: Vec<Task>,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str) -> App<'a> {
        App {
            title,
            should_quit: false,
            tab_index: 0,
            tick: 0,
            textarea: TextArea::default(),
            prompt_handler: None,
            console_log: vec![],
            overview_text: String::from("No document read."),
            task_list: Vec::new(),
        }
    }

    pub fn status_feedback(&self) -> String {
        for log_message in self.console_log.iter().rev() {
            if let LogMessage::Status(text) = log_message {
                return text.clone();
            }
        }
        String::from("Welcome to DevBoard.")
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
            (KeyCode::Tab, _) => self.increment_tab(),
            (KeyCode::BackTab, _) => self.decrement_tab(),
            (KeyCode::F(5), _) => {
                if let Err(e) = self.print_file_list() {
                    self.console_log.push(LogMessage::Response(format!("{e}")));
                };
            }
            (KeyCode::Delete, _) => {
                self.task_list = Vec::new();
            }
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                self.prompt_handler = Some(PromptHandler::SaveFile);
            }
            (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
                self.prompt_handler = Some(PromptHandler::LoadFile);
            }
            (KeyCode::Char(c), m) => match c {
                'q' => self.should_quit = true,
                'a' => self.prompt_handler = Some(PromptHandler::AddTask),
                'd' => self.prompt_handler = Some(PromptHandler::RemoveTask),
                _ => self
                    .console_log
                    .push(LogMessage::Status(format!("Unknown hotkey: {c} + {:?}", m))),
            },
            _ => self.console_log.push(LogMessage::Status(format!(
                "Unknown hotkey: {:?} + {:?}",
                key.code, key.modifiers
            ))),
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
            PromptHandler::AddTask => self.command_add_task(prompt_text),
            PromptHandler::RemoveTask => self.command_remove_task(prompt_text),
            PromptHandler::SaveFile => {
                if let Err(e) = self.command_save(prompt_text) {
                    self.console_log.push(LogMessage::Response(e.to_string()));
                }
            }
            PromptHandler::LoadFile => {
                if let Err(e) = self.command_load(prompt_text) {
                    self.console_log.push(LogMessage::Response(e.to_string()));
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

    fn command_add_task(&mut self, prompt_text: &str) {
        self.task_list.push(Task::new(prompt_text));
    }

    fn command_remove_task(&mut self, prompt_text: &str) {
        if let Ok(index) = prompt_text.parse() {
            if index < self.task_list.len() {
                self.task_list.remove(index);
            }
        }
    }

    fn command_save(&mut self, prompt_text: &str) -> Result<(), io::Error> {
        match bincode::serialize(&self.task_list) {
            Ok(encoded) => {
                let mut file = File::create(format!("data/{}", prompt_text))?;
                file.write_all(&encoded)?;
                Ok(())
            }
            Err(e) => Err(io::Error::new(ErrorKind::InvalidData, e.to_string())),
        }
    }

    fn command_load(&mut self, prompt_text: &str) -> Result<(), io::Error> {
        let mut file = File::open(format!("data/{}", prompt_text))?;
        let mut encoded: Vec<u8> = Vec::new();
        file.read_to_end(&mut encoded)?;
        match bincode::deserialize(encoded.as_slice()) {
            Ok(decoded) => {
                self.task_list = decoded;
                return Ok(());
            }
            Err(e) => Err(io::Error::new(ErrorKind::InvalidData, e.to_string())),
        }
    }

    fn print_file_list(&mut self) -> Result<(), io::Error> {
        let mut entries = fs::read_dir("data/")?
            .map(|res| res.map(|e| e.path().to_str().unwrap().to_string()))
            .collect::<Result<Vec<_>, io::Error>>()?;
        entries.sort();
        self.overview_text = format!("Available files:\n{}", entries.join("\n"));
        Ok(())
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
    let tick_rate = Duration::from_millis(TICK_RATE_MS);
    let mut app = App::new("Dev Board");
    app.print_file_list().unwrap();
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

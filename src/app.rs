/// App state and logic
use crate::ui;
use crossterm::event::{Event, KeyCode};
use std::fs;
use std::io::ErrorKind;
use std::{
    fs::File,
    io::{self, Read},
    time::{Duration, Instant},
};
use tui::{backend::Backend, Terminal};
use tui_textarea::{CursorMove, TextArea};

const TICK_RATE_MS: u64 = 25;

pub enum LogMessage {
    Status(String),
    Command(String),
    Response(String),
}

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
    pub focus_text: bool,
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
            focus_text: false,
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
                // Handle normal events
                if self.focus_text == false {
                    match key.code {
                        KeyCode::Tab => self.increment_tab(),
                        KeyCode::BackTab => self.decrement_tab(),
                        KeyCode::Enter => {
                            self.focus_text = true;
                        }
                        KeyCode::Char(c) => match c {
                            'q' => self.should_quit = true,
                            _ => self
                                .console_log
                                .push(LogMessage::Status(format!("Input: {c}"))),
                        },
                        _ => (),
                    }
                // Handle focused text input events
                } else {
                    match key.code {
                        KeyCode::Esc => {
                            self.focus_text = false;
                        }
                        KeyCode::Enter => {
                            self.focus_text = false;
                            self.run_command(&self.textarea.lines()[0].clone());
                        }
                        KeyCode::Tab => self.increment_tab(),
                        KeyCode::BackTab => self.decrement_tab(),
                        _ => {
                            self.textarea.input(key);
                        }
                    };
                }
            }
        }
        Ok(())
    }

    fn run_command(&mut self, command: &str) {
        self.console_log
            .push(LogMessage::Command(command.to_string()));
        self.textarea.move_cursor(CursorMove::Top);
        self.textarea.move_cursor(CursorMove::Head);
        while self.textarea.lines().len() > 1 {
            self.textarea.delete_line_by_end();
        }
        self.textarea.delete_line_by_end();
        self.overview_text = match read_file(command) {
            Err(e) => format!("{e}"),
            Ok(e) => e,
        };
        let response = LogMessage::Response("See overview.".to_string());
        self.console_log.push(response);
        self.task_list.push(Task::new(command));
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

fn read_file(file_name: &str) -> Result<String, io::Error> {
    let dir = fs::read_dir("data/")?;
    for dir_result in dir {
        let dir_result = dir_result?;
        let path = dir_result.path();
        if file_name == dir_result.file_name() {
            let mut encoded: Vec<u8> = Vec::new();
            let mut file = File::open(path)?;
            file.read_to_end(&mut encoded)?;
            match String::from_utf8(encoded) {
                Err(e) => return Err(io::Error::new(ErrorKind::InvalidData, e)),
                Ok(s) => return Ok(s),
            };
        }
    }
    let mut entries = fs::read_dir("data/")?
        .map(|res| res.map(|e| e.path().to_str().unwrap().to_string()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    entries.sort();
    Err(io::Error::new(
        ErrorKind::NotFound,
        format!(
            "No files found.\n\nAvailable files:\n  {}",
            entries.join("\n  ")
        ),
    ))
}

/// App state and logic
use crate::ui;
use crossterm::event::{Event, KeyCode};
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
        let mut args: Vec<&str> = command.split(" ").collect();
        let command = args.remove(0);
        match command {
            "add" => self.command_add_task(args),
            "rm" => self.command_remove_task(args),
            "save" => {
                if let Err(e) = self.command_save(args) {
                    self.console_log.push(LogMessage::Response(e.to_string()));
                }
            }
            "load" => {
                if let Err(e) = self.command_load(args) {
                    self.console_log.push(LogMessage::Response(e.to_string()));
                }
            }
            "clear" => {
                self.task_list = Vec::new();
            }
            _ => (),
        }
    }

    fn command_add_task(&mut self, args: Vec<&str>) {
        if args.len() == 0 {
            return;
        }
        self.task_list.push(Task::new(&args.join(" ")));
    }

    fn command_remove_task(&mut self, args: Vec<&str>) {
        if args.len() == 0 {
            return;
        }
        if let Ok(index) = args[0].parse() {
            if index < self.task_list.len() {
                self.task_list.remove(index);
            }
        }
    }

    fn command_save(&mut self, args: Vec<&str>) -> Result<(), io::Error> {
        if args.len() == 0 {
            return Err(io::Error::new(ErrorKind::NotFound, "Missing file name"));
        }
        match bincode::serialize(&self.task_list) {
            Ok(encoded) => {
                let mut file = File::create(format!("data/{}", args[0]))?;
                file.write_all(&encoded)?;
                Ok(())
            }
            Err(e) => Err(io::Error::new(ErrorKind::InvalidData, e.to_string())),
        }
    }

    fn command_load(&mut self, args: Vec<&str>) -> Result<(), io::Error> {
        if args.len() == 0 {
            self.overview_text = format!("Available files:\n{}", get_file_list()?.join("\n"));
            return Ok(());
        }
        let mut file = File::open(format!("data/{}", args[0]))?;
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

fn get_file_list() -> Result<Vec<String>, io::Error> {
    let mut entries = fs::read_dir("data/")?
        .map(|res| res.map(|e| e.path().to_str().unwrap().to_string()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    entries.sort();
    Ok(entries)
}

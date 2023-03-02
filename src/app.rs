/// App state and logic
use crossterm::event::{Event, KeyCode, KeyEvent};
use std::{collections::HashMap, io, process::Command, time::Duration};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tui_textarea::{CursorMove, TextArea};

#[derive(Eq, PartialEq, Hash, EnumIter)]
pub enum GitOutput {
    Status,
    Files,
    Log,
}

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub tab_index: usize,
    pub tick: i32,
    pub textarea: TextArea<'a>,
    pub focus_text: bool,
    pub git_text: HashMap<GitOutput, String>,
    pub console_text: String,
    pub feedback_text: String,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str) -> App<'a> {
        let mut git_text = HashMap::new();
        for go in GitOutput::iter() {
            git_text.insert(go, "".to_string());
        }
        App {
            title,
            should_quit: false,
            tab_index: 0,
            tick: 0,
            textarea: TextArea::default(),
            focus_text: false,
            git_text,
            console_text: String::from(""),
            feedback_text: String::from("Welcome to mingit."),
        }
    }

    pub fn on_tick(&mut self) {
        self.tick += 1;
    }

    pub fn handle_events(&mut self, timeout: Duration) -> io::Result<()> {
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = crossterm::event::read()? {
                if self.focus_text == false {
                    self.handle_other_events(key.code);
                } else {
                    self.handle_input_event(key);
                }
            }
        }
        Ok(())
    }

    fn handle_input_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => {
                self.focus_text = false;
            }
            KeyCode::Enter => {
                self.focus_text = false;
                self.console_text = self.run_git_command(&self.textarea.lines()[0]);
                self.textarea.move_cursor(CursorMove::Top);
                self.textarea.move_cursor(CursorMove::Head);
                while self.textarea.lines().len() > 1 {
                    self.textarea.delete_line_by_end();
                }
                self.textarea.delete_line_by_end();
            }
            _ => {
                self.textarea.input(key_event);
            }
        };
    }

    fn handle_other_events(&mut self, codepoint: KeyCode) {
        match codepoint {
            KeyCode::Tab => self.increment_tab(),
            KeyCode::BackTab => self.decrement_tab(),
            KeyCode::Enter => {
                self.focus_text = true;
            }
            KeyCode::Char(c) => match c {
                'q' => self.should_quit = true,
                _ => self.feedback_text = format!(">> Input keycode: {:?}", codepoint),
            },
            KeyCode::F(5) => {
                self.refresh_git();
            }
            keycode => self.feedback_text = format!(">> Input keycode: {:?}", keycode),
        }
    }

    fn refresh_git(&mut self) {
        self.git_text
            .insert(GitOutput::Status, self.run_git_command("status"));
        self.git_text
            .insert(GitOutput::Files, self.run_git_command("ls-files"));
    }

    fn run_git_command(&self, command: &str) -> String {
        let args: Vec<&str> = command.split(" ").collect();
        let output = Command::new("git").args(args).output().unwrap();
        let mut text_response = String::from_utf8(output.stdout).unwrap();
        if text_response.trim().len() == 0 {
            text_response = String::from_utf8(output.stderr).unwrap();
        }
        text_response
    }

    fn decrement_tab(&mut self) {
        if self.tab_index > 0 {
            self.tab_index -= 1;
        } else {
            self.tab_index = 2;
        }
    }

    fn increment_tab(&mut self) {
        if self.tab_index < 2 {
            self.tab_index += 1;
        } else {
            self.tab_index = 0;
        }
    }
}

/// App state and logic
use crossterm::event::{Event, KeyCode};
use std::{io, process::Command, time::Duration};
use tui_textarea::TextArea;

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub tab_index: usize,
    pub tick: i32,
    pub textarea: TextArea<'a>,
    pub focus_text: bool,
    pub full_text: String,
    pub feedback_text: String,
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
            full_text: String::from(""),
            feedback_text: String::from("Welcome to mingit."),
        }
    }

    pub fn on_tick(&mut self) {
        self.tick += 1;
    }

    pub fn handle_events(&mut self, timeout: Duration) -> io::Result<()> {
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = crossterm::event::read()? {
                if key.code == KeyCode::Esc {
                    self.focus_text = false;
                    self.feedback_text = format!(">> Defocused text.");
                } else if self.focus_text == true {
                    self.textarea.input(key);
                } else {
                    self.handle_other_events(key.code);
                }
            }
        }
        Ok(())
    }

    fn handle_other_events(&mut self, codepoint: KeyCode) {
        match codepoint {
            KeyCode::Tab => self.increment_tab(),
            KeyCode::BackTab => self.decrement_tab(),
            KeyCode::Enter => {
                self.feedback_text = format!(">> Focused text.");
                self.focus_text = true;
            }
            KeyCode::Char(c) => match c {
                'q' => self.should_quit = true,
                _ => self.feedback_text = format!(">> {c}"),
            },
            KeyCode::F(5) => {
                self.run_git_command();
            }
            keycode => {
                self.feedback_text = format!(">> Unkown keycode: {:?}", keycode);
            }
        }
    }

    fn run_git_command(&mut self) {
        let output = Command::new("git")
            .args(self.textarea.lines())
            .output()
            .expect("Failed to execute command");
        // TODO remove unwrap
        self.full_text = String::from(String::from_utf8(output.stdout).unwrap());
        self.feedback_text = format!(">> Ran command: git {:?}", self.textarea.lines());
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

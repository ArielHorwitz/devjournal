/// App state and logic
use crossterm::event::{Event, KeyCode, KeyEvent};
use std::{io, time::Duration};
use tui_textarea::{CursorMove, TextArea};

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub tab_index: usize,
    pub tick: i32,
    pub textarea: TextArea<'a>,
    pub focus_text: bool,
    pub console_log: Vec<String>,
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
            console_log: vec![String::from("Welcome to DevBoard.")],
            feedback_text: String::from("Welcome to DevBoard."),
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
                self.console_log.push(self.textarea.lines()[0].clone());
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
            keycode => self.feedback_text = format!(">> Input keycode: {:?}", keycode),
        }
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

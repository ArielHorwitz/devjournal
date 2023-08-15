use super::center_rect;
use crate::ui::styles;
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    layout::Rect,
    style::Style,
    text::Span,
    widgets::{Block, Borders, Clear},
    Frame,
};
use tui_textarea::{CursorMove, TextArea};

pub enum PromptEvent {
    AwaitingResult(String),
    Result(String),
    Cancelled,
}

pub struct PromptWidget<'a> {
    /// Text to prompt user
    prompt_text: String,
    /// Maximum width of the widget
    max_width: u16,
    margin: usize,
    width_hint: f32,
    textarea: TextArea<'a>,
    focus: bool,
    style_title: Style,
    style_border: Style,
    password: bool,
}

impl<'a> Default for PromptWidget<'a> {
    fn default() -> PromptWidget<'a> {
        let mut widget = PromptWidget {
            prompt_text: "Input:".to_owned(),
            max_width: 60,
            margin: 1,
            width_hint: 1.0,
            textarea: TextArea::default(),
            focus: true,
            style_title: styles::title(),
            style_border: styles::border_highlighted(),
            password: false,
        };
        widget.set_focus(true);
        widget
    }
}

impl<'a> PromptWidget<'a> {
    pub fn focus(mut self, focus: bool) -> PromptWidget<'a> {
        self.set_focus(focus);
        self
    }

    pub fn margin(mut self, margin: usize) -> PromptWidget<'a> {
        self.margin = margin;
        self
    }

    pub fn width_hint(mut self, hint: f32) -> PromptWidget<'a> {
        self.width_hint = hint;
        self
    }

    pub fn set_password(&mut self, is_password: bool) {
        self.password = is_password;
        self.set_focus(self.focus);
    }

    pub fn set_prompt_text(&mut self, text: &str) {
        self.prompt_text = text.to_owned();
    }

    pub fn get_text(&mut self) -> String {
        self.textarea
            .lines()
            .get(0)
            .unwrap_or(&String::from(""))
            .to_owned()
    }

    pub fn set_text(&mut self, text: &str) {
        self.textarea.move_cursor(CursorMove::Top);
        self.textarea.move_cursor(CursorMove::Head);
        while self.textarea.lines().len() > 1 {
            self.textarea.delete_line_by_end();
        }
        self.textarea.delete_line_by_end();
        self.textarea.insert_str(text);
    }

    pub fn set_focus(&mut self, focus: bool) {
        self.focus = focus;
        if self.focus {
            self.style_title = styles::title();
            self.style_border = styles::border_highlighted();
            self.textarea.set_cursor_line_style(match self.password {
                false => styles::prompt(),
                true => styles::prompt_password(),
            });
            self.textarea.set_cursor_style(styles::prompt_cursor());
        } else {
            self.style_title = styles::title_dim();
            self.style_border = styles::border();
            self.textarea.set_cursor_line_style(match self.password {
                false => styles::prompt_dim(),
                true => styles::prompt_password(),
            });
            self.textarea.set_cursor_style(styles::prompt_cursor_dim());
        }
    }

    pub fn clear(&mut self) {
        self.prompt_text = "".to_owned();
        self.set_text("");
        self.password = false;
    }

    pub fn draw<B: Backend>(&self, f: &mut Frame<B>, chunk: Rect) {
        let width = self
            .max_width
            .min((chunk.width as f32 * self.width_hint) as u16);
        let area = center_rect(width, 3, chunk, self.margin as u16);
        f.render_widget(Clear, area);
        let block = Block::default()
            .title(Span::styled(&self.prompt_text, self.style_title))
            .borders(Borders::ALL)
            .border_style(self.style_border);
        let inner = block.inner(area);
        f.render_widget(block, area);
        f.render_widget(self.textarea.widget(), inner)
    }

    pub fn handle_event(&mut self, key: KeyEvent) -> PromptEvent {
        match key.code {
            KeyCode::Esc => PromptEvent::Cancelled,
            KeyCode::Enter => PromptEvent::Result(self.get_text()),
            _ => {
                self.textarea.input(key);
                PromptEvent::AwaitingResult(self.get_text())
            }
        }
    }
}

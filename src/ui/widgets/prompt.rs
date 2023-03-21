use super::center_rect;
use crate::ui::styles;
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    layout::Rect,
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
    textarea: TextArea<'a>,
}

impl<'a> PromptWidget<'a> {
    pub fn default() -> PromptWidget<'a> {
        let mut widget = PromptWidget {
            prompt_text: "Input:".to_string(),
            max_width: 60,
            textarea: TextArea::default(),
        };
        widget.textarea.set_cursor_line_style(styles::prompt());
        widget.textarea.set_cursor_style(styles::prompt());
        widget
    }

    pub fn max_width(mut self, width: usize) -> PromptWidget<'a> {
        self.max_width = width as u16;
        self
    }

    pub fn set_prompt_text(&mut self, text: &str) {
        self.prompt_text = text.to_string();
    }

    pub fn get_text(&mut self) -> String {
        self.textarea.lines()[0].to_string()
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

    pub fn draw<B: Backend>(&self, f: &mut Frame<B>, chunk: Rect) {
        let width = self.max_width.min((chunk.width as f32 * 0.7) as u16);
        let area = center_rect(width, 3, chunk);
        f.render_widget(Clear, area);
        let block = Block::default()
            .title(Span::styled(&self.prompt_text, styles::prompt()))
            .borders(Borders::ALL)
            .border_style(styles::border_highlighted());
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

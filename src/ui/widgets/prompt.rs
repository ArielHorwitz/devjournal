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

use crate::ui::styles;

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
    /// Style for widget
    style: Style,
    /// Style for user input text
    style_input: Style,
    textarea: TextArea<'a>,
}

impl<'a> PromptWidget<'a> {
    pub fn default() -> PromptWidget<'a> {
        PromptWidget {
            prompt_text: "Input:".to_string(),
            max_width: 60,
            style: styles::title_highlighted(),
            style_input: styles::prompt(),
            textarea: TextArea::default(),
        }
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

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, chunk: Rect) {
        let width = self.max_width.min((chunk.width as f32 * 0.7) as u16);
        let area = center_rect(width, 3, chunk);
        f.render_widget(Clear, area);
        let block = Block::default()
            .title(Span::styled(&self.prompt_text, styles::prompt()))
            .borders(Borders::ALL)
            .border_style(styles::border_highlighted());
        let inner = block.inner(area);
        self.textarea.set_cursor_line_style(self.style_input);
        self.textarea.set_cursor_style(self.style_input);
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

fn center_rect(width: u16, height: u16, chunk: Rect) -> Rect {
    Rect::new(
        chunk
            .x
            .saturating_add(1)
            .max(chunk.x + chunk.width.saturating_sub(width) / 2),
        chunk
            .y
            .saturating_add(1)
            .max(chunk.y + chunk.height.saturating_sub(height) / 2),
        width.min(chunk.width.saturating_sub(2)),
        height.min(chunk.height.saturating_sub(2)),
    )
}

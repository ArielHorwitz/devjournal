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
}

impl<'a> PromptWidget<'a> {
    pub fn default() -> PromptWidget<'a> {
        let mut widget = PromptWidget {
            prompt_text: "Input:".to_string(),
            max_width: 60,
            margin: 1,
            width_hint: 1.0,
            textarea: TextArea::default(),
            focus: true,
            style_title: styles::title_highlighted(),
            style_border: styles::border_highlighted(),
        };
        widget.set_focus(true);
        widget
    }

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

    pub fn set_focus(&mut self, focus: bool) {
        self.focus = focus;
        if self.focus {
            self.style_title = styles::title_highlighted();
            self.style_border = styles::border_highlighted();
            self.textarea
                .set_cursor_line_style(styles::prompt_highlighted());
            self.textarea.set_cursor_style(styles::prompt_cursor());
        } else {
            self.style_title = styles::title();
            self.style_border = styles::border();
            self.textarea.set_cursor_line_style(styles::prompt());
            self.textarea.set_cursor_style(styles::prompt_cursor_dim());
        }
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
                return PromptEvent::AwaitingResult(self.get_text());
            }
        }
    }
}

use tui::style::{Color, Modifier, Style};

// Layout
pub fn title() -> Style {
    Style::default()
        .fg(Color::Rgb(48, 255, 48))
        .add_modifier(Modifier::BOLD)
}

pub fn title_dim() -> Style {
    Style::default()
        .fg(Color::Rgb(64, 152, 64))
        .add_modifier(Modifier::BOLD)
}

pub fn border() -> Style {
    Style::default().fg(Color::DarkGray)
}

pub fn border_highlighted() -> Style {
    Style::default().fg(Color::Rgb(110, 0, 110))
}

// Text
pub fn text() -> Style {
    Style::default().fg(Color::White)
}

pub fn text_dim() -> Style {
    Style::default().fg(Color::DarkGray)
}

pub fn text_good() -> Style {
    Style::default()
        .fg(Color::Rgb(0, 255, 32))
        .bg(Color::Rgb(0, 16, 16))
}

pub fn text_warning() -> Style {
    Style::default()
        .fg(Color::Rgb(255, 32, 0))
        .bg(Color::Rgb(16, 16, 0))
}

pub fn list_text() -> Style {
    Style::default().fg(Color::Rgb(128, 192, 255))
}

pub fn list_text_dim() -> Style {
    Style::default().fg(Color::Rgb(64, 96, 128))
}

pub fn list_text_highlight() -> Style {
    Style::default()
        .bg(Color::Rgb(48, 12, 48))
        .fg(Color::Rgb(128, 192, 255))
        .add_modifier(Modifier::BOLD)
}

// Prompt
pub fn prompt() -> Style {
    Style::default().fg(Color::Rgb(255, 128, 0))
}

pub fn prompt_dim() -> Style {
    Style::default().fg(Color::Rgb(128, 64, 0))
}

pub fn prompt_password() -> Style {
    Style::default()
        .bg(Color::Rgb(32, 32, 140))
        .fg(Color::Rgb(32, 32, 140))
}

pub fn prompt_cursor() -> Style {
    Style::default().bg(Color::Rgb(128, 64, 0))
}

pub fn prompt_cursor_dim() -> Style {
    Style::default().bg(Color::DarkGray)
}

// Tabs
pub fn tab() -> Style {
    Style::default()
        .fg(Color::Magenta)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
}

pub fn tab_dim() -> Style {
    Style::default().fg(Color::DarkGray)
}

// Statuses
pub fn warning() -> Style {
    Style::default()
        .bg(Color::Rgb(16, 32, 0))
        .fg(Color::Rgb(255, 192, 32))
}

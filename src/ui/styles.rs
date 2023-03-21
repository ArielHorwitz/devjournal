use tui::style::{Color, Modifier, Style};

pub fn list_normal() -> Style {
    Style::default()
}

pub fn list_highlight() -> Style {
    Style::default()
        .bg(Color::Rgb(48, 12, 48))
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}

pub fn dim() -> Style {
    Style::default().fg(Color::DarkGray)
}

pub fn active_tab() -> Style {
    Style::default()
        .fg(Color::Magenta)
        .add_modifier(Modifier::BOLD)
}

pub fn tab() -> Style {
    Style::default().fg(Color::DarkGray)
}

pub fn prompt() -> Style {
    Style::default().fg(Color::Green)
}

pub fn prompt_highlighted() -> Style {
    Style::default().fg(Color::Rgb(255, 128, 0))
}

pub fn prompt_cursor() -> Style {
    Style::default().bg(Color::Rgb(128, 64, 0))
}

pub fn prompt_cursor_dim() -> Style {
    Style::default().bg(Color::DarkGray)
}

pub fn border() -> Style {
    Style::default().fg(Color::DarkGray)
}

pub fn border_highlighted() -> Style {
    Style::default().fg(Color::Rgb(110, 0, 110))
}

pub fn text() -> Style {
    Style::default().fg(Color::White)
}

pub fn title() -> Style {
    Style::default()
        .fg(Color::Rgb(64, 152, 64))
        .add_modifier(Modifier::BOLD)
}

pub fn title_highlighted() -> Style {
    Style::default()
        .fg(Color::Rgb(48, 255, 48))
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
}

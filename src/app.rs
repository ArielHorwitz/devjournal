// App state and logic
pub mod data;
pub mod list;
use crate::ui::draw;
use crate::ui::events;
use crossterm::{
    event::{Event, KeyCode, KeyModifiers},
    terminal::SetTitle,
};
use data::App;
use platform_dirs::AppDirs;
use std::{
    fs,
    io::{self, stdout},
    time::{Duration, Instant},
};
use tui::{backend::Backend, Terminal};

const TICK_RATE_MS: u64 = 25;

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let datadir = AppDirs::new(Some("devjournal"), false)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "failed to create user folder"))?
        .data_dir;
    fs::create_dir_all(&datadir)?;
    let tick_rate = Duration::from_millis(TICK_RATE_MS);
    let mut app_state = App::new(datadir);
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|frame| draw(frame, &app_state, false))?;
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = crossterm::event::read()? {
                if (KeyCode::Char('q'), KeyModifiers::CONTROL) == (key.code, key.modifiers) {
                    return Ok(());
                }
                events::handle_event(key, &mut app_state);
            }
        };
        if last_tick.elapsed() >= tick_rate {
            let title = format!("Dev Journal - {}", app_state.journal.name);
            crossterm::queue!(stdout(), SetTitle(title))?;
            last_tick = Instant::now();
        }
    }
}

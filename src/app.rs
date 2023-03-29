// App state and logic
pub mod appstate;
pub mod list;
pub mod project;
use crate::ui::draw;
use crate::ui::events;
use appstate::AppState;
use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::SetTitle,
};
use platform_dirs::AppDirs;
use std::{
    fs,
    io::{self, stdout},
    path::PathBuf,
    time::{Duration, Instant},
};
use tui::{backend::Backend, Frame, Terminal};

const TICK_RATE_MS: u64 = 25;

enum Handled {
    Yes,
    No,
}

pub struct App<'a> {
    quit_flag: bool,
    app_state: AppState<'a>,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, datadir: PathBuf) -> App<'a> {
        App {
            quit_flag: false,
            app_state: AppState::new(title, datadir),
        }
    }

    pub fn quit_flag(&self) -> bool {
        self.quit_flag
    }

    fn on_tick(&mut self) {
        let title = format!("{} - {}", self.app_state.title, self.app_state.project.name);
        crossterm::queue!(stdout(), SetTitle(title)).unwrap();
    }

    pub fn handle_events(&mut self, timeout: Duration) -> io::Result<()> {
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = crossterm::event::read()? {
                if let Handled::No = self.handle_events_global(key) {
                    events::handle_event(key, &mut self.app_state);
                }
            }
        };
        Ok(())
    }

    fn handle_events_global(&mut self, key: KeyEvent) -> Handled {
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => self.quit_flag = true,
            _ => return Handled::No,
        };
        Handled::Yes
    }

    pub fn draw<B: Backend>(&self, frame: &mut Frame<B>) {
        draw(frame, &self.app_state, false);
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let datadir = AppDirs::new(Some("devboard"), false).unwrap().data_dir;
    fs::create_dir_all(&datadir).unwrap();
    let tick_rate = Duration::from_millis(TICK_RATE_MS);
    let mut app = App::new("DevBoard", datadir);
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| app.draw(f))?;
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        app.handle_events(timeout)?;
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
        if app.quit_flag() {
            return Ok(());
        }
    }
}

/// App state and logic
pub mod project;
use crate::ui::{
    self,
    widgets::{files::FileListWidget, project::ProjectWidget},
};
use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::SetTitle,
};
use platform_dirs::AppDirs;
use std::{
    fs,
    io::{self, stdout},
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};
use tui::{backend::Backend, Terminal};

const TICK_RATE_MS: u64 = 25;

enum Handled {
    Yes,
    No,
}

pub struct App<'a> {
    pub title: &'a str,
    datadir: PathBuf,
    quit_flag: bool,
    pub tab_index: usize,
    pub user_feedback_text: String,
    pub help_text: String,
    pub filelist_widget: FileListWidget<'a>,
    pub project_widget: ProjectWidget<'a>,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, datadir: PathBuf) -> App<'a> {
        let welcome = format!("Welcome to {title}.");
        App {
            title,
            datadir: datadir.clone(),
            quit_flag: false,
            tab_index: 0,
            user_feedback_text: welcome.clone(),
            help_text: welcome.clone(),
            filelist_widget: FileListWidget::new(datadir.to_str().unwrap()),
            project_widget: ProjectWidget::new(datadir.to_str().unwrap()),
        }
    }

    pub fn set_feedback_text(&mut self, text: &str) {
        self.user_feedback_text = text.to_string();
    }

    fn on_tick(&mut self) {
        let title = format!("{} - {}", self.title, self.project_widget.project_name());
        crossterm::queue!(stdout(), SetTitle(title)).unwrap();
    }

    pub fn handle_events(&mut self, timeout: Duration) -> io::Result<()> {
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = crossterm::event::read()? {
                if self.tab_index == 1 {
                    self.set_feedback_text(&format!("{:?}", key));
                }
                if let Handled::No = self.handle_events_global(key) {
                    self.project_widget.handle_event(key);
                }
            }
        }
        Ok(())
    }

    fn handle_events_global(&mut self, key: KeyEvent) -> Handled {
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => self.quit_flag = true,
            (KeyCode::Char('o'), KeyModifiers::ALT) => self.open_datadir(),
            (KeyCode::F(1), _) => self.tab_index = 0,
            (KeyCode::F(2), _) => self.tab_index = 1,
            _ => return Handled::No,
        };
        Handled::Yes
    }

    fn open_datadir(&self) {
        Command::new("xdg-open").arg(&self.datadir).spawn().unwrap();
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let datadir = AppDirs::new(Some("devboard"), false).unwrap().data_dir;
    fs::create_dir_all(&datadir).unwrap();
    let tick_rate = Duration::from_millis(TICK_RATE_MS);
    let mut app = App::new("DevBoard", datadir);
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        app.handle_events(timeout)?;
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
        if app.quit_flag {
            return Ok(());
        }
    }
}

/// Main entry point
mod app;
mod crypto;
mod journal;
mod ui;
use app::run_app;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{backend::CrosstermBackend, Terminal};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(default_value_t = String::from(""))]
    journal_name: String,
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let target_name = match args.journal_name.as_str() {
        "" => None,
        s => Some(s.to_owned()),
    };
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // create and run the app
    let res = run_app(&mut terminal, target_name);
    // restore terminal
    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    // Send errors to stderr
    if let Err(err) = res {
        eprintln!("{err}")
    }
    Ok(())
}

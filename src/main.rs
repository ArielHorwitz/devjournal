/// Main entry point
mod app;
mod crypto;
mod ui;
use anyhow::{anyhow, Result};
use app::run_app;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use geckopanda::prelude::*;
use platform_dirs::AppDirs;
use std::fs;
use std::io;
use tui::{backend::CrosstermBackend, Terminal};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(default_value_t = String::from(""))]
    journal_name: String,
    #[arg(short, long)]
    google_drive: bool,
}

pub fn main() -> Result<()> {
    let args = Args::parse();
    println!("{args:?}");
    let target_name = match args.journal_name.as_str() {
        "" => None,
        s => Some(s.to_owned()),
    };
    // setup file storage backend
    let user_dirs = AppDirs::new(Some("devjournal"), false)
        .ok_or_else(|| anyhow!("failed to get user directories"))?;
    let storage: Box<dyn Storage> = if args.google_drive {
        let client_secret = include_str!("../client_secret.json");
        fs::create_dir_all(&user_dirs.state_dir)?;
        let token_pathbuf = user_dirs.state_dir.join("tokencache.json");
        let token_cache = token_pathbuf
            .to_str()
            .ok_or_else(|| anyhow!("failed to get token cache file"))?;
        Box::new(GoogleDriveStorage::new(client_secret, token_cache)?)
    } else {
        let datadir = user_dirs
            .data_dir
            .to_str()
            .ok_or_else(|| anyhow!("failed to get data directory"))?;
        Box::new(LocalDiskStorage::new(datadir)?)
    };
    // setup terminal
    let mut res = None;
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    if crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture).is_ok() {
        let backend = CrosstermBackend::new(stdout);
        if let Ok(mut terminal) = Terminal::new(backend) {
            // create and run the app
            res = Some(run_app(&mut terminal, &*storage, target_name));
            crossterm::execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )
            .ok();
            terminal.show_cursor().ok();
        }
    }
    // restore terminal
    disable_raw_mode()?;
    // Send errors to stderr
    if let Some(Err(err)) = res {
        eprintln!("{err}");
    }
    Ok(())
}

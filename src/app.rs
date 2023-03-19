/// App state and logic
pub mod project;
use self::project::{List, Project, Task};
use crate::ui::{self, widgets::list::handle_event};
use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::SetTitle,
};
use pathdiff::diff_paths;
use platform_dirs::AppDirs;
use std::{
    fmt,
    fs::{self, remove_file, File},
    io::{self, stdout, ErrorKind, Read, Write},
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};
use tui::{backend::Backend, Terminal};
use tui_textarea::{CursorMove, TextArea};

const TICK_RATE_MS: u64 = 25;
const CREATE_CHAR: char = '⁕';
const LOAD_CHAR: char = '★';
const SAVE_CHAR: char = '☑';
const DELETE_CHAR: char = '☒';

enum Handled {
    Yes,
    No,
}

#[derive(Clone, Debug)]
pub enum AppFocus {
    FileList,
    TaskList(usize),
    Prompt(PromptHandler),
}

#[derive(Clone, Debug, Copy)]
pub enum PromptHandler {
    AddTask,
    RenameTask,
    SaveFileAs,
    AddFile,
    RenameSubProject,
}

impl fmt::Display for PromptHandler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub struct App<'a> {
    pub title: &'a str,
    datadir: PathBuf,
    last_focus: AppFocus,
    pub focus: AppFocus,
    quit_flag: bool,
    pub tab_index: usize,
    tick: i32,
    pub textarea: TextArea<'a>,
    pub user_feedback_text: String,
    pub help_text: String,
    pub file_list: List<String>,
    pub project: Project,
    active_file: PathBuf,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, datadir: PathBuf) -> App<'a> {
        let welcome = format!("Welcome to {title}.");
        let active_file = get_default_file_path(&datadir).unwrap();
        let mut app = App {
            title,
            datadir,
            last_focus: AppFocus::FileList,
            focus: AppFocus::TaskList(0),
            quit_flag: false,
            tab_index: 0,
            tick: 0,
            textarea: TextArea::default(),
            user_feedback_text: "".to_string(),
            help_text: welcome.clone(),
            file_list: List::new(),
            project: Project::new("dev", "Tasks"),
            active_file: active_file.clone(),
        };
        app.set_active_file(active_file);
        app.load_file(None).unwrap_or(());
        app.refresh_file_list().unwrap();
        app.set_feedback_text(&welcome);
        app
    }

    pub fn set_feedback_text(&mut self, text: &str) {
        self.user_feedback_text = text.to_string();
    }

    pub fn on_tick(&mut self) {
        self.tick += 1;
    }

    pub fn get_active_filename(&self) -> String {
        diff_paths(&self.active_file, &self.datadir)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }

    fn set_active_file(&mut self, active_file: PathBuf) {
        self.active_file = active_file;
        let title = format!("{} - {}", self.title, self.get_active_filename());
        crossterm::queue!(stdout(), SetTitle(title)).unwrap_or(());
    }

    pub fn handle_events(&mut self, timeout: Duration) -> io::Result<()> {
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = crossterm::event::read()? {
                if self.tab_index == 1 {
                    self.set_feedback_text(&format!("{:?}", key));
                }
                if let Handled::No = self.handle_events_global(key) {
                    match self.focus {
                        AppFocus::FileList => self.handle_events_filelist(key),
                        AppFocus::TaskList(i) => self.handle_event_tasklist(key, i),
                        AppFocus::Prompt(handlerkind) => {
                            self.handle_event_prompt(key, &handlerkind);
                        }
                    };
                }
            }
        }
        Ok(())
    }

    fn focus_last(&mut self) {
        let last_focus = self.last_focus.clone();
        self.last_focus = self.focus.clone();
        self.focus = last_focus;
    }

    fn focus_next(&mut self) {
        self.last_focus = self.focus.clone();
        match self.focus {
            AppFocus::FileList => self.focus = AppFocus::TaskList(0),
            AppFocus::TaskList(index) => {
                if index < self.project.subprojects.len() - 1 {
                    self.focus = AppFocus::TaskList(index + 1);
                } else {
                    self.focus = AppFocus::FileList;
                }
            }
            AppFocus::Prompt(_) => self.focus = AppFocus::FileList,
        }
    }

    fn focus_prev(&mut self) {
        self.last_focus = self.focus.clone();
        match self.focus {
            AppFocus::FileList => {
                self.focus = AppFocus::TaskList(self.project.subprojects.len() - 1)
            }
            AppFocus::TaskList(index) => {
                if index == 0 {
                    self.focus = AppFocus::FileList;
                } else {
                    self.focus = AppFocus::TaskList(index - 1)
                }
            }
            AppFocus::Prompt(_) => self.focus = AppFocus::FileList,
        }
    }

    fn focus_prompt(&mut self, handlerkind: PromptHandler) {
        self.last_focus = self.focus.clone();
        self.focus = AppFocus::Prompt(handlerkind);
    }

    fn handle_events_global(&mut self, key: KeyEvent) -> Handled {
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => self.quit_flag = true,
            (KeyCode::Char('o'), KeyModifiers::ALT) => self.open_datadir(),
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                match self.save_file(None) {
                    Ok(_) => self.set_feedback_text(&format!(
                        "{SAVE_CHAR} Saved file: {}",
                        self.get_active_filename()
                    )),
                    Err(e) => self.set_feedback_text(&e.to_string()),
                };
            }
            (KeyCode::Tab, _) => self.focus_next(),
            (KeyCode::BackTab, _) => self.focus_prev(),
            (KeyCode::F(1), _) => self.tab_index = 0,
            (KeyCode::F(2), _) => self.tab_index = 1,
            (KeyCode::F(5), _) => {
                match self.refresh_file_list() {
                    Ok(_) => self.set_feedback_text("Refreshed file list."),
                    Err(e) => self.set_feedback_text(&e.to_string()),
                };
            }
            _ => return Handled::No,
        };
        Handled::Yes
    }

    fn handle_events_filelist(&mut self, key: KeyEvent) {
        if let Err(()) = handle_event(&mut self.file_list, key) {
            match (key.code, key.modifiers) {
                (KeyCode::Char('n'), KeyModifiers::NONE) => {
                    self.focus_prompt(PromptHandler::AddFile);
                }
                (KeyCode::Char('d'), KeyModifiers::NONE) => {
                    if let Some(selected) = self.file_list.pop_selected() {
                        if let Err(e) = self.delete_file(&selected) {
                            self.set_feedback_text(&e.to_string());
                        } else {
                            self.set_feedback_text(&format!("Deleted file: {}", selected));
                        }
                    }
                }
                (KeyCode::Char('s'), KeyModifiers::ALT) => {
                    if let Some(selected) = self.file_list.selected_value() {
                        match self.save_file(Some(&selected.clone())) {
                            Ok(_) => {
                                if let Err(e) = self.refresh_file_list() {
                                    self.set_feedback_text(&e.to_string());
                                };
                            }
                            Err(e) => self.set_feedback_text(&e.to_string()),
                        };
                    }
                }
                (KeyCode::Enter, KeyModifiers::NONE) => {
                    if let Some(filename) = self.file_list.selected_value() {
                        self.load_file(Some(&filename.clone())).unwrap();
                    };
                }
                _ => (),
            }
        }
    }

    fn handle_event_tasklist(&mut self, key: KeyEvent, index: usize) {
        if let Err(()) = handle_event(&mut self.project.subprojects[index].tasks, key) {
            match (key.code, key.modifiers) {
                (KeyCode::Char('s'), KeyModifiers::ALT) => {
                    self.focus_prompt(PromptHandler::SaveFileAs);
                }
                (KeyCode::Char('a'), KeyModifiers::NONE) => {
                    self.focus_prompt(PromptHandler::AddTask);
                }
                (KeyCode::Char('d'), KeyModifiers::NONE) => {
                    if let Some(desc) = self.project.subprojects[index].tasks.pop_selected() {
                        self.set_feedback_text(&format!("Deleted task: {}", desc));
                    }
                }
                (KeyCode::Char('R'), KeyModifiers::SHIFT) => {
                    self.focus_prompt(PromptHandler::RenameSubProject);
                }
                (KeyCode::Char('r'), KeyModifiers::NONE) => {
                    if let Some(text) = self.project.subprojects[index].tasks.selected_value() {
                        self.set_prompt_text(&text.to_string());
                        self.focus_prompt(PromptHandler::RenameTask);
                    }
                }
                _ => (),
            }
        }
    }

    fn handle_event_prompt(&mut self, key: KeyEvent, handlerkind: &PromptHandler) {
        match key.code {
            KeyCode::Esc => self.focus_last(),
            KeyCode::Enter => {
                if let AppFocus::TaskList(index) = self.last_focus {
                    let prompt_text = self.get_prompt_text();
                    self.set_prompt_text("");
                    self.focus_last();
                    self.handle_prompt(handlerkind, &prompt_text, index);
                }
            }
            _ => {
                self.textarea.input(key);
            }
        }
    }

    fn handle_prompt(&mut self, handlerkind: &PromptHandler, prompt_text: &str, index: usize) {
        match handlerkind {
            PromptHandler::AddTask => {
                self.project.subprojects[index]
                    .tasks
                    .add_item(Task::new(prompt_text));
                self.set_feedback_text(&format!("Added task: {prompt_text}"));
            }
            PromptHandler::RenameTask => {
                self.project.subprojects[index]
                    .tasks
                    .replace_selected(Task::new(prompt_text));
                self.set_feedback_text(&format!("Renamed task: {prompt_text}"));
            }
            PromptHandler::SaveFileAs => {
                match self.save_file(Some(prompt_text)) {
                    Ok(_) => {
                        if let Err(e) = self.refresh_file_list() {
                            self.set_feedback_text(&e.to_string());
                        };
                    }
                    Err(e) => self.set_feedback_text(&e.to_string()),
                };
            }
            PromptHandler::AddFile => {
                if let Err(e) = self.load_file(Some(prompt_text)) {
                    self.set_feedback_text(&e.to_string());
                }
            }
            PromptHandler::RenameSubProject => {
                self.project.subprojects[index.clone()].name = prompt_text.to_string();
            }
        };
    }

    fn get_prompt_text(&mut self) -> String {
        self.textarea.lines()[0].to_string()
    }

    fn set_prompt_text(&mut self, text: &str) {
        self.textarea.move_cursor(CursorMove::Top);
        self.textarea.move_cursor(CursorMove::Head);
        while self.textarea.lines().len() > 1 {
            self.textarea.delete_line_by_end();
        }
        self.textarea.delete_line_by_end();
        self.textarea.insert_str(text);
    }

    fn save_file(&mut self, filename: Option<&str>) -> io::Result<()> {
        if let Some(name) = filename {
            self.set_active_file(self.datadir.join(name));
        }
        set_default_file_path(&self.datadir, self.active_file.to_str().unwrap())?;
        let filepath = self.active_file.clone();
        match bincode::serialize(&self.project) {
            Err(e) => Err(io::Error::new(ErrorKind::InvalidData, e.to_string())),
            Ok(encoded) => {
                let mut file = File::create(&filepath)?;
                file.write_all(&encoded)?;
                self.refresh_file_list().unwrap_or(());
                self.set_feedback_text(&format!(
                    "{SAVE_CHAR} Saved file: {}",
                    self.get_active_filename()
                ));
                Ok(())
            }
        }
    }

    fn load_file(&mut self, filename: Option<&str>) -> io::Result<()> {
        if let Some(name) = filename {
            self.set_active_file(self.datadir.join(name));
        }
        set_default_file_path(&self.datadir, self.active_file.to_str().unwrap())?;
        let mut action_name = format!("{LOAD_CHAR} Loaded");
        if !self.active_file.exists() {
            create_file(self.active_file.to_str().unwrap()).unwrap();
            action_name = format!("{CREATE_CHAR} Created");
        }
        self.project = Project::from_file(&self.active_file)?;
        for subproject in &mut self.project.subprojects {
            subproject.tasks.deselect();
        }
        self.set_feedback_text(&format!(
            "{action_name} file: {}",
            self.get_active_filename()
        ));
        self.refresh_file_list().unwrap();
        Ok(())
    }

    fn delete_file(&mut self, filename: &str) -> io::Result<()> {
        let filepath = self.datadir.join(filename);
        remove_file(&filepath)?;
        let relative_filepath = diff_paths(filepath.to_str().unwrap(), &self.datadir)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        self.set_feedback_text(&format!("{DELETE_CHAR} Deleted file: {relative_filepath}"));
        self.refresh_file_list()?;
        Ok(())
    }

    fn refresh_file_list(&mut self) -> io::Result<()> {
        let mut entries = fs::read_dir(&self.datadir)?
            .map(|res| {
                res.map(|e| {
                    diff_paths(e.path(), &self.datadir)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string()
                })
            })
            .filter(|x| match x {
                Err(_) => false,
                Ok(s) => !s.ends_with(".config"),
            })
            .collect::<Result<Vec<_>, io::Error>>()?;
        entries.sort();
        self.file_list.clear_items();
        entries
            .iter()
            .map(|f| self.file_list.add_item(f.clone()))
            .last();
        self.help_text = format!(
            "Project: {}\nFile: {}",
            self.project.name,
            self.active_file.file_name().unwrap().to_str().unwrap(),
        );
        Ok(())
    }

    fn open_datadir(&self) {
        Command::new("xdg-open").arg(&self.datadir).spawn().unwrap();
    }
}

fn create_file(filepath: &str) -> io::Result<()> {
    let empty_data: List<Task> = List::new();
    match bincode::serialize(&empty_data) {
        Err(e) => Err(io::Error::new(ErrorKind::InvalidData, e.to_string())),
        Ok(encoded) => {
            let mut file = File::create(&filepath)?;
            file.write_all(&encoded)?;
            Ok(())
        }
    }
}

fn set_default_file_path(datadir: &Path, default_file_path: &str) -> io::Result<()> {
    fs::write(Path::join(&datadir, ".config"), default_file_path)?;
    Ok(())
}

fn get_default_file_path(datadir: &Path) -> io::Result<PathBuf> {
    let config_path = Path::join(&datadir, ".config");
    if config_path.exists() == false {
        File::create(&config_path)?;
    };
    let mut encoded: Vec<u8> = Vec::new();
    File::open(&config_path)?.read_to_end(&mut encoded)?;
    match String::from_utf8(encoded) {
        Err(e) => Err(io::Error::new(ErrorKind::InvalidData, e)),
        Ok(filepath) => {
            let path = Path::new(&filepath).to_path_buf();
            if filepath == "" || path.ends_with(".config") || path.is_dir() {
                Ok(datadir.join("dev"))
            } else {
                Ok(path)
            }
        }
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

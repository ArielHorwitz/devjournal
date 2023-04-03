use super::widgets::{files::FileListResult, prompt::PromptEvent};
use crate::app::data::{
    load_decrypt, save_encrypt, App, AppPrompt, FileRequest, Journal, JournalPrompt, Project,
    SubProject, Task, DEFAULT_WIDTH_PERCENT,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{path::PathBuf, process::Command};

pub fn handle_event(key: KeyEvent, state: &mut App) {
    match handle_global_event(key, state) {
        Some(feedback) => state.user_feedback_text = feedback,
        None => {
            let is_prompt = state
                .journal
                .project()
                .map_or_else(|| false, |p| p.prompt_request.is_some());
            let result = {
                if state.prompt_request.is_some() {
                    handle_app_prompt_event(key, state)
                } else if state.file_request.is_some() {
                    handle_filelist_event(key, state)
                } else if is_prompt {
                    handle_journal_prompt_event(key, state)
                } else {
                    handle_journal_event(key, state)
                }
            };
            if let Some(feedback) = result {
                state.user_feedback_text = feedback;
            };
        }
    }
}

fn handle_global_event(key: KeyEvent, state: &mut App) -> Option<String> {
    match (key.code, key.modifiers) {
        // Global operations
        (KeyCode::Char('o'), KeyModifiers::ALT) => return open_datadir(state),
        (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
            set_app_prompt(state, AppPrompt::NewJournal, "New file name:", "", false);
        }
        _ => (),
    };
    None
}

fn handle_journal_event(key: KeyEvent, state: &mut App) -> Option<String> {
    match (key.code, key.modifiers) {
        // New
        (KeyCode::Char('n'), KeyModifiers::ALT) => {
            set_project_prompt(
                state,
                JournalPrompt::AddProject,
                "New project name:",
                "",
                false,
            );
        }
        (KeyCode::Char('N'), KeyModifiers::SHIFT) => {
            set_project_prompt(
                state,
                JournalPrompt::AddSubProject,
                "New Subproject Name:",
                "",
                false,
            );
        }
        (KeyCode::Char('n'), KeyModifiers::NONE) => {
            set_project_prompt(state, JournalPrompt::AddTask, "New Task:", "", false);
        }
        // Rename
        (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
            let prefill = state.journal.name.clone();
            set_project_prompt(
                state,
                JournalPrompt::RenameJournal,
                "Journal Name:",
                &prefill,
                false,
            );
        }
        (KeyCode::Char('r'), KeyModifiers::ALT) => {
            let prefill = state.journal.project()?.name.clone();
            set_project_prompt(
                state,
                JournalPrompt::RenameProject,
                "Project Name:",
                &prefill,
                false,
            );
        }
        (KeyCode::Char('R'), KeyModifiers::SHIFT) => {
            let prefill = state.journal.project()?.subproject()?.name.clone();
            set_project_prompt(
                state,
                JournalPrompt::RenameSubProject,
                "Subproject Name:",
                &prefill,
                false,
            );
        }
        (KeyCode::Char('r'), KeyModifiers::NONE) => {
            if let Some(task) = state.journal.project()?.subproject()?.task() {
                let desc = task.desc.clone();
                set_project_prompt(
                    state,
                    JournalPrompt::RenameTask,
                    "Rename Task:",
                    &desc,
                    false,
                );
            }
        }
        // Delete
        (KeyCode::Char('d'), KeyModifiers::ALT) => {
            state.journal.projects.pop_selected();
            bind_focus_size(state);
        }
        (KeyCode::Char('D'), KeyModifiers::SHIFT) => {
            state.journal.project()?.subprojects.pop_selected();
            bind_focus_size(state);
        }
        (KeyCode::Char('d'), KeyModifiers::NONE) => {
            state.journal.project()?.subproject()?.tasks.pop_selected();
        }
        // Navigation
        (KeyCode::Esc, KeyModifiers::NONE) => {
            state.journal.project()?.subproject()?.tasks.deselect();
        }
        (KeyCode::Tab, KeyModifiers::NONE) => state.journal.projects.select_next(),
        (KeyCode::BackTab, _) => state.journal.projects.select_prev(),
        (KeyCode::PageDown, KeyModifiers::CONTROL) => {
            state.journal.projects.select_next();
        }
        (KeyCode::PageUp, KeyModifiers::CONTROL) => {
            state.journal.projects.select_prev();
        }
        (KeyCode::Char('l'), KeyModifiers::NONE) => {
            state.journal.project()?.subprojects.select_next()
        }
        (KeyCode::Char('h'), KeyModifiers::NONE) => {
            state.journal.project()?.subprojects.select_prev()
        }
        (KeyCode::Char('j'), KeyModifiers::NONE) => {
            state.journal.project()?.subproject()?.tasks.select_next();
        }
        (KeyCode::Char('k'), KeyModifiers::NONE) => {
            state.journal.project()?.subproject()?.tasks.select_prev();
        }
        // Shift
        (KeyCode::PageDown, KeyModifiers::ALT) => {
            state.journal.projects.shift_next().ok();
        }
        (KeyCode::PageUp, KeyModifiers::ALT) => {
            state.journal.projects.shift_prev().ok();
        }
        (KeyCode::Char('L'), KeyModifiers::SHIFT) => {
            state.journal.project()?.subprojects.shift_next().ok();
        }
        (KeyCode::Char('H'), KeyModifiers::SHIFT) => {
            state.journal.project()?.subprojects.shift_prev().ok();
        }
        (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
            state
                .journal
                .project()?
                .subproject()?
                .tasks
                .shift_next()
                .ok();
        }
        (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
            state
                .journal
                .project()?
                .subproject()?
                .tasks
                .shift_prev()
                .ok();
        }
        // Move
        (KeyCode::Char('l'), KeyModifiers::CONTROL) => move_task(state, false),
        (KeyCode::Char('h'), KeyModifiers::CONTROL) => move_task(state, true),
        // UI
        (KeyCode::Char('='), KeyModifiers::NONE) => {
            state.journal.project()?.focused_width_percent += 5;
            bind_focus_size(state);
        }
        (KeyCode::Char('-'), KeyModifiers::NONE) => {
            if let Some(project) = state.journal.project() {
                project.focused_width_percent = project.focused_width_percent.saturating_sub(5);
                bind_focus_size(state);
            }
        }
        (KeyCode::Char('\\'), KeyModifiers::NONE) => {
            state.journal.project()?.split_vertical = !state.journal.project()?.split_vertical;
        }
        // File
        (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
            let name = state.journal.name.clone();
            set_project_prompt(
                state,
                JournalPrompt::SetPassword,
                &format!("Set new password for `{name}`:"),
                "",
                true,
            );
        }
        (KeyCode::Char('o'), KeyModifiers::CONTROL) => {
            state.file_request = Some(FileRequest::Load);
            state.filelist.refresh_filelist();
            state.filelist.set_title_text("Open Journal:");
            state.filelist.set_prompt_text("Create New File:");
        }
        (KeyCode::Char('O'), KeyModifiers::SHIFT) => {
            state.file_request = Some(FileRequest::LoadMerge);
            state.filelist.refresh_filelist();
            state.filelist.set_title_text("Merge Journal:");
            state.filelist.set_prompt_text("");
        }
        (KeyCode::Char('s'), KeyModifiers::ALT) => {
            state.file_request = Some(FileRequest::Save);
            state.filelist.refresh_filelist();
            state.filelist.set_title_text("Save Journal:");
            state.filelist.set_prompt_text("Save File As:");
        }
        (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
            return match save_state(state, None) {
                Err(e) => Some(e),
                Ok(_) => Some(format!("Saved journal: {:?}", state.filepath)),
            };
        }
        // Other
        (KeyCode::Char(c), _) => {
            // Navigation (project by number key)
            if let Some(digit) = c.to_digit(10) {
                state.journal.projects.select(digit as usize - 1).ok();
            };
        }
        _ => (),
    };
    None
}

fn move_task(state: &mut App, to_prev: bool) {
    if let Some(project) = state.journal.project() {
        if let Some(subproject) = project.subproject() {
            if let Some(task) = subproject.tasks.pop_selected() {
                let target_subproject = match to_prev {
                    true => project
                        .subprojects
                        .get_item_mut(project.subprojects.prev_index()),
                    false => project
                        .subprojects
                        .get_item_mut(project.subprojects.next_index()),
                };
                let target_subproject = target_subproject
                    .expect("cycling through at least one subproject should yield a subproject");
                target_subproject.tasks.insert_item(
                    target_subproject.tasks.selection(),
                    task,
                    true,
                );
                match to_prev {
                    true => project.subprojects.select_prev(),
                    false => project.subprojects.select_next(),
                }
            }
        }
    }
}

fn handle_app_prompt_event(key: KeyEvent, state: &mut App) -> Option<String> {
    let request = state
        .prompt_request
        .clone()
        .expect("should not be handling prompt events without a request");
    match state.prompt.handle_event(key) {
        PromptEvent::Cancelled => {
            state.prompt_request = None;
            None
        }
        PromptEvent::AwaitingResult(_) => None,
        PromptEvent::Result(result_text) => {
            state.prompt.clear();
            state.prompt_request = None;
            match request {
                AppPrompt::NewJournal => {
                    state.journal = Journal::new(&result_text);
                    state.filepath = state.datadir.join(result_text);
                    save_state(state, None).ok()?;
                    reset_ui(state);
                    Some(format!("New journal created at {:?}", state.filepath))
                }
                AppPrompt::LoadFile(name) => match load_state(state, &name, &result_text, false) {
                    Err(e) => Some(e),
                    Ok(_) => Some(format!("Loaded journal: {:?}", state.filepath)),
                },
                AppPrompt::MergeFile(name) => match load_state(state, &name, &result_text, true) {
                    Err(e) => Some(e),
                    Ok(_) => Some(format!("Loaded journal: {:?}", state.filepath)),
                },
            }
        }
    }
}

fn handle_journal_prompt_event(key: KeyEvent, state: &mut App) -> Option<String> {
    if let Some(request) = state.journal.project()?.prompt_request.clone() {
        match state.journal.project()?.prompt.handle_event(key) {
            PromptEvent::Cancelled => state.journal.project()?.prompt_request = None,
            PromptEvent::AwaitingResult(_) => (),
            PromptEvent::Result(result_text) => {
                state.journal.project()?.prompt.clear();
                state.journal.project()?.prompt_request = None;
                match request {
                    JournalPrompt::AddProject => {
                        state
                            .journal
                            .projects
                            .add_item(Project::new(&result_text), true);
                        bind_focus_size(state);
                    }
                    JournalPrompt::AddSubProject => {
                        if let Some(project) = state.journal.project() {
                            project
                                .subprojects
                                .add_item(SubProject::new(&result_text), true);
                            bind_focus_size(state);
                        }
                    }
                    JournalPrompt::AddTask => {
                        state
                            .journal
                            .project()?
                            .subproject()?
                            .tasks
                            .add_item(Task::new(&result_text), true);
                    }
                    JournalPrompt::RenameJournal => {
                        state.journal.name = result_text;
                        return Some(format!("Renamed journal: {}", state.journal.name));
                    }
                    JournalPrompt::RenameProject => {
                        if let Some(project) = state.journal.project() {
                            project.name = result_text;
                            return Some(format!("Renamed project: {}", project.name));
                        }
                    }
                    JournalPrompt::RenameSubProject => {
                        state.journal.project()?.subproject()?.name = result_text;
                    }
                    JournalPrompt::RenameTask => {
                        state.journal.project()?.subproject()?.task()?.desc = result_text;
                    }
                    JournalPrompt::SetPassword => {
                        state.journal.password = result_text;
                        return Some("Set encryption password".to_owned());
                    }
                };
                state.journal.project()?.prompt_request = None;
            }
        };
    }
    None
}

fn handle_filelist_event(key: KeyEvent, state: &mut App) -> Option<String> {
    match state.filelist.handle_event(key) {
        FileListResult::AwaitingResult => (),
        FileListResult::Cancelled => state.file_request = None,
        FileListResult::Feedback(message) => state.user_feedback_text = message,
        FileListResult::Result(name) => {
            if let Some(fr) = state.file_request {
                match fr {
                    FileRequest::Load => set_app_prompt(
                        state,
                        AppPrompt::LoadFile(name.clone()),
                        &format!("Password for `{name}`:"),
                        "",
                        true,
                    ),
                    FileRequest::LoadMerge => set_app_prompt(
                        state,
                        AppPrompt::MergeFile(name.clone()),
                        &format!("Password for `{name}`:"),
                        "",
                        true,
                    ),
                    FileRequest::Save => {
                        return match save_state(state, Some(&state.datadir.join(name))) {
                            Err(e) => Some(e),
                            Ok(_) => Some(format!("Saved journal {:?}", state.filepath)),
                        };
                    }
                }
                state.file_request = None;
            }
        }
    }
    None
}

fn set_app_prompt(
    state: &mut App,
    request: AppPrompt,
    prompt_text: &str,
    prefill_text: &str,
    password: bool,
) {
    state.prompt.set_prompt_text(prompt_text);
    state.prompt.set_text(prefill_text);
    state.prompt_request = Some(request);
    state.prompt.set_password(password);
}

fn set_project_prompt(
    state: &mut App,
    request: JournalPrompt,
    prompt_text: &str,
    prefill_text: &str,
    password: bool,
) {
    if let Some(project) = state.journal.project() {
        project.prompt.set_prompt_text(prompt_text);
        project.prompt.set_text(prefill_text);
        project.prompt_request = Some(request);
        project.prompt.set_password(password);
    }
}

fn reset_ui(state: &mut App) {
    if let Some(project) = state.journal.project() {
        project.focused_width_percent = DEFAULT_WIDTH_PERCENT;
        bind_focus_size(state);
    }
}

fn bind_focus_size(state: &mut App) {
    if let Some(project) = state.journal.project() {
        let min_width = (100. / project.subprojects.len() as f32).max(5.) as u16;
        project.focused_width_percent = project.focused_width_percent.min(95).max(min_width);
    }
}

fn open_datadir(state: &App) -> Option<String> {
    if let Err(e) = Command::new("xdg-open").arg(&state.datadir).spawn() {
        return Some(format!("failed to open {:?} [{e}]", &state.datadir));
    }
    None
}

fn save_state(state: &mut App, filepath: Option<&PathBuf>) -> Result<(), String> {
    let filepath = filepath.unwrap_or(&state.filepath);
    save_encrypt(&state.journal, filepath, &state.journal.password)?;
    state.filelist.refresh_filelist();
    Ok(())
}

fn load_state(state: &mut App, name: &str, key: &str, merge: bool) -> Result<(), String> {
    let filepath = state.datadir.join(name);
    if !filepath.exists() {
        return Err("file does not exist".to_owned());
    }
    let loaded_journal = {
        if let Ok(journal) = load_decrypt::<Journal>(&filepath, key) {
            journal
        } else {
            load_decrypt::<Project>(&filepath, key)?.into()
        }
    };
    state.journal = match merge {
        true => state.journal.clone() + loaded_journal,
        false => loaded_journal,
    };
    state.journal.password = key.to_owned();
    state.filepath = filepath;
    state.filelist.refresh_filelist();
    reset_ui(state);
    Ok(())
}

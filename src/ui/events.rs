use super::widgets::{files::FileListResult, prompt::PromptEvent};
use crate::app::data::{
    filename, App, AppPrompt, DataDeserialize, DataSerialize, Error, FileRequest, Journal,
    JournalPrompt, Project, Result, SubProject, Task, DEFAULT_WIDTH_PERCENT,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{path::PathBuf, process::Command};

pub fn handle_event(key: KeyEvent, state: &mut App) {
    if !handle_global_event(key, state) {
        let is_prompt = state
            .journal
            .project()
            .map_or_else(|| false, |p| p.prompt_request.is_some());
        if state.prompt_request.is_some() {
            handle_app_prompt_event(key, state);
        } else if state.file_request.is_some() {
            handle_filelist_event(key, state);
        } else if is_prompt {
            handle_journal_prompt_event(key, state);
        } else {
            handle_journal_event(key, state);
        }
    };
}

fn handle_global_event(key: KeyEvent, state: &mut App) -> bool {
    match (key.code, key.modifiers) {
        // Global operations
        (KeyCode::Char('o'), KeyModifiers::ALT) => {
            if let Err(e) = open_datadir(state) {
                state.add_feedback(Error::from_cause("Failed to save file", e));
            };
        }
        (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
            set_app_prompt(state, AppPrompt::NewJournal, "New file name:", "", false);
        }
        _ => return false,
    };
    true
}

fn handle_journal_event(key: KeyEvent, state: &mut App) {
    match (key.code, key.modifiers) {
        // New
        (KeyCode::Char('n'), KeyModifiers::ALT) => {
            if let Some(project) = state.journal.project() {
                set_project_prompt(
                    project,
                    JournalPrompt::AddProject,
                    "New project name:",
                    "",
                    false,
                );
            }
        }
        (KeyCode::Char('N'), KeyModifiers::SHIFT) => {
            if let Some(project) = state.journal.project() {
                set_project_prompt(
                    project,
                    JournalPrompt::AddSubProject,
                    "New Subproject Name:",
                    "",
                    false,
                );
            }
        }
        (KeyCode::Char('n'), KeyModifiers::NONE) => {
            if let Some(project) = state.journal.project() {
                set_project_prompt(project, JournalPrompt::AddTask, "New Task:", "", false);
            }
        }
        // Rename
        (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
            let prefill = state.journal.name.clone();
            if let Some(project) = state.journal.project() {
                set_project_prompt(
                    project,
                    JournalPrompt::RenameJournal,
                    "Journal Name:",
                    &prefill,
                    false,
                );
            }
        }
        (KeyCode::Char('r'), KeyModifiers::ALT) => {
            if let Some(project) = state.journal.project() {
                set_project_prompt(
                    project,
                    JournalPrompt::RenameProject,
                    "Project Name:",
                    &project.name.clone(),
                    false,
                );
            }
        }
        (KeyCode::Char('R'), KeyModifiers::SHIFT) => {
            if let Some(project) = state.journal.project() {
                if project.subprojects.selection().is_some() {
                    let prefill = project
                        .subproject()
                        .expect("selection is Some, should not be empty")
                        .name
                        .clone();
                    set_project_prompt(
                        project,
                        JournalPrompt::RenameSubProject,
                        "Subproject Name:",
                        &prefill,
                        false,
                    );
                };
            }
        }
        (KeyCode::Char('r'), KeyModifiers::NONE) => {
            if let Some(project) = state.journal.project() {
                let mut task_name = None;
                if let Some(subproject) = project.subproject() {
                    if let Some(task) = subproject.task() {
                        task_name = Some(task.desc.clone());
                    }
                }
                if let Some(prefill) = task_name {
                    set_project_prompt(
                        project,
                        JournalPrompt::RenameTask,
                        "Rename Task:",
                        &prefill,
                        false,
                    );
                }
            }
        }
        // Delete
        (KeyCode::Char('d'), KeyModifiers::ALT) => {
            state.journal.projects.pop_selected();
        }
        (KeyCode::Char('D'), KeyModifiers::SHIFT) => {
            if let Some(project) = state.journal.project() {
                project.subprojects.pop_selected();
            };
        }
        (KeyCode::Char('d'), KeyModifiers::NONE) => {
            if let Some(project) = state.journal.project() {
                if let Some(subproject) = project.subproject() {
                    subproject.tasks.pop_selected();
                }
            }
        }
        // Navigation
        (KeyCode::Esc, KeyModifiers::NONE) => {
            if let Some(project) = state.journal.project() {
                if let Some(subproject) = project.subproject() {
                    subproject.tasks.deselect();
                }
            }
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
            if let Some(project) = state.journal.project() {
                project.subprojects.select_next();
            }
        }
        (KeyCode::Char('h'), KeyModifiers::NONE) => {
            if let Some(project) = state.journal.project() {
                project.subprojects.select_prev();
            }
        }
        (KeyCode::Char('j'), KeyModifiers::NONE) => {
            if let Some(project) = state.journal.project() {
                if let Some(subproject) = project.subproject() {
                    subproject.tasks.select_next();
                }
            }
        }
        (KeyCode::Char('k'), KeyModifiers::NONE) => {
            if let Some(project) = state.journal.project() {
                if let Some(subproject) = project.subproject() {
                    subproject.tasks.select_prev();
                }
            }
        }
        // Shift
        (KeyCode::PageDown, KeyModifiers::ALT) => {
            state.journal.projects.shift_next().ok();
        }
        (KeyCode::PageUp, KeyModifiers::ALT) => {
            state.journal.projects.shift_prev().ok();
        }
        (KeyCode::Char('L'), KeyModifiers::SHIFT) => {
            if let Some(project) = state.journal.project() {
                project.subprojects.shift_next().ok();
            }
        }
        (KeyCode::Char('H'), KeyModifiers::SHIFT) => {
            if let Some(project) = state.journal.project() {
                project.subprojects.shift_prev().ok();
            }
        }
        (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
            if let Some(project) = state.journal.project() {
                if let Some(subproject) = project.subproject() {
                    subproject.tasks.shift_next().ok();
                }
            }
        }
        (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
            if let Some(project) = state.journal.project() {
                if let Some(subproject) = project.subproject() {
                    subproject.tasks.shift_prev().ok();
                }
            }
        }
        // Move
        (KeyCode::Char('l'), KeyModifiers::CONTROL) => move_task(state, false),
        (KeyCode::Char('h'), KeyModifiers::CONTROL) => move_task(state, true),
        // UI
        (KeyCode::Char('='), KeyModifiers::NONE) => {
            if let Some(project) = state.journal.project() {
                project.focused_width_percent += 5;
                bind_focus_size(project);
            }
        }
        (KeyCode::Char('-'), KeyModifiers::NONE) => {
            if let Some(project) = state.journal.project() {
                project.focused_width_percent = project.focused_width_percent.saturating_sub(5);
                bind_focus_size(project);
            }
        }
        (KeyCode::Char('\\'), KeyModifiers::NONE) => {
            if let Some(project) = state.journal.project() {
                project.split_vertical = !project.split_vertical;
            }
        }
        // File
        (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
            let name = state.journal.name.clone();
            if let Some(project) = state.journal.project() {
                set_project_prompt(
                    project,
                    JournalPrompt::SetPassword,
                    &format!("Set new password for `{name}`:"),
                    "",
                    true,
                );
            }
        }
        (KeyCode::Char('o'), KeyModifiers::CONTROL) => {
            state.file_request = Some(FileRequest::Load);
            state.filelist.reset();
            state.filelist.set_title_text("Open Journal:");
            state.filelist.set_prompt_text("Create New File:");
        }
        (KeyCode::Char('O'), KeyModifiers::SHIFT) => {
            state.file_request = Some(FileRequest::LoadMerge);
            state.filelist.reset();
            state.filelist.set_title_text("Merge Journal:");
            state.filelist.set_prompt_text("");
        }
        (KeyCode::Char('s'), KeyModifiers::ALT) => {
            state.file_request = Some(FileRequest::Save);
            state.filelist.reset();
            state.filelist.set_title_text("Save Journal:");
            state.filelist.set_prompt_text("Save File As:");
        }
        (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
            return match save_state(state, None) {
                Err(e) => state.add_feedback(Error::from_cause("Failed to save file", e)),
                Ok(_) => {
                    state.add_feedback(format!("Saved journal `{}`", filename(&state.filepath)))
                }
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

fn handle_app_prompt_event(key: KeyEvent, state: &mut App) {
    let request = state
        .prompt_request
        .clone()
        .expect("should not be handling prompt events without a request");
    match state.prompt.handle_event(key) {
        PromptEvent::Cancelled => {
            state.prompt_request = None;
        }
        PromptEvent::AwaitingResult(_) => (),
        PromptEvent::Result(result_text) => {
            state.prompt.clear();
            state.prompt_request = None;
            match request {
                AppPrompt::NewJournal => {
                    state.journal = Journal::new(&result_text);
                    state.filepath = state.datadir.join(result_text);
                    match save_state(state, None) {
                        Err(e) => {
                            state.add_feedback(Error::from_cause("Failed to save file", e));
                        }
                        Ok(_) => {
                            if let Some(project) = state.journal.project() {
                                reset_ui(project);
                            };
                            state.add_feedback(format!(
                                "Created journal `{}`",
                                filename(&state.filepath)
                            ));
                        }
                    }
                }
                AppPrompt::LoadFile(name) => match load_state(state, &name, &result_text, false) {
                    Err(e) => state.add_feedback(Error::from_cause("Failed to load file", e)),
                    Ok(_) => state
                        .add_feedback(format!("Loaded journal `{}`", filename(&state.filepath))),
                },
                AppPrompt::MergeFile(name) => match load_state(state, &name, &result_text, true) {
                    Err(e) => state.add_feedback(Error::from_cause("Failed to merge file", e)),
                    Ok(_) => state
                        .add_feedback(format!("Merged journal `{}`", filename(&state.filepath))),
                },
            };
        }
    }
}

fn handle_journal_prompt_event(key: KeyEvent, state: &mut App) {
    if let Some(project) = state.journal.project() {
        if let Some(request) = project.prompt_request.clone() {
            match project.prompt.handle_event(key) {
                PromptEvent::Cancelled => project.prompt_request = None,
                PromptEvent::AwaitingResult(_) => (),
                PromptEvent::Result(result_text) => {
                    project.prompt.clear();
                    project.prompt_request = None;
                    match request {
                        JournalPrompt::AddProject => {
                            state
                                .journal
                                .projects
                                .add_item(Project::new(&result_text), true);
                        }
                        JournalPrompt::AddSubProject => {
                            project
                                .subprojects
                                .add_item(SubProject::new(&result_text), true);
                            bind_focus_size(project);
                        }
                        JournalPrompt::AddTask => {
                            if let Some(subproject) = project.subproject() {
                                subproject.tasks.add_item(Task::new(&result_text), true);
                            }
                        }
                        JournalPrompt::RenameJournal => {
                            state.journal.name = result_text;
                            return state
                                .add_feedback(format!("Renamed journal: {}", state.journal.name));
                        }
                        JournalPrompt::RenameProject => {
                            project.name = result_text.clone();
                            return state.add_feedback(format!("Renamed project: {result_text}",));
                        }
                        JournalPrompt::RenameSubProject => {
                            if let Some(subproject) = project.subproject() {
                                subproject.name = result_text;
                            }
                        }
                        JournalPrompt::RenameTask => {
                            if let Some(subproject) = project.subproject() {
                                if let Some(task) = subproject.task() {
                                    task.desc = result_text;
                                }
                            }
                        }
                        JournalPrompt::SetPassword => {
                            state.journal.password = result_text;
                            state.add_feedback("Set encryption password");
                        }
                    };
                }
            };
        }
    }
}

fn handle_filelist_event(key: KeyEvent, state: &mut App) {
    match state.filelist.handle_event(key) {
        FileListResult::AwaitingResult => (),
        FileListResult::Cancelled => state.file_request = None,
        FileListResult::Feedback(message) => state.add_feedback(message),
        FileListResult::Result(name) => {
            if let Some(fr) = state.file_request {
                state.file_request = None;
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
                        let filepath = state.datadir.join(name);
                        return match save_state(state, Some(&filepath)) {
                            Err(e) => {
                                state.add_feedback(Error::from_cause("Failed to save file", e))
                            }
                            Ok(_) => state
                                .add_feedback(format!("Saved journal `{}`", filename(&filepath))),
                        };
                    }
                }
            }
        }
    }
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
    project: &mut Project,
    request: JournalPrompt,
    prompt_text: &str,
    prefill_text: &str,
    password: bool,
) {
    project.prompt.set_prompt_text(prompt_text);
    project.prompt.set_text(prefill_text);
    project.prompt_request = Some(request);
    project.prompt.set_password(password);
}

fn reset_ui(project: &mut Project) {
    project.focused_width_percent = DEFAULT_WIDTH_PERCENT;
    bind_focus_size(project);
}

fn bind_focus_size(project: &mut Project) {
    let min_width = (100. / project.subprojects.len() as f32).max(5.) as u16;
    project.focused_width_percent = project.focused_width_percent.min(95).max(min_width);
}

fn open_datadir(state: &App) -> Result<()> {
    Command::new("xdg-open")
        .arg(&state.datadir)
        .spawn()
        .map_err(Error::from)?;
    Ok(())
}

fn save_state(state: &mut App, filepath: Option<&PathBuf>) -> Result<()> {
    let filepath = filepath.unwrap_or(&state.filepath);
    state
        .journal
        .save_encrypt(filepath, &state.journal.password)?;
    state.filepath = filepath.clone();
    state.filelist.reset();
    Ok(())
}

fn load_state(state: &mut App, name: &str, key: &str, merge: bool) -> Result<()> {
    let filepath = state.datadir.join(name);
    if !filepath.exists() {
        Journal::new(name)
            .save_encrypt(&filepath, key)
            .map_err(|e| Error::from(format!("failed to create new file [{e}]")))?;
    }
    let loaded_journal = Journal::load_decrypt(&filepath, key)?;
    state.journal = match merge {
        true => state.journal.clone() + loaded_journal,
        false => loaded_journal,
    };
    state.journal.password = key.to_owned();
    state.filepath = filepath;
    state.filelist.reset();
    Ok(())
}

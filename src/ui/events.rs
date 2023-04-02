use super::widgets::{files::FileListResult, prompt::PromptEvent};
use crate::app::data::{
    load_decrypt, save_encrypt, App, FileRequest, Journal, Project, PromptRequest, SubProject,
    Task, DEFAULT_PROJECT_FILENAME, DEFAULT_WIDTH_PERCENT,
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
                if state.file_request.is_some() {
                    handle_filelist_event(key, state)
                } else if is_prompt {
                    handle_prompt_event(key, state)
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
        _ => None,
    }
}

fn handle_journal_event(key: KeyEvent, state: &mut App) -> Option<String> {
    match (key.code, key.modifiers) {
        // New
        (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
            state.journal = Default::default();
            state.filepath = state.datadir.join(DEFAULT_PROJECT_FILENAME);
            reset_ui(state);
            return Some("New journal created".to_owned());
        }
        (KeyCode::Char('n'), KeyModifiers::ALT) => {
            set_prompt(state, PromptRequest::AddProject, "New project name:");
        }
        (KeyCode::Char('N'), KeyModifiers::SHIFT) => {
            set_prompt(state, PromptRequest::AddSubProject, "New Subproject Name:");
        }
        (KeyCode::Char('n'), KeyModifiers::NONE) => {
            set_prompt(state, PromptRequest::AddTask, "New Task:");
        }
        // Rename
        (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
            let prefill = state.journal.name.clone();
            set_prompt_extra(
                state,
                PromptRequest::RenameJournal,
                "Journal Name:",
                &prefill,
                false,
            );
        }
        (KeyCode::Char('r'), KeyModifiers::ALT) => {
            let prefill = state.journal.project()?.name.clone();
            set_prompt_extra(
                state,
                PromptRequest::RenameProject,
                "Project Name:",
                &prefill,
                false,
            );
        }
        (KeyCode::Char('R'), KeyModifiers::SHIFT) => {
            let prefill = state.journal.project()?.subproject()?.name.clone();
            set_prompt_extra(
                state,
                PromptRequest::RenameSubProject,
                "Subproject Name:",
                &prefill,
                false,
            );
        }
        (KeyCode::Char('r'), KeyModifiers::NONE) => {
            if let Some(task) = state.journal.project()?.subproject()?.task() {
                let desc = task.desc.clone();
                set_prompt_extra(
                    state,
                    PromptRequest::RenameTask,
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
        (KeyCode::PageDown, KeyModifiers::CONTROL) => {
            state.journal.projects.move_down().ok();
        }
        (KeyCode::PageUp, KeyModifiers::CONTROL) => {
            state.journal.projects.move_up().ok();
        }
        (KeyCode::Char('L'), KeyModifiers::SHIFT) => {
            state.journal.project()?.subprojects.move_down().ok();
        }
        (KeyCode::Char('H'), KeyModifiers::SHIFT) => {
            state.journal.project()?.subprojects.move_up().ok();
        }
        (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
            state
                .journal
                .project()?
                .subproject()?
                .tasks
                .move_down()
                .ok();
        }
        (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
            state.journal.project()?.subproject()?.tasks.move_up().ok();
        }
        // Move
        (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
            if let Some(project) = state.journal.project() {
                if let Some(subproject) = project.subproject() {
                    if let Some(task) = subproject.tasks.pop_selected() {
                        let target_subproject = project.subprojects.next_item_mut().expect(
                            "next subproject should exist if at least one subproject exists",
                        );
                        target_subproject.tasks.insert_item(
                            target_subproject.tasks.selected(),
                            task,
                            true,
                        );
                        project.subprojects.select_next();
                    }
                }
            }
        }
        (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
            if let Some(project) = state.journal.project() {
                if let Some(subproject) = project.subproject() {
                    if let Some(task) = subproject.tasks.pop_selected() {
                        let target_subproject = project.subprojects.prev_item_mut().expect(
                            "prev subproject should exist if at least one subproject exists",
                        );
                        target_subproject.tasks.insert_item(
                            target_subproject.tasks.selected(),
                            task,
                            true,
                        );
                        project.subprojects.select_prev()
                    }
                }
            }
        }
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
        (KeyCode::Char('p'), KeyModifiers::ALT) => {
            let name = state.journal.project()?.name.clone();
            set_prompt_extra(
                state,
                PromptRequest::SetPassword,
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
        (KeyCode::Char('s'), KeyModifiers::ALT) => {
            state.file_request = Some(FileRequest::Save);
            state.filelist.refresh_filelist();
            state.filelist.set_title_text("Save Journal:");
            state.filelist.set_prompt_text("Save File As:");
        }
        (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
            return match save_project(state, None) {
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

fn handle_prompt_event(key: KeyEvent, state: &mut App) -> Option<String> {
    if let Some(request) = state.journal.project()?.prompt_request.clone() {
        match state.journal.project()?.prompt.handle_event(key) {
            PromptEvent::Cancelled => state.journal.project()?.prompt_request = None,
            PromptEvent::AwaitingResult(_) => (),
            PromptEvent::Result(result_text) => {
                clear_prompt(state);
                match request {
                    PromptRequest::SetPassword => {
                        state.journal.password = result_text;
                        return Some("Reset journal password".to_owned());
                    }
                    PromptRequest::GetLoadPassword(name) => {
                        return match load_project(state, &name, &result_text) {
                            Err(e) => Some(e),
                            Ok(_) => Some(format!("Loaded journal: {:?}", state.filepath)),
                        };
                    }
                    PromptRequest::RenameJournal => {
                        state.journal.name = result_text;
                        return Some(format!("Renamed journal: {}", state.journal.name));
                    }
                    PromptRequest::AddProject => {
                        state.journal.projects.insert_item(
                            state.journal.projects.next_index(),
                            Project {
                                name: result_text,
                                ..Default::default()
                            },
                            true,
                        );
                        bind_focus_size(state);
                    }
                    PromptRequest::RenameProject => {
                        if let Some(project) = state.journal.project() {
                            project.name = result_text;
                            return Some(format!("Renamed project: {}", project.name));
                        }
                    }
                    PromptRequest::RenameSubProject => {
                        state.journal.project()?.subproject()?.name = result_text;
                    }
                    PromptRequest::AddSubProject => {
                        if let Some(project) = state.journal.project() {
                            project.subprojects.insert_item(
                                project.subprojects.next_index(),
                                SubProject::new(&result_text),
                                true,
                            );
                            bind_focus_size(state);
                        }
                    }
                    PromptRequest::AddTask => {
                        state
                            .journal
                            .project()?
                            .subproject()?
                            .tasks
                            .add_item(Task::new(&result_text));
                    }
                    PromptRequest::RenameTask => {
                        state.journal.project()?.subproject()?.task()?.desc = result_text.clone();
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
            if let Some(fr) = &state.file_request {
                match fr {
                    FileRequest::Load => set_prompt_extra(
                        state,
                        PromptRequest::GetLoadPassword(name.clone()),
                        &format!("Password for `{name}`:"),
                        "",
                        true,
                    ),
                    FileRequest::Save => {
                        return match save_project(state, Some(&state.datadir.join(name))) {
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

fn clear_prompt(state: &mut App) {
    if let Some(project) = state.journal.project() {
        project.prompt.set_prompt_text("Input:");
        project.prompt.set_text("");
        project.prompt_request = None;
        project.prompt.set_password(false);
    }
}

fn set_prompt(state: &mut App, request: PromptRequest, prompt_text: &str) {
    set_prompt_extra(state, request, prompt_text, "", false)
}

fn set_prompt_extra(
    state: &mut App,
    request: PromptRequest,
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
        let min_width = (100. / project.subprojects.items().len() as f32).max(5.) as u16;
        project.focused_width_percent = project.focused_width_percent.min(95).max(min_width);
    }
}

fn open_datadir(state: &App) -> Option<String> {
    if let Err(e) = Command::new("xdg-open").arg(&state.datadir).spawn() {
        return Some(format!("failed to open {:?} [{e}]", &state.datadir,));
    }
    None
}

fn save_project(state: &mut App, filepath: Option<&PathBuf>) -> Result<(), String> {
    let filepath = filepath.unwrap_or(&state.filepath);
    save_encrypt(&state.journal, filepath, &state.journal.password)?;
    state.filelist.refresh_filelist();
    Ok(())
}

fn load_project(state: &mut App, name: &str, key: &str) -> Result<(), String> {
    let filepath = state.datadir.join(name);
    if !filepath.exists() {
        return Err("file does not exist".to_owned());
    }
    state.journal = {
        if let Ok(journal) = load_decrypt::<Journal>(&filepath, key) {
            journal
        } else {
            load_decrypt::<Project>(&filepath, key)?.into()
        }
    };
    state.journal.password = key.to_owned();
    state.filepath = filepath;
    state.filelist.refresh_filelist();
    reset_ui(state);
    Ok(())
}

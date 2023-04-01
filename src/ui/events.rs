use super::widgets::{files::FileListResult, prompt::PromptEvent};
use crate::app::data::{
    load_decrypt, save_encrypt, App, FileRequest, Project, PromptRequest, SubProject, Task,
    DEFAULT_PROJECT_FILENAME, DEFAULT_WIDTH_PERCENT,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{path::PathBuf, process::Command};

pub fn handle_event(key: KeyEvent, state: &mut App) {
    match handle_global_event(key, state) {
        Some(feedback) => state.user_feedback_text = feedback,
        None => {
            let result = {
                if state.file_request.is_some() {
                    handle_filelist_event(key, state)
                } else if state.project.prompt_request.is_some() {
                    handle_prompt_event(key, state)
                } else {
                    handle_subproject_event(key, state)
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
        (KeyCode::F(1), _) => state.tab_index = 0,
        (KeyCode::F(2), _) => state.tab_index = 1,
        _ => (),
    };
    None
}

fn handle_subproject_event(key: KeyEvent, state: &mut App) -> Option<String> {
    match (key.code, key.modifiers) {
        // Project operations
        (KeyCode::Char('n'), KeyModifiers::ALT) => {
            state.project = Project::default();
            state.filepath = state.datadir.join(DEFAULT_PROJECT_FILENAME);
            reset_ui(state);
            return Some("New project created".to_owned());
        }
        (KeyCode::Char('p'), KeyModifiers::ALT) => {
            let name = state.project()?.name.clone();
            set_prompt_extra(
                state,
                PromptRequest::SetProjectPassword,
                &format!("Set new password for `{name}`:"),
                "",
                true,
            );
        }
        (KeyCode::Char('r'), KeyModifiers::ALT) => {
            set_prompt(state, PromptRequest::RenameProject, "New Project Name:");
        }
        (KeyCode::Char('R'), KeyModifiers::SHIFT) => {
            let name = state.project()?.subproject()?.name.clone();
            set_prompt_extra(
                state,
                PromptRequest::RenameSubProject,
                "New Subproject Name:",
                &name,
                false,
            );
        }
        (KeyCode::Char('='), KeyModifiers::NONE) => {
            state.project()?.focused_width_percent += 5;
            bind_focus_size(state);
        }
        (KeyCode::Char('-'), KeyModifiers::NONE) => {
            state.project()?.focused_width_percent =
                state.project()?.focused_width_percent.saturating_sub(5);
            bind_focus_size(state);
        }
        (KeyCode::Char('N'), KeyModifiers::SHIFT) => {
            set_prompt(state, PromptRequest::AddSubProject, "New Subproject Name:");
        }
        (KeyCode::Char('D'), KeyModifiers::SHIFT) => {
            state.project()?.subprojects.pop_selected();
            bind_focus_size(state);
        }
        (KeyCode::Char('l'), KeyModifiers::NONE) => state.project()?.subprojects.select_next(),
        (KeyCode::Char('h'), KeyModifiers::NONE) => state.project()?.subprojects.select_prev(),
        (KeyCode::Char('l'), KeyModifiers::ALT) => {
            state.project()?.subprojects.move_down().ok();
        }
        (KeyCode::Char('h'), KeyModifiers::ALT) => {
            state.project()?.subprojects.move_up().ok();
        }
        (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
            if let Some(subproject) = state.project()?.subproject() {
                if let Some(task) = subproject.tasks.pop_selected() {
                    let target_subproject =
                        state.project.subprojects.next_item_mut().expect(
                            "next subproject should exist if at least one subproject exists",
                        );
                    target_subproject.tasks.insert_item(
                        target_subproject.tasks.selected(),
                        task,
                        true,
                    );
                    state.project.subprojects.select_next();
                }
            }
        }
        (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
            if let Some(subproject) = state.project()?.subproject() {
                if let Some(task) = subproject.tasks.pop_selected() {
                    let target_subproject =
                        state.project.subprojects.prev_item_mut().expect(
                            "prev subproject should exist if at least one subproject exists",
                        );
                    target_subproject.tasks.insert_item(
                        target_subproject.tasks.selected(),
                        task,
                        true,
                    );
                    state.project.subprojects.select_prev()
                }
            }
        }
        // Subproject operations
        (KeyCode::Char('n'), KeyModifiers::NONE) => {
            set_prompt(state, PromptRequest::AddTask, "New Task:");
        }
        (KeyCode::Char('d'), KeyModifiers::NONE) => {
            state.project()?.subproject()?.tasks.pop_selected();
        }
        (KeyCode::Char('r'), KeyModifiers::NONE) => {
            if let Some(task) = state.project()?.subproject()?.task() {
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
        // Subproject navigation
        (KeyCode::Esc, KeyModifiers::NONE) => {
            state.project()?.subproject()?.tasks.deselect();
        }
        (KeyCode::Char('j'), KeyModifiers::NONE) => {
            state.project()?.subproject()?.tasks.select_next();
        }
        (KeyCode::Char('k'), KeyModifiers::NONE) => {
            state.project()?.subproject()?.tasks.select_prev();
        }
        (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
            state.project()?.subproject()?.tasks.move_down().ok();
        }
        (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
            state.project()?.subproject()?.tasks.move_up().ok();
        }
        (KeyCode::Char('\\'), KeyModifiers::NONE) => {
            state.project()?.split_vertical = !state.project()?.split_vertical;
        }
        // File operations
        (KeyCode::Char('o'), KeyModifiers::CONTROL) => {
            state.file_request = Some(FileRequest::Load);
            state.filelist.refresh_filelist();
            state.filelist.set_title_text("Open Project:");
            state.filelist.set_prompt_text("Create New File:");
        }
        (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
            return match save_project(state, None) {
                Err(e) => Some(e),
                Ok(_) => Some(format!("Saved project: {:?}", state.filepath)),
            };
        }
        (KeyCode::Char('s'), KeyModifiers::ALT) => {
            state.file_request = Some(FileRequest::Save);
            state.filelist.refresh_filelist();
            state.filelist.set_title_text("Save Project:");
            state.filelist.set_prompt_text("Save File As:");
        }
        _ => (),
    };
    None
}

fn handle_prompt_event(key: KeyEvent, state: &mut App) -> Option<String> {
    if let Some(request) = state.project.prompt_request.clone() {
        match state.project.prompt.handle_event(key) {
            PromptEvent::Cancelled => state.project.prompt_request = None,
            PromptEvent::AwaitingResult(_) => (),
            PromptEvent::Result(result_text) => {
                clear_prompt(state);
                match request {
                    PromptRequest::SetProjectPassword => {
                        state.project.password = result_text;
                        return Some("Reset project password".to_owned());
                    }
                    PromptRequest::GetLoadPassword(name) => {
                        return match load_project(state, &name, &result_text) {
                            Err(e) => Some(e),
                            Ok(_) => Some(format!("Loaded project: {:?}", state.filepath)),
                        };
                    }
                    PromptRequest::RenameProject => {
                        state.project.name = result_text;
                        return Some(format!("Renamed project: {}", state.project.name));
                    }
                    PromptRequest::RenameSubProject => {
                        state.project()?.subproject()?.name = result_text;
                    }
                    PromptRequest::AddSubProject => {
                        if let Some(project) = state.project() {
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
                            .project()?
                            .subproject()?
                            .tasks
                            .add_item(Task::new(&result_text));
                    }
                    PromptRequest::RenameTask => {
                        state.project()?.subproject()?.task()?.desc = result_text.clone();
                    }
                };
                state.project.prompt_request = None;
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
                            Ok(_) => Some(format!("Saved project {:?}", state.filepath)),
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
    state.project.prompt.set_prompt_text("Input:");
    state.project.prompt.set_text("");
    state.project.prompt_request = None;
    state.project.prompt.set_password(false);
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
    state.project.prompt.set_prompt_text(prompt_text);
    state.project.prompt.set_text(prefill_text);
    state.project.prompt_request = Some(request);
    state.project.prompt.set_password(password);
}

fn reset_ui(state: &mut App) {
    state.project.focused_width_percent = DEFAULT_WIDTH_PERCENT;
    bind_focus_size(state);
}

fn bind_focus_size(state: &mut App) {
    let min_width = (100. / state.project.subprojects.items().len() as f32).max(5.) as u16;
    state.project.focused_width_percent =
        state.project.focused_width_percent.min(95).max(min_width);
}

fn open_datadir(state: &App) -> Option<String> {
    if let Err(e) = Command::new("xdg-open").arg(&state.datadir).spawn() {
        return Some(format!("failed to open {:?} [{e}]", &state.datadir,));
    }
    None
}

fn save_project(state: &mut App, filepath: Option<&PathBuf>) -> Result<(), String> {
    let filepath = filepath.unwrap_or(&state.filepath);
    save_encrypt(&state.project, filepath, &state.project.password)?;
    state.filelist.refresh_filelist();
    Ok(())
}

fn load_project(state: &mut App, name: &str, key: &str) -> Result<(), String> {
    let filepath = state.datadir.join(name);
    if !filepath.exists() {
        return Err("file does not exist".to_owned());
    }
    state.project = load_decrypt(&filepath, key)?;
    state.project.password = key.to_owned();
    state.filepath = filepath;
    state.filelist.refresh_filelist();
    reset_ui(state);
    Ok(())
}

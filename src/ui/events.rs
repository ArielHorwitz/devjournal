use super::widgets::{files::FileListResult, prompt::PromptEvent};
use crate::app::data::{
    App, FileRequest, Project, PromptRequest, SubProject, Task, DEFAULT_PROJECT_FILENAME,
    DEFAULT_WIDTH_PERCENT,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{path::PathBuf, process::Command};
use tui::layout::Direction;

pub fn handle_event(key: KeyEvent, state: &mut App) {
    match handle_global_event(key, state) {
        Some(feedback) => state.user_feedback_text = feedback,
        None => {
            let result = {
                if state.file_request.is_some() {
                    handle_filelist_event(key, state)
                } else if state.project_state.prompt_request.is_some() {
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
    let selected_subproject = state.project.subprojects.get_item_mut(None);
    match (key.code, key.modifiers) {
        // Project operations
        (KeyCode::Char('n'), KeyModifiers::ALT) => {
            state.project = Project::default();
            state.project_filepath = state.datadir.join(DEFAULT_PROJECT_FILENAME);
            reset_ui(state);
            return Some("New project created".to_owned());
        }
        (KeyCode::Char('p'), KeyModifiers::ALT) => {
            set_prompt_extra(
                state,
                PromptRequest::SetProjectPassword,
                &format!("Set new password for `{}`:", state.project.name),
                "",
                true,
            );
        }
        (KeyCode::Char('r'), KeyModifiers::ALT) => {
            set_prompt(state, PromptRequest::RenameProject, "New Project Name:");
        }
        (KeyCode::Char('R'), KeyModifiers::SHIFT) => {
            if let Some(subproject) = selected_subproject {
                let name = subproject.name.clone();
                set_prompt_extra(
                    state,
                    PromptRequest::RenameSubProject,
                    "New Subproject Name:",
                    &name,
                    false,
                );
            }
        }
        (KeyCode::Char('='), KeyModifiers::NONE) => {
            state.project_state.focused_width_percent += 5;
            bind_focus_size(state);
        }
        (KeyCode::Char('-'), KeyModifiers::NONE) => {
            state.project_state.focused_width_percent =
                state.project_state.focused_width_percent.saturating_sub(5);
            bind_focus_size(state);
        }
        (KeyCode::Char('N'), KeyModifiers::SHIFT) => {
            set_prompt(state, PromptRequest::AddSubProject, "New Subproject Name:");
        }
        (KeyCode::Char('D'), KeyModifiers::SHIFT) => {
            state.project.subprojects.pop_selected();
            bind_focus_size(state);
        }
        (KeyCode::Char('l'), KeyModifiers::NONE) => state.project.subprojects.select_next(),
        (KeyCode::Char('h'), KeyModifiers::NONE) => state.project.subprojects.select_prev(),
        (KeyCode::Char('l'), KeyModifiers::ALT) => {
            state.project.subprojects.move_down().ok();
        }
        (KeyCode::Char('h'), KeyModifiers::ALT) => {
            state.project.subprojects.move_up().ok();
        }
        (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
            if let Some(subproject) = selected_subproject {
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
            if let Some(subproject) = selected_subproject {
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
            if let Some(subproject) = selected_subproject {
                subproject.tasks.pop_selected();
            }
        }
        (KeyCode::Char('r'), KeyModifiers::NONE) => {
            if let Some(subproject) = selected_subproject {
                if let Some(task) = subproject.tasks.get_item_mut(None) {
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
        }
        // Subproject navigation
        (KeyCode::Esc, KeyModifiers::NONE) => {
            if let Some(subproject) = selected_subproject {
                subproject.tasks.deselect();
            }
        }
        (KeyCode::Char('j'), KeyModifiers::NONE) => {
            if let Some(subproject) = selected_subproject {
                subproject.tasks.select_next();
            }
        }
        (KeyCode::Char('k'), KeyModifiers::NONE) => {
            if let Some(subproject) = selected_subproject {
                subproject.tasks.select_prev();
            }
        }
        (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
            if let Some(subproject) = selected_subproject {
                subproject.tasks.move_down().ok();
            }
        }
        (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
            if let Some(subproject) = selected_subproject {
                subproject.tasks.move_up().ok();
            }
        }
        (KeyCode::Char('\\'), KeyModifiers::NONE) => {
            state.project_state.split_orientation = match state.project_state.split_orientation {
                Direction::Horizontal => Direction::Vertical,
                Direction::Vertical => Direction::Horizontal,
            };
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
                Ok(_) => Some(format!("Saved project: {:?}", state.project_filepath)),
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
    if let Some(pr) = state.project_state.prompt_request.clone() {
        match state.project_state.prompt.handle_event(key) {
            PromptEvent::Cancelled => state.project_state.prompt_request = None,
            PromptEvent::AwaitingResult(_) => (),
            PromptEvent::Result(result_text) => {
                clear_prompt(state);
                let subproject = state.project.subprojects.get_item_mut(None);
                match pr {
                    PromptRequest::SetProjectPassword => {
                        state.project_state.project_password = result_text;
                        return Some("Reset project password".to_owned());
                    }
                    PromptRequest::GetLoadPassword(name) => {
                        return match load_project(state, &name, &result_text) {
                            Err(e) => Some(e),
                            Ok(_) => Some(format!("Loaded project: {:?}", state.project_filepath)),
                        };
                    }
                    PromptRequest::RenameProject => {
                        state.project.name = result_text;
                        return Some(format!("Renamed project: {}", state.project.name));
                    }
                    PromptRequest::RenameSubProject => {
                        if let Some(subproject) = subproject {
                            subproject.name = result_text;
                        }
                    }
                    PromptRequest::AddSubProject => {
                        state.project.subprojects.insert_item(
                            state.project.subprojects.next_index(),
                            SubProject::new(&result_text),
                            true,
                        );
                        bind_focus_size(state);
                    }
                    PromptRequest::AddTask => {
                        if let Some(subproject) = subproject {
                            subproject.tasks.add_item(Task::new(&result_text));
                        };
                    }
                    PromptRequest::RenameTask => {
                        if let Some(subproject) = subproject {
                            if let Some(task) = subproject.tasks.get_item(None) {
                                let new_task = Task {
                                    desc: result_text.clone(),
                                    ..task.clone()
                                };
                                subproject.tasks.replace_selected(new_task);
                            }
                        }
                    }
                };
                state.project_state.prompt_request = None;
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
                            Ok(_) => Some(format!("Saved project {:?}", state.project_filepath)),
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
    state.project_state.prompt.set_prompt_text("Input:");
    state.project_state.prompt.set_text("");
    state.project_state.prompt_request = None;
    state.project_state.prompt.set_password(false);
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
    state.project_state.prompt.set_prompt_text(prompt_text);
    state.project_state.prompt.set_text(prefill_text);
    state.project_state.prompt_request = Some(request);
    state.project_state.prompt.set_password(password);
}

fn reset_ui(state: &mut App) {
    state.project_state.focused_width_percent = DEFAULT_WIDTH_PERCENT;
    bind_focus_size(state);
}

fn bind_focus_size(state: &mut App) {
    let min_width = (100. / state.project.subprojects.items().len() as f32).max(5.) as u16;
    state.project_state.focused_width_percent = state
        .project_state
        .focused_width_percent
        .min(95)
        .max(min_width);
}

fn open_datadir(state: &App) -> Option<String> {
    if let Err(e) = Command::new("xdg-open").arg(&state.datadir).spawn() {
        return Some(format!("failed to open {:?} [{e}]", &state.datadir,));
    }
    None
}

fn save_project(state: &mut App, filepath: Option<&PathBuf>) -> Result<(), String> {
    let filepath = filepath.unwrap_or(&state.project_filepath);
    state
        .project
        .save_file(filepath, &state.project_state.project_password)?;
    state.filelist.refresh_filelist();
    Ok(())
}

fn load_project(state: &mut App, name: &str, key: &str) -> Result<(), String> {
    let filepath = state.datadir.join(name);
    state.project = Project::from_file(&filepath, key)?;
    state.project_state.project_password = key.to_owned();
    state.project_filepath = filepath;
    state.filelist.refresh_filelist();
    reset_ui(state);
    Ok(())
}

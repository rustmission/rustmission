use std::sync::{Arc, Mutex};

use crate::ui::components::Component;

use ratatui::{prelude::*, widgets::Paragraph};
use rm_shared::{action::Action, status_task::StatusTask};
use throbber_widgets_tui::ThrobberState;
use tokio::time::{self, Instant};

pub struct StatusBar {
    task: StatusTask,
    pub task_status: CurrentTaskState,
}

impl StatusBar {
    pub const fn new(task: StatusTask, task_status: CurrentTaskState) -> Self {
        Self { task, task_status }
    }
}

fn format_display_name(name: &str) -> String {
    if name.len() < 60 {
        name.to_string()
    } else {
        let truncated = &name[0..59];
        format!("\"{truncated}...\"")
    }
}

impl Component for StatusBar {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        match &self.task_status {
            CurrentTaskState::Loading(state) => {
                let status_text = match &self.task {
                    StatusTask::Add(name) => {
                        let display_name = format_display_name(&name);
                        format!("Adding {display_name}")
                    }
                    StatusTask::Delete(name) => {
                        let display_name = format_display_name(&name);
                        format!("Deleting {display_name}")
                    }
                };
                let default_throbber = throbber_widgets_tui::Throbber::default()
                    .label(status_text)
                    .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));
                f.render_stateful_widget(
                    default_throbber.clone(),
                    rect,
                    &mut state.lock().unwrap(),
                );
            }
            task_state => {
                let status_text = match task_state {
                    CurrentTaskState::Failure => match &self.task {
                        StatusTask::Add(name) => {
                            let display_name = format_display_name(&name);
                            format!(" Error adding {display_name}")
                        }
                        StatusTask::Delete(name) => {
                            let display_name = format_display_name(&name);
                            format!(" Error deleting {display_name}")
                        }
                    },
                    CurrentTaskState::Success(_) => match &self.task {
                        StatusTask::Add(name) => {
                            let display_name = format_display_name(&name);
                            format!(" Added {display_name}")
                        }
                        StatusTask::Delete(name) => {
                            let display_name = format_display_name(&name);
                            format!(" Deleted {display_name}")
                        }
                    },
                    _ => return,
                };
                let mut line = Line::default();
                match task_state {
                    CurrentTaskState::Failure => {
                        line.push_span(Span::styled("", Style::default().red()));
                    }
                    CurrentTaskState::Success(_) => {
                        line.push_span(Span::styled("", Style::default().green()));
                    }
                    _ => return,
                }
                line.push_span(Span::raw(status_text));
                let paragraph = Paragraph::new(line);
                f.render_widget(paragraph, rect);
            }
        }
    }

    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::Tick => self.tick(),
            Action::TaskSuccess => {
                self.task_status.set_success(time::Instant::now());
                Some(Action::Render)
            }
            Action::Error(_) => {
                self.task_status.set_failure();
                Some(Action::Render)
            }
            _ => Some(action),
        }
    }

    fn tick(&mut self) -> Option<Action> {
        match &self.task_status {
            CurrentTaskState::Loading(state) => {
                state.lock().unwrap().calc_next();
                Some(Action::Render)
            }
            CurrentTaskState::Success(start) => {
                let expiration_duration = time::Duration::from_secs(5);
                if start.elapsed() >= expiration_duration {
                    return Some(Action::Quit);
                }
                None
            }
            _ => None,
        }
    }
}

#[derive(Clone)]
pub enum CurrentTaskState {
    Loading(Arc<Mutex<ThrobberState>>),
    Success(Instant),
    Failure,
}

impl CurrentTaskState {
    fn set_failure(&mut self) {
        *self = Self::Failure;
    }

    fn set_success(&mut self, start: Instant) {
        *self = Self::Success(start);
    }
}

use crossterm::event::KeyCode;

use std::time::Duration;

use crate::utils;

#[derive(Clone)]
pub enum AppEvent {
    Stop,
    Quit,
    None,
    Toggle,
}

#[derive(Clone)]
pub enum AppStatus {
    Running, // TODO better variable name
    Paused,
    Quit,
}

impl AppEvent {
    pub fn from_keycode(keycode: KeyCode) -> utils::MyResult<AppEvent> {
        match keycode {
            KeyCode::Char(char) => match char {
                'q' => Ok(AppEvent::Quit),
                ' ' => Ok(AppEvent::Toggle),
                's' => Ok(AppEvent::Stop),
                _ => Ok(AppEvent::None),
            },
            _ => Ok(AppEvent::None),
        }
    }
}

#[derive(Clone)]
pub struct App {
    work_duration: u64,  // in seconds
    break_duration: u64, // in seconds
    progress: u64,       // in seconds
    is_working: bool,
    status: AppStatus,
}

impl App {
    pub fn new(work_duration: u64, break_duration: u64) -> App {
        App {
            work_duration,
            break_duration,
            progress: work_duration,
            is_working: true,
            status: AppStatus::Running,
        }
    }

    fn update_progress(&mut self) {
        if matches!(self.status, AppStatus::Running) && self.progress > 0 {
            self.progress -= 1;
        }

        if self.progress == 0 {
            self.status = AppStatus::Paused;

            if self.is_working {
                // start break timer
                self.progress = self.break_duration;
                self.is_working = false;
            } else {
                // start work timer
                self.progress = self.work_duration;
                self.is_working = true;
            }
        }
    }

    fn toggle(&mut self) {
        if matches!(self.status, AppStatus::Running) {
            self.status = AppStatus::Paused;
        } else {
            self.status = AppStatus::Running;
        }
    }

    pub fn on_tick(&mut self) {
        self.update_progress();
    }

    fn stop(&mut self) {
        self.status = AppStatus::Paused;
        self.progress = self.work_duration;
    }

    pub fn get_progress(&self) -> String {
        let duration = Duration::from_secs(self.progress);
        let hours = duration.as_secs() / 3600;
        let minutes = (duration.as_secs() % 3600) / 60;
        let seconds = duration.as_secs() % 60;

        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    pub fn set_work_duration(&mut self, secs: u64) {
        self.progress = secs;
        self.work_duration = secs;
    }

    pub fn set_break_duration(&mut self, secs: u64) {
        self.break_duration = secs;
    }

    pub fn get_status(&self) -> AppStatus {
        self.status.clone()
    }

    pub fn on(&mut self, event: AppEvent) {
        match event {
            AppEvent::Quit => {
                self.status = AppStatus::Quit;
            }
            AppEvent::Toggle => {
                self.toggle();
            }
            AppEvent::Stop => {
                self.stop();
            }
            AppEvent::None => (),
        };
    }
}

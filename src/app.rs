use crate::Utils;
use std::time::Duration;

#[derive(Clone)]
pub enum AppEvent {
    Stop,
    Quit,
    None,
    Toggle,
}

impl AppEvent {
    pub fn from_char(input: char) -> Utils::MyResult<AppEvent> {
        match input {
            'q' => Ok(AppEvent::Quit),
            ' ' => Ok(AppEvent::Toggle),
            's' => Ok(AppEvent::Stop),
            _ => Ok(AppEvent::None),
        }
    }
}

pub struct App {
    work_duration: u64,  // in seconds
    break_duration: u64, // in seconds
    progress: u64,       // in seconds
    is_working: bool,
    pub is_running: bool,
    pub should_quit: bool,
}

impl App {
    pub fn new(work_duration: u64, break_duration: u64) -> App {
        App {
            work_duration,
            break_duration,
            progress: work_duration,
            is_running: true,
            is_working: true,
            should_quit: false,
        }
    }

    fn update_progress(&mut self) {
        if self.is_running && self.progress > 0 {
            self.progress -= 1;
        } else if self.progress == 0 {
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
        self.is_running = !self.is_running
    }

    pub fn on_tick(&mut self) {
        self.update_progress();
    }

    fn stop(&mut self) {
        self.is_running = false;
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

    pub fn on(&mut self, event: AppEvent) {
        match event {
            AppEvent::Quit => {
                self.should_quit = true;
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

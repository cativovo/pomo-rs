mod app;
mod ui;
mod utils;

use app::{App, AppStatus};
use crossterm::event::poll;
use std::{
    io,
    time::{Duration, Instant},
};
use tui::{backend::CrosstermBackend, Terminal};
use ui::Ui;
use utils::{format_secs, MyResult};

pub fn start() -> MyResult<()> {
    let stdout = io::stdout();
    let mut app = App::new(5, 3);
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let work_time = format_secs(app.get_work_duration());
    let break_time = format_secs(app.get_break_duration());
    let mut ui = Ui::new(
        "Pomodoro",
        ["Hours", "Minutes", "Seconds"],
        [
            work_time.map(|e| e.to_string()),
            break_time.map(|e| e.to_string()),
        ],
    );
    ui.setup_terminal()?;

    let tick_rate = Duration::from_secs(1);
    let mut last_tick = Instant::now();

    loop {
        ui.draw(&mut terminal, &mut app)?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_secs(0));

        if matches!(app.get_status(), AppStatus::Paused) || poll(timeout)? {
            // blocks the current thread
            ui.handle_keypress(&mut app)?;
        };

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }

        if let AppStatus::Quit = app.get_status() {
            ui.cleanup()?;

            return Ok(());
        }
    }
}

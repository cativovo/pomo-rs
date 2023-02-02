mod app;
mod ui;
mod utils;

use app::{App, AppEvent};
use crossterm::event::poll;
use std::{
    io,
    time::{Duration, Instant},
};
use tui::{backend::CrosstermBackend, Terminal};
use ui::{cleanup, draw, get_pressed_key, setup_terminal};
use utils::MyResult;

pub fn start() -> MyResult<()> {
    let stdout = io::stdout();
    let mut app = App::new(5, 3);
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    setup_terminal()?;

    let tick_rate = Duration::from_secs(1);
    let mut last_tick = Instant::now();

    loop {
        draw(&mut terminal, &mut app)?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_secs(0));

        if !app.is_running || poll(timeout)? {
            // blocks the current thread
            if let Some(pressed_key) = get_pressed_key()? {
                let event = AppEvent::from_char(pressed_key)?;
                app.on(event);
            }
        };

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }

        if app.should_quit {
            cleanup()?;

            return Ok(());
        }
    }
}

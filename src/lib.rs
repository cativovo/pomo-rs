use std::{
    error::Error,
    io::{self, Stdout},
    time::{Duration, Instant},
};

use crossterm::{
    cursor,
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Clone)]
enum AppEvent {
    Stop,
    Quit,
    None,
    Toggle,
}

impl AppEvent {
    fn from_char(input: char) -> MyResult<AppEvent> {
        match input {
            'q' => Ok(AppEvent::Quit),
            ' ' => Ok(AppEvent::Toggle),
            's' => Ok(AppEvent::Stop),
            _ => Ok(AppEvent::None),
        }
    }
}

// TODO: move to its own module
struct App {
    work_duration: u64,  // in seconds
    break_duration: u64, // in seconds
    progress: u64,       // in seconds
    is_running: bool,
    is_working: bool,
}

impl App {
    fn new(work_duration: u64, break_duration: u64) -> App {
        App {
            work_duration,
            break_duration,
            progress: work_duration,
            is_running: true,
            is_working: true,
        }
    }

    fn decrease_progress(&mut self) {
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

    fn on_tick(&mut self) {
        self.decrease_progress();
    }

    fn stop(&mut self) {
        self.is_running = false;
        self.progress = self.work_duration;
    }

    fn get_progress(&self) -> String {
        let duration = Duration::from_secs(self.progress);
        let hours = duration.as_secs() / 3600;
        let minutes = (duration.as_secs() % 3600) / 60;
        let seconds = duration.as_secs() % 60;

        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    fn set_work_duration(&mut self, secs: u64) {
        self.progress = secs;
        self.work_duration = secs;
    }

    fn set_break_duration(&mut self, secs: u64) {
        self.break_duration = secs;
    }
}

fn setup_terminal() -> MyResult<()> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;

    Ok(())
}

fn cleanup() -> MyResult<()> {
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    execute!(stdout, cursor::Show)?;
    disable_raw_mode()?;

    Ok(())
}

fn draw(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> MyResult<()> {
    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default().title("Pomodoro").borders(Borders::ALL);

        // move me
        let paragraph = Paragraph::new(Span::raw(app.get_progress()))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true });
        let paragraph_width = size.width / 2;
        let paragraph_height = size.height / 2;
        let paragraph_posx = (size.width / 2) - (paragraph_width / 2);
        let paragraph_posy = (size.height / 2) - (paragraph_height / 2);

        f.render_widget(block, size);
        f.render_widget(
            paragraph,
            Rect::new(
                paragraph_posx,
                paragraph_posy,
                paragraph_width,
                paragraph_height,
            ),
        );
    })?;

    Ok(())
}

fn get_pressed_key() -> MyResult<Option<char>> {
    // `read()` blocks until an `Event` is available
    match read()? {
        Event::Key(event) => {
            if let KeyCode::Char(char) = event.code {
                return Ok(Some(char));
            }
        }
        _ => {}
    };

    Ok(None)
}

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
                let app_event = AppEvent::from_char(pressed_key)?;
                match app_event {
                    AppEvent::Quit => {
                        break;
                    }
                    AppEvent::Toggle => {
                        app.toggle();
                    }
                    AppEvent::Stop => {
                        app.stop();
                    }
                    AppEvent::None => (),
                };
            }
        };

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }

    cleanup()?;

    Ok(())
}

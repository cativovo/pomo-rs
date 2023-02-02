use crate::{app::App, utils::MyResult};
use crossterm::{
    cursor,
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Stdout};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

pub fn setup_terminal() -> MyResult<()> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;

    Ok(())
}

pub fn cleanup() -> MyResult<()> {
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    execute!(stdout, cursor::Show)?;
    disable_raw_mode()?;

    Ok(())
}

pub fn draw(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> MyResult<()> {
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

pub fn get_pressed_key() -> MyResult<Option<char>> {
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

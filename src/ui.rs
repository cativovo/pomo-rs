use crate::{
    app::{App, AppEvent, AppStatus},
    utils::MyResult,
};
use crossterm::{
    cursor,
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Stdout};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

enum InputFocus {
    Hours,
    Minutes,
    Seconds,
}

impl InputFocus {
    fn from_usize(index: usize) -> InputFocus {
        match index {
            0 => InputFocus::Hours,
            1 => InputFocus::Minutes,
            2 => InputFocus::Seconds,
            _ => {
                if index > 2 {
                    InputFocus::Hours
                } else {
                    InputFocus::Seconds
                }
            }
        }
    }

    fn to_usize(input_focus: &InputFocus) -> usize {
        match input_focus {
            InputFocus::Hours => 0,
            InputFocus::Minutes => 1,
            InputFocus::Seconds => 2,
        }
    }
}

enum UiMode {
    Normal,
    EditingWork,
    EditingBreak,
}

impl UiMode {
    pub fn from_keycode(keycode: &KeyCode) -> Option<UiMode> {
        match keycode {
            KeyCode::Esc => return Some(UiMode::Normal),
            KeyCode::Char(char) => match char {
                'b' => return Some(UiMode::EditingWork),
                'w' => return Some(UiMode::EditingBreak),
                _ => (),
            },
            _ => (),
        }

        None
    }
}

pub struct Ui<'a> {
    title: &'a str,
    stdout: Stdout,
    input: String,
    mode: UiMode,
    input_focus: usize,
    input_titles: [&'a str; 3],
}

impl<'a> Ui<'a> {
    fn next(&mut self) {
        self.input_focus = (self.input_focus + 1) % self.input_titles.len();
    }

    fn prev(&mut self) {
        if self.input_focus > 0 {
            self.input_focus -= 1;
        } else {
            self.input_focus = self.input_titles.len() - 1;
        }
    }

    fn select_tab(&mut self, keycode: &KeyCode) {
        match keycode {
            KeyCode::Tab => {
                self.next();
            }
            KeyCode::BackTab => {
                self.prev();
            }
            _ => (),
        };
    }

    fn set_input(&mut self, c: char) {
        if c.is_numeric() {
            self.input.push(c);
        }
    }

    fn delete_input(&mut self) {
        self.input.pop();
    }

    pub fn new(title: &'a str, input_titles: [&'a str; 3]) -> Ui<'a> {
        Ui {
            title,
            stdout: io::stdout(),
            input: String::new(),
            mode: UiMode::Normal,
            input_focus: 0,
            input_titles,
        }
    }

    pub fn setup_terminal(&mut self) -> MyResult<()> {
        execute!(self.stdout, EnterAlternateScreen, EnableMouseCapture)?;
        enable_raw_mode()?;

        Ok(())
    }

    pub fn cleanup(&mut self) -> MyResult<()> {
        execute!(self.stdout, LeaveAlternateScreen, DisableMouseCapture)?;
        execute!(self.stdout, cursor::Show)?;
        disable_raw_mode()?;

        Ok(())
    }

    fn render_timer(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, progress: String) {
        let mut size = frame.size().clone();
        let paragraph = Paragraph::new(Span::raw(progress))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true });
        size.width = size.width / 2;
        size.height = size.height / 2;
        size.x = size.width / 2;
        size.y = size.height / 2;

        frame.render_widget(paragraph, size);
    }

    fn render_input(&self, frame: &mut Frame<CrosstermBackend<Stdout>>) {
        let mut size = frame.size().clone();
        size.width = size.width / 2;
        size.x = size.width / 2;
        size.height = (size.height as f64 * 0.35).floor() as u16;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(5)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(size);

        let titles = self
            .input_titles
            .iter()
            .map(|t| {
                let (first, rest) = t.split_at(1);
                Spans::from(vec![
                    Span::styled(first, Style::default().fg(Color::Yellow)),
                    Span::styled(rest, Style::default().fg(Color::Green)),
                ])
            })
            .collect();

        let selected = self.input_focus;

        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("Tabs"))
            .select(selected)
            .style(Style::default().fg(Color::Cyan))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Black),
            );
        frame.render_widget(tabs, chunks[0]);

        let selected_title = self.input_titles[selected];
        let input = self.input.as_str();
        let paragraph = Paragraph::new(input)
            .style(Style::default())
            .block(Block::default().borders(Borders::ALL).title(selected_title));

        frame.render_widget(paragraph, chunks[1]);
        // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
        frame.set_cursor(
            // Put cursor past the end of the input text
            chunks[1].x + UnicodeWidthStr::width(input) as u16 + 1,
            // Move one line down, from the border to the input line
            chunks[1].y + 1,
        );
    }

    pub fn draw(
        &self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        app: &mut App,
    ) -> MyResult<()> {
        terminal.draw(|frame| {
            let block = Block::default().title(self.title).borders(Borders::ALL);

            frame.render_widget(block, frame.size());
            self.render_timer(frame, app.get_progress());

            if !matches!(self.mode, UiMode::Normal) {
                self.render_input(frame);
            }
        })?;

        Ok(())
    }

    pub fn handle_keypress(&mut self, app: &mut App) -> MyResult<()> {
        // `read()` blocks until an `Event` is available
        if let Event::Key(event) = read()? {
            let keycode = event.code;

            if let Some(ui_mode) = UiMode::from_keycode(&keycode) {
                self.mode = ui_mode;
            }

            if !matches!(self.mode, UiMode::Normal) {
                self.select_tab(&keycode);

                match keycode {
                    KeyCode::Char(c) => {
                        self.set_input(c);
                    }
                    KeyCode::Backspace => {
                        self.delete_input();
                    }
                    _ => (),
                }
            }

            let event = AppEvent::from_keycode(keycode)?;
            app.on(event);
        }

        Ok(())
    }
}

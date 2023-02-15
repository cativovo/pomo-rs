use crate::{
    app::{App, AppEvent, AppStatus},
    utils::{format_secs, to_secs, MyResult},
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

pub enum UiMode {
    Normal,
    EditingWork,
    EditingBreak,
}

impl UiMode {
    fn to_usize(ui_mode: &UiMode) -> usize {
        match ui_mode {
            UiMode::EditingWork | UiMode::Normal => 0,
            UiMode::EditingBreak => 1,
        }
    }

    pub fn from_keycode(keycode: &KeyCode) -> Option<UiMode> {
        match keycode {
            KeyCode::Esc => return Some(UiMode::Normal),
            KeyCode::Char(char) => match char {
                'w' => return Some(UiMode::EditingWork),
                'b' => return Some(UiMode::EditingBreak),
                _ => None,
            },
            _ => None,
        }
    }
}

pub struct Ui<'a> {
    title: &'a str,
    stdout: Stdout,
    inputs: [[String; 3]; 2], //[work time, break time] hours, minutes, seconds
    mode: UiMode,
    tab_focus: usize,
    input_titles: [&'a str; 3], // hours, minutes, seconds
}

impl<'a> Ui<'a> {
    fn next_tab(&mut self) {
        self.tab_focus = (self.tab_focus + 1) % self.input_titles.len();
    }

    fn prev_tab(&mut self) {
        if self.tab_focus > 0 {
            self.tab_focus -= 1;
        } else {
            self.tab_focus = self.input_titles.len() - 1;
        }
    }

    fn set_initial_tab_focus(&mut self) {
        self.tab_focus = 0;
    }

    fn select_tab(&mut self, keycode: &KeyCode) {
        match keycode {
            KeyCode::Tab => {
                self.next_tab();
            }
            KeyCode::BackTab => {
                self.prev_tab();
            }
            _ => (),
        };
    }

    fn set_input(&mut self, c: char) {
        if c.is_numeric() {
            let tab_focus = self.tab_focus;

            if tab_focus == 1 || tab_focus == 2 {
                let mut input_clone = self.get_input().clone();
                input_clone.push(c);

                let duration: u64 = input_clone.parse().unwrap_or(0);

                if duration < 60 {
                    self.inputs[UiMode::to_usize(&self.mode)][self.tab_focus].push(c);
                }
            } else {
                self.inputs[UiMode::to_usize(&self.mode)][self.tab_focus].push(c);
            }
        }
    }

    fn delete_input(&mut self) {
        self.inputs[UiMode::to_usize(&self.mode)][self.tab_focus].pop();
    }

    fn get_input(&self) -> String {
        self.inputs[UiMode::to_usize(&self.mode)][self.tab_focus].clone()
    }

    fn get_inputs(&self) -> [String; 3] {
        self.inputs[UiMode::to_usize(&self.mode)].clone()
    }

    pub fn new(title: &'a str, input_titles: [&'a str; 3], inputs: [[String; 3]; 2]) -> Ui<'a> {
        Ui {
            title,
            stdout: io::stdout(),
            inputs,
            mode: UiMode::Normal,
            tab_focus: 0,
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
        let selected = self.tab_focus;
        let selected_title = self.input_titles[selected];
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
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("Tabs"))
            .select(selected)
            .style(Style::default().fg(Color::Cyan))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Black),
            );
        let input = self.get_input();
        let time = input.parse::<u64>().unwrap_or(0);
        let time = {
            if time > 0 {
                time.to_string()
            } else {
                "".to_string()
            }
        };
        let time = time.as_str();

        let paragraph = Paragraph::new(time)
            .style(Style::default())
            .block(Block::default().borders(Borders::ALL).title(selected_title));

        frame.render_widget(tabs, chunks[0]);
        frame.render_widget(paragraph, chunks[1]);
        // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
        frame.set_cursor(
            // Put cursor past the end of the input text
            chunks[1].x + UnicodeWidthStr::width(time) as u16 + 1,
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
                app.stop();

                match keycode {
                    KeyCode::Char(c) => {
                        self.set_input(c);
                    }
                    KeyCode::Backspace => {
                        self.delete_input();
                    }
                    _ => (),
                }

                let [hours, minutes, secs] =
                    self.get_inputs().map(|e| e.parse::<u64>().unwrap_or(0));
                let secs = to_secs(hours, minutes, secs);

                match self.mode {
                    UiMode::EditingWork => {
                        app.set_work_duration(secs);
                    }
                    UiMode::EditingBreak => {
                        app.set_break_duration(secs);
                    }
                    _ => (),
                }
            }

            // reset tab focus
            if matches!(keycode, KeyCode::Esc) {
                self.set_initial_tab_focus()
            }

            let event = AppEvent::from_keycode(keycode)?;
            app.on(event);
        }

        Ok(())
    }
}

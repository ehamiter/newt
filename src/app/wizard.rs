//! Wizard event loop and user interaction.

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use super::state::App;
use super::types::{Answers, Step};
use crate::ui;

/// Run the interactive wizard and return the collected answers.
pub fn run_wizard(project_name: &str) -> Result<Option<Answers>, Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new(project_name.to_string());

    let result = event_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    match result {
        Ok(true) => Ok(Some(app.answers())),
        Ok(false) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Main event loop handling keyboard input.
fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<bool, Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if let Event::Key(key) = event::read()? {
            // Handle global quit
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                return Ok(false);
            }

            // Handle step-specific input
            if app.step == Step::Summary {
                match key.code {
                    KeyCode::Enter | KeyCode::Char('y') => return Ok(true),
                    KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('q') => return Ok(false),
                    KeyCode::Left | KeyCode::Char('b') => {
                        app.step = app.step.prev().unwrap_or(Step::Summary);
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if let Some(list) = app.active_list_mut() {
                            list.move_up();
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if let Some(list) = app.active_list_mut() {
                            list.move_down();
                        }
                    }
                    KeyCode::Char(' ') => {
                        if let Some(list) = app.active_list_mut() {
                            list.toggle();
                        }
                    }
                    KeyCode::Enter | KeyCode::Right => {
                        if let Some(next) = app.step.next() {
                            app.step = next;
                        } else {
                            return Ok(true);
                        }
                    }
                    KeyCode::Esc | KeyCode::Left | KeyCode::Char('b') => {
                        match app.step.prev() {
                            Some(prev) => app.step = prev,
                            None => return Ok(false),
                        }
                    }
                    KeyCode::Char('q') => return Ok(false),
                    _ => {}
                }
            }
        }
    }
}
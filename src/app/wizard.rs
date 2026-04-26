//! Wizard event loop and user interaction.

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

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

    // Enable mouse capture for better terminal experience
    let _ = crossterm::execute!(io::stdout(), crossterm::event::EnableMouseCapture);

    let result = event_loop(&mut terminal, &mut app);

    // Clean up mouse capture
    let _ = crossterm::execute!(io::stdout(), crossterm::event::DisableMouseCapture);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    match result {
        Ok(true) => Ok(Some(app.answers())),
        Ok(false) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Main event loop handling keyboard input and window resize.
fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<bool, Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        // Use poll with timeout to handle resize events without blocking
        if let Ok(true) = event::poll(Duration::from_millis(100)) {
            match event::read()? {
                // Handle window resize events
                Event::Resize(_, _) => {
                    // Terminal is automatically resized by ratatui
                    continue;
                }
                // Handle key events with proper key event kind filtering
                Event::Key(key) => {
                    // Skip key release events, only process press
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    // Handle global quit
                    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                        return Ok(false);
                    }

                    // Handle step-specific input
                    if app.step == Step::Summary {
                        match key.code {
                            KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => return Ok(true),
                            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Char('q') => return Ok(false),
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
                            // Select all in current step
                            KeyCode::Char('a') => {
                                if let Some(list) = app.active_list_mut() {
                                    for item in &mut list.items {
                                        item.selected = true;
                                    }
                                }
                            }
                            // Deselect all in current step
                            KeyCode::Char('A') => {
                                if let Some(list) = app.active_list_mut() {
                                    for item in &mut list.items {
                                        item.selected = false;
                                    }
                                }
                            }
                            // Jump to first item
                            KeyCode::Home => {
                                if let Some(list) = app.active_list_mut() {
                                    list.cursor = 0;
                                }
                            }
                            // Jump to last item
                            KeyCode::End => {
                                if let Some(list) = app.active_list_mut() {
                                    list.cursor = list.items.len().saturating_sub(1);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                // Ignore other events
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wizard_step_navigation() {
        let step = Step::Languages;
        assert_eq!(step.next(), Some(Step::Databases));
        assert_eq!(step.prev(), None);

        let step = Step::Summary;
        assert_eq!(step.next(), None);
        assert_eq!(step.prev(), Some(Step::Gitignore));
    }
}
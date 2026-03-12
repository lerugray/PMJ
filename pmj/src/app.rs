/// App module — main event loop and terminal management.

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::game::GameState;
use crate::input;
use crate::ui;

pub fn run() -> io::Result<()> {
    // Set up terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture,
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create game state
    let mut game = GameState::new();

    // Main loop
    loop {
        terminal.draw(|f| ui::draw(f, &game))?;

        // Poll for events with a timeout (allows future animation)
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                let should_quit = input::handle_key(&mut game, key);
                if should_quit {
                    break;
                }
            }
        }
    }

    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

    Ok(())
}

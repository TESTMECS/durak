use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

mod app;
mod game;
mod ui;

use app::App;

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new();
    let res = app.run(&mut terminal);
    
    // At this point, safe_exit should have restored the terminal if 
    // an error occurred within the app.run function.
    // We only need to restore for cases where safe_exit wasn't called
    if res.is_ok() {
        // Normal exit - restore terminal if needed
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
    } else if let Err(err) = res {
        // Something went wrong but safe_exit wasn't called - restore now
        // Try to restore terminal state, but don't fail if we can't
        let _ = disable_raw_mode();
        let _ = terminal.backend_mut().execute(LeaveAlternateScreen);
        let _ = terminal.show_cursor();
        
        // Print the error
        eprintln!("Error: {:?}", err);
        return Err(err.into());
    }

    Ok(())
}

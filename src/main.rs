use anyhow::Result;
use crossterm::{
    ExecutableCommand, execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
mod app;
mod game;
mod ui;
use app::App;
extern crate lazy_static;
extern crate log;
extern crate ratatui;
fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();
    let res = app.run(&mut terminal);
    if res.is_ok() {
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
    } else if let Err(err) = res {
        let _ = disable_raw_mode();
        let _ = terminal.backend_mut().execute(LeaveAlternateScreen);
        let _ = terminal.show_cursor();
        // Print the error
        eprintln!("Error: {:?}", err);
        return Err(err.into());
    }
    Ok(())
}

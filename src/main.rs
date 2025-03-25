use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::{error::Error, io, fs::File, io::Write};
use std::env;
use log::{LevelFilter, info, error};
use env_logger::Builder;
use env_logger::WriteStyle;
use lazy_static::lazy_static;
use std::sync::Mutex;

mod app;
mod game;
mod ui;

use app::App;

lazy_static! {
    // Store log messages for the debug overlay
    static ref LOG_BUFFER: Mutex<Vec<(String, String, log::Level)>> = Mutex::new(Vec::new());
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup logging
    setup_logging()?;
    info!("Starting Durak game");
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create app and run it
    let mut app = App::new();
    info!("App initialized, starting main loop");
    let result = app.run(&mut terminal);
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    if let Err(err) = result {
        error!("Error running app: {:?}", err);
        println!("Error: {:?}", err);
    }
    
    info!("Shutting down Durak game");
    Ok(())
}

// Create our custom logger that logs to file and stores in memory for the overlay
fn setup_logging() -> Result<(), Box<dyn Error>> {
    let mut builder = Builder::new();
    builder.filter_level(LevelFilter::Debug);
    builder.write_style(WriteStyle::Always);

    // Check for debug mode with file output
    let debug_file = env::var("DURAK_DEBUG_FILE").unwrap_or_default();
    if !debug_file.is_empty() {
        let file = File::create(debug_file)?;
        builder.target(env_logger::Target::Pipe(Box::new(file)));
    } else {
        // Default to nothing for env_logger
        builder.target(env_logger::Target::Pipe(Box::new(io::sink())));
    }

    // Set up a custom formatter to capture logs for our display system
    builder.format(|buf, record| {
        let timestamp = buf.timestamp();
        let level = record.level();
        let args = record.args().to_string();
        
        // Store in our global buffer
        if let Ok(mut buffer) = LOG_BUFFER.lock() {
            // Limit size to avoid memory issues
            if buffer.len() >= 100 {
                buffer.remove(0);
            }
            buffer.push((timestamp.to_string(), args.clone(), level));
        }
        
        // Normal formatting for the file output
        writeln!(buf, "[{}] {}: {}", timestamp, level, args)
    });

    builder.init();
    
    Ok(())
}

// Retrieves the log buffer for display in the debug overlay
pub fn get_log_messages() -> Vec<(String, String, log::Level)> {
    if let Ok(buffer) = LOG_BUFFER.lock() {
        buffer.clone()
    } else {
        Vec::new()
    }
}

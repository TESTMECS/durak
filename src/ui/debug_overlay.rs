use lazy_static::lazy_static;
use log::Level;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::sync::Mutex;
// Buffer to hold our UI log messages
lazy_static! {
    static ref UI_LOG_BUFFER: Mutex<Vec<(String, String, Level)>> = Mutex::new(Vec::new());
}

#[allow(dead_code)]
pub struct LogMessage {
    pub timestamp: String,
    pub message: String,
    pub level: LogLevel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<Level> for LogLevel {
    fn from(level: Level) -> Self {
        match level {
            Level::Error => LogLevel::Error,
            Level::Warn => LogLevel::Warn,
            Level::Info => LogLevel::Info,
            Level::Debug => LogLevel::Debug,
            Level::Trace => LogLevel::Trace,
        }
    }
}

// Logging functions that log to both the regular system and the UI overlay
pub fn debug<S: AsRef<str>>(message: S) {
    let message_ref = message.as_ref();
    log_message(message_ref, LogLevel::Debug);
}

pub fn info<S: AsRef<str>>(message: S) {
    let message_ref = message.as_ref();
    log_message(message_ref, LogLevel::Info);
}
#[allow(dead_code)]
pub fn warn<S: AsRef<str>>(message: S) {
    let message_ref = message.as_ref();
    log_message(message_ref, LogLevel::Warn);
}

pub fn error<S: AsRef<str>>(message: S) {
    let message_ref = message.as_ref();
    log_message(message_ref, LogLevel::Error);
}

pub fn trace<S: AsRef<str>>(message: S) {
    let message_ref = message.as_ref();
    log_message(message_ref, LogLevel::Trace);
}

// Add a message to our UI log buffer
fn log_message(message: &str, level: LogLevel) {
    // Create timestamp
    let now = chrono::Local::now();
    let timestamp = now.format("%H:%M:%S%.3f").to_string();

    if let Ok(mut buffer) = UI_LOG_BUFFER.lock() {
        // Keep only the last 100 messages to avoid memory issues
        if buffer.len() >= 100 {
            buffer.remove(0);
        }

        // Convert LogLevel to log::Level for storage
        let log_level = match level {
            LogLevel::Error => Level::Error,
            LogLevel::Warn => Level::Warn,
            LogLevel::Info => Level::Info,
            LogLevel::Debug => Level::Debug,
            LogLevel::Trace => Level::Trace,
        };

        buffer.push((timestamp, message.to_string(), log_level));
    }
}

// Debug overlay widget
pub struct DebugOverlay {}

impl DebugOverlay {
    pub fn new() -> Self {
        Self {}
    }

    fn get_log_color(level: LogLevel) -> Color {
        match level {
            LogLevel::Error => Color::Red,
            LogLevel::Warn => Color::Yellow,
            LogLevel::Info => Color::Green,
            LogLevel::Debug => Color::Blue,
            LogLevel::Trace => Color::DarkGray,
        }
    }
}

impl Widget for DebugOverlay {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Only use bottom half of the screen for logs
        let log_area_height = area.height.saturating_mul(2) / 3;
        let log_area = Rect {
            x: area.x,
            y: area.height.saturating_sub(log_area_height),
            width: area.width,
            height: log_area_height,
        };
        // Create a background for our debug area
        let debug_block = Block::default()
            .title(" Debug Overlay [d to toggle] ")
            .borders(Borders::ALL)
            .style(
                Style::default()
                    .bg(Color::Black)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );
        // Get inner area before rendering the block
        let inner_area = debug_block.inner(log_area);
        // Render the block background
        debug_block.render(log_area, buf);
        // Get log messages from our buffer
        let messages = if let Ok(buffer) = UI_LOG_BUFFER.lock() {
            buffer.clone()
        } else {
            Vec::new()
        };
        // Create text for log messages
        let mut text = Vec::new();
        for (timestamp, message, level) in
            messages.into_iter().rev().take(inner_area.height as usize)
        {
            let log_level = LogLevel::from(level);
            let level_str = format!(
                "[{}]",
                match log_level {
                    LogLevel::Error => "ERROR",
                    LogLevel::Warn => "WARN ",
                    LogLevel::Info => "INFO ",
                    LogLevel::Debug => "DEBUG",
                    LogLevel::Trace => "TRACE",
                }
            );
            let line = Line::from(vec![
                Span::styled(
                    format!("{} ", timestamp),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{} ", level_str),
                    Style::default().fg(Self::get_log_color(log_level)),
                ),
                Span::raw(message),
            ]);
            text.push(line);
        }
        // Create the paragraph and render it within the block's inner area
        let paragraph =
            Paragraph::new(text).style(Style::default().bg(Color::Black).fg(Color::White));
        paragraph.render(inner_area, buf);
    }
}

use lazy_static::lazy_static;
use log::Level;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::fmt::Write;
use std::sync::Mutex;
lazy_static! {
    static ref UI_LOG_BUFFER: Mutex<Vec<(String, String, Level)>> = Mutex::new(Vec::new());
}
const BUFFER_LIMIT: usize = 100;
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
static LEVEL_STR: [&str; 5] = ["ERROR", "WARN ", "INFO ", "DEBUG", "TRACE"];
static LEVEL_COLOR: [Color; 5] = [
    Color::Red,
    Color::Yellow,
    Color::Green,
    Color::Blue,
    Color::DarkGray,
];
const LEVEL_MAP: [Level; 5] = [
    Level::Error,
    Level::Warn,
    Level::Info,
    Level::Debug,
    Level::Trace,
];

#[inline]
fn to_sys_level(l: LogLevel) -> Level {
    LEVEL_MAP[l as usize]
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

pub fn make_timestamp() -> String {
    let now = chrono::Local::now();
    let mut s = String::with_capacity(16);
    write!(s, "{}", now.format("%H:%M:%S%.3f")).unwrap();
    s
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
    let timestamp = make_timestamp();
    if let Ok(mut buffer) = UI_LOG_BUFFER.lock() {
        if buffer.len() == BUFFER_LIMIT {
            buffer.pop();
        }
        let obj = (timestamp, message.to_string(), to_sys_level(level));
        buffer.insert(0, obj);
    }
}

// Debug overlay widget
pub struct DebugOverlay {}

impl DebugOverlay {
    pub fn new() -> Self {
        Self {}
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
        let messages = {
            if let Ok(buffer) = UI_LOG_BUFFER.lock() {
                let take = inner_area.height as usize;
                let len = buffer.len();
                let start = len.saturating_sub(take);
                buffer[start..len].to_vec()
            } else {
                Vec::new()
            }
        };
        // Create text for log messages
        let mut text = Vec::with_capacity(inner_area.height as usize);
        for msg in messages.into_iter().rev() {
            let (timestamp, message, level) = msg;
            let log_level = LogLevel::from(level);
            let i = log_level as usize;
            let line = Line::from(vec![
                Span::styled(
                    format!("{} ", timestamp),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(LEVEL_STR[i], Style::default().fg(LEVEL_COLOR[i])),
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

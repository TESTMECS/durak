pub mod input;
pub mod render;
pub mod state;

// New modular structure replacing the monolithic logic.rs
mod ai_handler;
mod app_core;
mod game_actions;
mod game_loop;

// Re-export the App struct from app_core
pub use app_core::App;

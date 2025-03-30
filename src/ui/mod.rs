// Re-export necessary UI components and logging helpers
pub mod card_view;
pub mod debug_overlay;
pub mod game_ui;

// Logging helpers are not used outside, remove re-export
// pub use debug_overlay::{debug, info, warn, error, trace};

// GameUI and DebugOverlay are internal to the render module now.

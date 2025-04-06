# CLAUDE.md - Durak Game Dev Guide

## Build & Run Commands
- Build: `cargo build`
- Run: `cargo run`
- Release build: `cargo run --release`
- Debug with logging: `DURAK_DEBUG_FILE=durak_debug.log cargo run`
- Clippy lint: `cargo clippy`
- Format: `cargo fmt`
- Test with Clippy: `cargo clippy`

## Code Style
- Use Rust 2021 edition
- Follow standard Rust naming: snake_case for variables/functions, CamelCase for types
- Organize imports: std first, external crates alphabetically, then internal modules
- Document public functions with /// comments including descriptions, args, and returns
- Use Result<T, E> with anyhow for error handling, log errors before propagation
- Prefer pattern matching with match or if let over unwrap()/expect()
- Organize modules hierarchically (game/ui structure)
- Use strong typing with enums for state management
- Separate game logic from UI rendering
- Use log macros (info!, debug!, error!) for important state changes
## Code Quality
- Prefer simple solutions. 
- Avoid code duplication. Check if functionality can be reused from other files. 
- Try to keep code in smaller files around 200 to 300 lines. Files longer than this are ok as long as they are self contained. 
- Be a minimalist in your thinking. 
- Only implement features that are well-understood. 

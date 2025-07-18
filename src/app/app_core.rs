use super::render::render_ui;
use super::state::AppState;
use crate::ui::debug_overlay::{debug, error, info};
use crossterm::ExecutableCommand;
use crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};
use ratatui::Terminal;
use ratatui::backend::Backend;
use std::io::{self, stdout};

use crate::game::{AiDifficulty, AiPlayer, GamePhase, GameState, PlayerType};

pub struct App {
    pub game_state: GameState,
    pub app_state: AppState,
    pub selected_card_idx: Option<usize>,
    pub selected_cards: Vec<usize>,
    pub ai_player: AiPlayer,
    pub should_quit: bool,
    pub show_debug: bool,
    pub multiple_selection_mode: bool,
    pub selected_difficulty: AiDifficulty,
}

impl App {
    pub fn new() -> Self {
        info("Creating new App instance");
        let mut game_state = GameState::new();
        info("Adding human player");
        game_state.add_player("Player".to_string(), PlayerType::Human);
        info("Adding computer player");
        game_state.add_player("Computer".to_string(), PlayerType::Computer);
        Self {
            game_state,
            app_state: AppState::MainMenu,
            selected_card_idx: None,
            selected_cards: Vec::new(),
            ai_player: AiPlayer::new(AiDifficulty::Medium),
            should_quit: false,
            show_debug: false,
            multiple_selection_mode: false,
            selected_difficulty: AiDifficulty::Medium,
        }
    }
    /// Safely exits the game, restoring terminal state
    /// This should be called when encountering errors to ensure terminal is restored
    pub fn safe_exit(&mut self, error_msg: Option<&str>) -> io::Result<()> {
        self.should_quit = true;
        if let Some(msg) = error_msg {
            error(format!("Game error: {}", msg));
        }
        // Restore terminal to normal state
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        // If there was an error message, print it to stderr
        if let Some(msg) = error_msg {
            eprintln!("Error: {}", msg);
        }
        Ok(())
    }
    /// Show the debug overlay while in game (Press 'd' to toggle)
    pub fn toggle_debug(&mut self) {
        self.show_debug = !self.show_debug;
    }
    /// Quit the game and call `safe_exit`
    /// Print the error message if terminal restoration fails for debugging purposes.
    pub fn quit(&mut self) {
        // Use safe_exit without error message for normal exit
        if let Err(e) = self.safe_exit(None) {
            error(format!("Failed to restore terminal during quit: {}", e));
        }
    }
    /// Show rules on the main menu page.
    pub fn show_rules(&mut self) {
        self.app_state = AppState::RulesPage;
    }
    /// Backlink to the main menu from the menu pages.
    pub fn return_to_menu(&mut self) {
        self.app_state = AppState::MainMenu;
    }
    /// Show the difficulty select page.
    pub fn show_difficulty_select(&mut self) {
        self.app_state = AppState::DifficultySelect;
    }
    /// Changes the AI difficulty when selected from the difficulty select page.
    /// Return to the main menu afterwards.
    pub fn select_difficulty(&mut self, difficulty: AiDifficulty) {
        self.selected_difficulty = difficulty;
        self.ai_player = AiPlayer::new(difficulty);
        info(format!("AI difficulty changed to: {}", difficulty));
        self.app_state = AppState::MainMenu;
    }
    /// Toggles the multiple selection mode for the player.
    /// When enabled, the player can select multiple cards of the same rank.
    /// Should update the Controls UI with "ON"
    pub fn toggle_multiple_selection(&mut self) {
        self.multiple_selection_mode = !self.multiple_selection_mode;
        if !self.multiple_selection_mode {
            self.selected_card_idx = None;
        }
    }
    /// Toggles the selection of card in the player's hand by adding it to the `selected_cards` list.
    pub fn toggle_card_selection(&mut self, card_idx: usize) {
        if let Some(pos) = self.selected_cards.iter().position(|&idx| idx == card_idx) {
            self.selected_cards.remove(pos);
            debug(format!("Deselected card at index {}", card_idx));
        } else {
            self.selected_cards.push(card_idx);
            debug(format!("Selected card at index {}", card_idx));
        }
    }
    /// Get the current player index based on the game phase.
    pub fn current_player_index(&self) -> usize {
        match *self.game_state.game_phase() {
            GamePhase::Attack => self.game_state.current_attacker(),
            GamePhase::Defense => self.game_state.current_defender(),
            _ => self.game_state.current_attacker(),
        }
    }
    /// Calls the render UI method
    /// See `render.rs` for implementation.
    pub fn render<B: Backend>(&self, terminal: &mut Terminal<B>) -> io::Result<()> {
        terminal.draw(|f| render_ui(self, f))?;
        Ok(())
    }
}

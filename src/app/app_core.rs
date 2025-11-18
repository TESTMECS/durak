use super::render::render_ui;
use super::state::AppState;
use crate::game::{AiDifficulty, AiPlayer, GamePhase, GameState};
use crate::ui::debug_overlay::{debug, error, info};
use crossterm::ExecutableCommand;
use crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};
use ratatui::Terminal;
use ratatui::backend::Backend;
use std::io::{self, stdout};
pub struct App {
    pub game_state: GameState,
    pub app_state: AppState,
    pub selected_card_idx: Option<usize>,
    pub selected_cards: Vec<usize>,
    pub should_quit: bool,
    pub show_debug: bool,
    pub multiple_selection_mode: bool,
    pub ai_player: AiPlayer,
    pub selected_difficulty: AiDifficulty,
}
impl App {
    pub fn new() -> Self {
        info("Creating new App instance");
        Self {
            game_state: GameState::new(),
            app_state: AppState::MainMenu,
            selected_card_idx: None,
            selected_cards: Vec::new(),
            ai_player: AiPlayer::new(AiDifficulty::Medium),
            selected_difficulty: AiDifficulty::Medium,
            should_quit: false,
            show_debug: false,
            multiple_selection_mode: false,
        }
    }
    pub fn safe_exit(&mut self, error_msg: Option<&str>) -> io::Result<()> {
        self.should_quit = true;
        if let Some(msg) = error_msg {
            error(format!("Game error: {}", msg));
        }
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        if let Some(msg) = error_msg {
            eprintln!("Error: {}", msg);
        }
        Ok(())
    }
    pub fn toggle_debug(&mut self) {
        self.show_debug = !self.show_debug;
    }
    pub fn quit(&mut self) {
        if let Err(e) = self.safe_exit(None) {
            error(format!("Failed to restore terminal during quit: {}", e));
        }
    }
    #[inline]
    pub fn show_rules(&mut self) {
        self.app_state = AppState::RulesPage;
    }
    #[inline]
    pub fn return_to_menu(&mut self) {
        self.app_state = AppState::MainMenu;
    }
    #[inline]
    pub fn show_difficulty_select(&mut self) {
        self.app_state = AppState::DifficultySelect;
    }
    pub fn select_difficulty(&mut self, difficulty: AiDifficulty) {
        self.selected_difficulty = difficulty;
        self.ai_player = AiPlayer::new(difficulty);
        info(format!("AI difficulty changed to: {}", difficulty));
        self.app_state = AppState::MainMenu;
    }
    pub fn toggle_multiple_selection(&mut self) {
        self.multiple_selection_mode = !self.multiple_selection_mode;
        if !self.multiple_selection_mode {
            self.selected_card_idx = None;
        }
    }
    pub fn toggle_card_selection(&mut self, card_idx: usize) {
        if let Some(pos) = self.selected_cards.iter().position(|&idx| idx == card_idx) {
            self.selected_cards.remove(pos);
            debug(format!("Deselected card at index {}", card_idx));
        } else {
            self.selected_cards.push(card_idx);
            debug(format!("Selected card at index {}", card_idx));
        }
    }
    pub fn current_player_index(&self) -> usize {
        match *self.game_state.game_phase() {
            GamePhase::Attack => self.game_state.current_attacker(),
            GamePhase::Defense => self.game_state.current_defender(),
            _ => self.game_state.current_attacker(),
        }
    }
    pub fn render<B: Backend>(&self, terminal: &mut Terminal<B>) -> io::Result<()> {
        terminal.draw(|f| render_ui(self, f))?;
        Ok(())
    }
}

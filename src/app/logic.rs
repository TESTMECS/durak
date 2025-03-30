use crossterm::event::{self, Event, KeyCode, KeyEventKind};
// Import log crate macros
use ratatui::backend::Backend;
use ratatui::Terminal;
use std::io;

// Use super to access sibling modules
use super::input::{handle_key_input, AppAction};
use super::render::render_ui;
use super::state::AppState;
// Remove conflicting import of functions from debug_overlay
// use crate::ui::debug_overlay::{debug, error, info, trace, warn};

use crate::game::{AiDifficulty, AiPlayer, GamePhase, GameState, PlayerType};

// Make the struct public
pub struct App {
    pub game_state: GameState,
    pub app_state: AppState,
    pub selected_card_idx: Option<usize>,
    pub selected_cards: Vec<usize>,
    pub ai_player: AiPlayer,
    pub should_quit: bool,
    pub show_debug: bool,
    pub multiple_selection_mode: bool,
}

// Implementation remains the same, just moved here
impl App {
    pub fn new() -> Self {
        //info!("Creating new App instance");
        let mut game_state = GameState::new();

        // Add a human player
        game_state.add_player("Player".to_string(), PlayerType::Human);

        // Add an AI opponent
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
        }
    }

    pub fn toggle_debug(&mut self) {
        self.show_debug = !self.show_debug;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    #[allow(dead_code)]
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn show_rules(&mut self) {
        self.app_state = AppState::RulesPage;
    }

    pub fn return_to_menu(&mut self) {
        self.app_state = AppState::MainMenu;
    }

    pub fn toggle_multiple_selection(&mut self) {
        self.multiple_selection_mode = !self.multiple_selection_mode;
        self.selected_cards.clear();
        if !self.multiple_selection_mode {
            self.selected_card_idx = None;
        }
    }

    pub fn toggle_card_selection(&mut self, card_idx: usize) {
        if let Some(pos) = self.selected_cards.iter().position(|&idx| idx == card_idx) {
            self.selected_cards.remove(pos);
            // debug!("Deselected card at index {}", card_idx);
        } else {
            self.selected_cards.push(card_idx);
            // debug!("Selected card at index {}", card_idx);
        }
    }

    pub fn process_ai_turn(&mut self) {
        if *self.game_state.game_phase() == GamePhase::GameOver
            || self.app_state == AppState::GameOver
        {
            return; // Don't process if already over
        }
        if !matches!(
            *self.game_state.game_phase(),
            GamePhase::Attack | GamePhase::Defense
        ) {
            return;
        }

        // debug!("Starting AI turn processing loop");

        loop {
            let current_player_idx = self.current_player_index();
            let is_ai_turn = self.game_state.players()[current_player_idx].player_type()
                == &PlayerType::Computer;

            // Check if game is over *before* AI acts
            if self.game_state.check_game_over() {
                self.app_state = AppState::GameOver;
                break;
            }

            if !is_ai_turn {
                // debug!("Not AI turn or phase changed, stopping AI loop");
                break;
            }

            // AI makes its move
            let mut _ai_action_taken = false; // Prefix with underscore as value is not read
            match *self.game_state.game_phase() {
                GamePhase::Attack => {
                    let attack_cards = self
                        .ai_player
                        .make_multi_attack_move(&self.game_state, current_player_idx);
                    if !attack_cards.is_empty() {
                        let mut sorted_indices = attack_cards.clone();
                        sorted_indices.sort_by(|a, b| b.cmp(a));
                        let mut all_successful = true;
                        for &idx in sorted_indices.iter().rev() {
                            let result = self.game_state.attack(idx);
                            if let Err(_e) = result {
                                all_successful = false;
                                break;
                            }
                        }
                        if all_successful {
                            // debug!("AI attack successful");
                            _ai_action_taken = true; // Assign to prefixed variable
                        } else {
                            // debug!("AI attack failed, breaking loop");
                            break; // Prevent infinite loop on failed attack
                        }
                    } else {
                        self.game_state.draw_cards(); // Pass involves drawing
                        _ai_action_taken = true; // Assign to prefixed variable
                    }
                }
                GamePhase::Defense => {
                    if self
                        .ai_player
                        .should_take_cards(&self.game_state, current_player_idx)
                    {
                        if let Err(_e) = self.game_state.take_cards() {
                            break;
                        }
                        self.game_state.draw_cards(); // Taking involves drawing
                        _ai_action_taken = true; // Assign to prefixed variable
                    } else {
                        let mut defense_possible = true;
                        while self
                            .game_state
                            .table_cards()
                            .iter()
                            .any(|(_, d)| d.is_none())
                        {
                            // While undefended cards exist
                            if let Some(card_idx) = self
                                .ai_player
                                .make_defense_move(&self.game_state, current_player_idx)
                            {
                                if let Err(_e) = self.game_state.defend(card_idx) {
                                    defense_possible = false;
                                    break;
                                }
                            } else {
                                // AI couldn't find a card to defend the current attack
                                // debug!("AI cannot defend further, attempting to take cards");
                                defense_possible = false;
                                break;
                            }
                        }

                        if !defense_possible {
                            if let Err(_) = self.game_state.take_cards() {
                                break; // Critical error, stop AI
                            }
                            self.game_state.draw_cards();
                        } else {
                            // debug!("AI successfully defended all attacks");
                            // If defense was successful, the turn ends, and cards are implicitly discarded
                            // before the draw phase begins in the next cycle (or handled by draw_cards).
                        }
                        _ai_action_taken = true; // Assign to prefixed variable
                    }
                }
                _ => {
                    break;
                } // Should not happen based on initial check
            }

            // Check game over *after* AI action
            if self.game_state.check_game_over() {
                self.app_state = AppState::GameOver;
                break;
            }

            // If AI didn't pass/take/fail, and it's still their turn (e.g., successful defense), continue loop
            // If AI passed/took/failed, or turn changed, break the loop
            let next_player_idx = self.current_player_index(); // Recalculate after potential state change
            let next_is_ai_turn =
                self.game_state.players()[next_player_idx].player_type() == &PlayerType::Computer;
            let phase = self.game_state.game_phase().clone(); // Capture current phase

            if _ai_action_taken
                && (phase == GamePhase::Attack || phase == GamePhase::Defense)
                && next_is_ai_turn
            {
                // debug!("AI action complete, continuing AI turn in phase {:?}", phase);
            } else {
                // debug!("AI action ({}) resulted in phase {:?} or different player ({}), breaking AI loop", _ai_action_taken, phase, next_player_idx); // Read prefixed variable
                break;
            }
        }
        // debug!("AI processing complete");
    }

    fn current_player_index(&self) -> usize {
        match *self.game_state.game_phase() {
            GamePhase::Attack => self.game_state.current_attacker(),
            GamePhase::Defense => self.game_state.current_defender(),
            _ => self.game_state.current_attacker(),
        }
    }

    pub fn on_key(&mut self, key: KeyCode) {
        // Trace the key press, current state, and game phase
        // trace!("Key: {:?}, State: {:?}, Phase: {:?}", key, self.app_state, self.game_state.game_phase());

        if let Some(action) = handle_key_input(&self.app_state, self.game_state.game_phase(), key) {
            self.process_action(action);
        } else {
            // trace!("No action mapped for key");
        }
    }

    fn process_action(&mut self, action: AppAction) {
        // debug!("Processing action: {:?}", action);
        match action {
            AppAction::Quit => self.quit(),
            AppAction::ToggleDebug => self.toggle_debug(),
            AppAction::ShowRules => self.show_rules(),
            AppAction::ReturnToMenu => self.return_to_menu(),
            AppAction::SelectNextCard => self.select_next_card(),
            AppAction::SelectPrevCard => self.select_prev_card(),
            AppAction::ToggleMultiSelect => self.toggle_multiple_selection(),
            AppAction::ToggleCardSelection => {
                if self.multiple_selection_mode {
                    if let Some(idx) = self.selected_card_idx {
                        self.toggle_card_selection(idx);
                    }
                }
            }
            AppAction::StartGame => self.start_game_action(),
            AppAction::PlaySelectedCard => self.play_card_action(),
            AppAction::PassTurn => self.pass_turn_action(),
            AppAction::TakeCards => self.take_cards_action(),
            AppAction::StartNewGame => self.start_game_action(),
            AppAction::AcknowledgeDraw => self.acknowledge_draw_action(),
        }
    }

    fn start_game_action(&mut self) {
        self.game_state.setup_game();
        self.app_state = AppState::Playing;
        self.selected_card_idx = None;
        self.selected_cards.clear();
        self.multiple_selection_mode = false;
        self.check_game_over_and_process_ai(); // Check AI turn immediately after setup
    }

    fn select_next_card(&mut self) {
        if let Some(player) = self.game_state.players().get(self.current_player_index()) {
            if player.player_type() == &PlayerType::Human {
                let hand_size = player.hand_size();
                if hand_size > 0 {
                    let _old_idx = self.selected_card_idx;
                    self.selected_card_idx = match self.selected_card_idx {
                        Some(idx) if idx < hand_size - 1 => Some(idx + 1),
                        None => Some(0),
                        Some(_) => Some(0), // Wrap around
                    };
                    // debug!("Select next: {:?} -> {:?}", old_idx, self.selected_card_idx);
                }
            }
        }
    }

    fn select_prev_card(&mut self) {
        if let Some(player) = self.game_state.players().get(self.current_player_index()) {
            if player.player_type() == &PlayerType::Human {
                let hand_size = player.hand_size();
                if hand_size > 0 {
                    let _old_idx = self.selected_card_idx;
                    self.selected_card_idx = match self.selected_card_idx {
                        Some(idx) if idx > 0 => Some(idx - 1),
                        None => Some(hand_size - 1), // Wrap around
                        Some(_) => Some(hand_size - 1),
                    };
                    // debug!("Select prev: {:?} -> {:?}", old_idx, self.selected_card_idx);
                }
            }
        }
    }

    fn play_card_action(&mut self) {
        let current_player_idx = self.current_player_index();
        if self.game_state.players()[current_player_idx].player_type() == &PlayerType::Human {
            match *self.game_state.game_phase() {
                GamePhase::Attack => self.handle_attack_phase(current_player_idx),
                GamePhase::Defense => self.handle_defense_phase(current_player_idx),
                _ => {
                    // debug!("Ignoring PlayCard action outside of attack or defense phase");
                    return;
                }
            }
        } else {
            // debug!("Ignoring PlayCard action during AI turn");
            return;
        }
    }

    fn pass_turn_action(&mut self) {
        let player_idx = self.current_player_index();
        if *self.game_state.game_phase() == GamePhase::Attack
            && self.game_state.players()[player_idx].player_type() == &PlayerType::Human
        {
            self.game_state.draw_cards();
            self.check_game_over_and_process_ai();
        } else {
            // debug!("Ignore PassTurn");
            return; // Ignore pass if it's not the human player's turn
        }
    }

    fn take_cards_action(&mut self) {
        let player_idx = self.current_player_index();
        if *self.game_state.game_phase() == GamePhase::Defense
            && self.game_state.players()[player_idx].player_type() == &PlayerType::Human
        {
            if let Err(_e) = self.game_state.take_cards() {
                //error!("Human take cards failed: {}", e);
                //warn!("Unable to take cards");
                self.check_game_over_and_process_ai();
            } else {
                self.game_state.draw_cards();
                self.check_game_over_and_process_ai();
            }
        } else {
            // debug!("Ignore TakeCards");
            return;
        }
    }

    fn acknowledge_draw_action(&mut self) {
        // debug!("Acknowledging draw phase");
        self.game_state.draw_cards();
        if *self.game_state.game_phase() == GamePhase::Drawing {
            //warn!("Drawing phase stuck, forcing Attack");
            self.game_state = GameState::force_attack_phase(self.game_state.clone());
        }
        self.check_game_over_and_process_ai();
    }

    fn check_game_over_and_process_ai(&mut self) {
        // Check game over BEFORE processing AI
        if self.game_state.check_game_over() {
            self.app_state = AppState::GameOver;
            return;
        }
        self.process_ai_turn();
        if self.game_state.check_game_over() {
            //info!("Game over detected after AI processing");
            self.app_state = AppState::GameOver;
        }
    }

    fn handle_attack_phase(&mut self, player_idx: usize) {
        let mut cards_played = false;
        if self.multiple_selection_mode && !self.selected_cards.is_empty() {
            match self.play_selected_cards(player_idx) {
                Ok(_) => {
                    cards_played = true;
                    // debug!("Multi-attack OK");
                }
                Err(_) => {
                    //warn!("{}", &e);
                }
            }
            self.selected_cards.clear();
        } else if let Some(idx) = self.selected_card_idx {
            match self.game_state.attack(idx) {
                Ok(_) => {
                    cards_played = true;
                    // debug!("Single attack OK");
                }
                Err(_) => {
                    //warn!("Single attack failed: {}", e);
                }
            }
        } else {
            // debug!("Attack action with no card selected");
            //warn!("Attack attempted without selecting a card.");
        }
        if cards_played {
            self.selected_card_idx = None;
            self.check_game_over_and_process_ai();
        }
    }

    fn handle_defense_phase(&mut self, _player_idx: usize) {
        let mut card_played = false;
        if let Some(idx) = self.selected_card_idx {
            match self.game_state.defend(idx) {
                Ok(_) => {
                    card_played = true;
                    // debug!("Defense OK");
                }
                Err(_e) => {}
            }
        }
        if card_played {
            self.selected_card_idx = None;
            self.check_game_over_and_process_ai();
        }
    }

    fn play_selected_cards(&mut self, player_idx: usize) -> Result<(), String> {
        let _hand_size_before = self.game_state.players()[player_idx].hand_size();
        if self.selected_cards.is_empty() {
            return Err("No cards selected".to_string());
        }

        let mut sorted_indexes = self.selected_cards.clone();
        sorted_indexes.sort_by(|a, b| b.cmp(a)); // Sort descending for safe removal

        // Validate ranks if multiple cards
        if sorted_indexes.len() > 1 {
            let hand = self.game_state.players()[player_idx].hand();
            if sorted_indexes.iter().any(|&idx| idx >= hand.len()) {
                return Err("Invalid card index in selection".to_string());
            }
            let first_rank = hand[sorted_indexes[0]].rank;
            if !sorted_indexes
                .iter()
                .all(|&idx| hand[idx].rank == first_rank)
            {
                return Err("Cannot attack with multiple cards of different ranks".to_string());
            }
            // Validate against defender hand size
            let defender_hand_size =
                self.game_state.players()[self.game_state.current_defender()].hand_size();
            if sorted_indexes.len() > defender_hand_size {
                return Err(format!(
                    "Cannot attack with {} cards, defender only has {}",
                    sorted_indexes.len(),
                    defender_hand_size
                ));
            }
        }

        // Attempt to play cards (indices are sorted descending)
        for &idx in &sorted_indexes {
            // Iterate over original sorted indices
            // Need to adjust index based on cards already removed IF we removed ascending
            // Since we sort descending, the index remains correct relative to the *current* hand
            // But GameState::attack expects index relative to hand *before* any removal in this turn.
            // It's simpler to pass the cards themselves or rethink index handling.
            // For now, let's assume GameState::attack handles indices correctly based on its internal state after each attack.

            // THIS IS WRONG - attack needs the index from the ORIGINAL hand state.
            // We need to rethink how multi-attack interfaces with GameState.attack
            // Option 1: GameState::attack_multiple(Vec<usize>)
            // Option 2: Pass cards, not indices
            // Option 3: App removes cards first, then tells GameState about them (but validation needs GameState)

            // Let's try a temporary fix: Attack with highest index first.
            let current_hand_idx = idx; // This is only correct for the FIRST card played this way.

            // TODO: Fix multi-card attack index handling!
            match self.game_state.attack(current_hand_idx) {
                Ok(_) => continue,
                Err(e) => {
                    // Important: If one card fails, we need to potentially revert previous attacks in this batch!
                    // This simple loop doesn't handle reverts. Multi-attack needs atomic handling in GameState.
                    return Err(format!(
                        "Attack failed for card index {}: {}",
                        current_hand_idx, e
                    ));
                }
            }
        }

        // If all attacks succeeded (or loop didn't run for single card)
        Ok(())
    }

    pub fn render<B: Backend>(&self, terminal: &mut Terminal<B>) -> io::Result<()> {
        terminal.draw(|f| render_ui(self, f))?;
        Ok(())
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        while !self.should_quit {
            self.render(terminal)?;
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.on_key(key.code);
                    }
                }
            }
        }
        Ok(())
    }
}

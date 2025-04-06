/*
 * logic.rs - Main game logic for the Durak card game
 *
 * This file contains the App struct which manages the game state and user interface.
 * It handles:
 * - User input processing and actions
 * - Game state transitions
 * - AI decision making
 * - Card selection and gameplay mechanics
 *
 * The game flow is event-driven, with each user action triggering appropriate state changes
 * and AI responses when needed.
 */

use super::input::{handle_key_input, AppAction};
use super::render::render_ui;
use super::state::AppState;
use crate::ui::debug_overlay::{debug, info, trace};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::backend::Backend;
use ratatui::Terminal;
use std::io;

use crate::game::card::Card;
use crate::game::{AiDifficulty, AiPlayer, GamePhase, GameState, PlayerType};

/// Main application struct that contains all game state and UI state
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
        let ai_difficulty = AiDifficulty::Medium;
        info(format!("Setting initial AI difficulty to: {}", ai_difficulty));

        Self {
            game_state,
            app_state: AppState::MainMenu,
            selected_card_idx: None,
            selected_cards: Vec::new(),
            ai_player: AiPlayer::new(ai_difficulty),
            should_quit: false,
            show_debug: false,
            multiple_selection_mode: false,
            selected_difficulty: AiDifficulty::Medium,
        }
    }

    pub fn toggle_debug(&mut self) {
        self.show_debug = !self.show_debug;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn show_rules(&mut self) {
        self.app_state = AppState::RulesPage;
    }

    pub fn return_to_menu(&mut self) {
        self.app_state = AppState::MainMenu;
    }
    
    pub fn show_difficulty_select(&mut self) {
        self.app_state = AppState::DifficultySelect;
    }
    
    pub fn select_difficulty(&mut self, difficulty: AiDifficulty) {
        self.selected_difficulty = difficulty;
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
        // adds to the players attacking cards. self.selected_cards
        if let Some(pos) = self.selected_cards.iter().position(|&idx| idx == card_idx) {
            self.selected_cards.remove(pos);
            debug(format!("Deselected card at index {}", card_idx));
        } else {
            self.selected_cards.push(card_idx);
            debug(format!("Selected card at index {}", card_idx));
        }
    }

    fn current_player_index(&self) -> usize {
        // Get the current active player index based on game phase
        match *self.game_state.game_phase() {
            GamePhase::Attack => self.game_state.current_attacker(),
            GamePhase::Defense => self.game_state.current_defender(),
            _ => self.game_state.current_attacker(),
        }
    }

    pub fn process_ai_turn(&mut self) {
        // Use a counter to prevent potential infinite loops
        let mut turn_counter = 0;
        const MAX_TURNS: i32 = 10; // Safety limit for iterations

        while turn_counter < MAX_TURNS {
            turn_counter += 1;
            debug(format!("AI turn iteration {}", turn_counter));

            // Check for game over - this also sets the winner
            if self.game_state.check_game_over() {
                self.app_state = AppState::GameOver;
                return;
            }

            // Get the current player based on game phase
            let current_player_idx = self.current_player_index();

            // Check if it's AI's turn
            let is_ai_turn = self.game_state.players()[current_player_idx].player_type()
                == &PlayerType::Computer;

            if !is_ai_turn {
                debug("Not AI's turn, ending AI processing");
                return; // Not AI's turn
            }

            debug(format!(
                "AI playing in phase: {:?}",
                self.game_state.game_phase()
            ));

            // Log current game state for debugging
            debug(format!(
                "Current attacker: {}, Current defender: {}",
                self.game_state.current_attacker(),
                self.game_state.current_defender()
            ));

            // Handle based on game phase
            match *self.game_state.game_phase() {
                GamePhase::Attack => {
                    // AI decides to attack or pass
                    debug("AI attempting to attack");
                    let attack_result = self.handle_ai_attack(current_player_idx);
                    debug(format!("AI attack result: {:?}", attack_result));

                    // Check if we successfully transitioned to Defense phase
                    if *self.game_state.game_phase() == GamePhase::Defense {
                        // We're now in defense phase
                        let defender_idx = self.game_state.current_defender();
                        let is_human_defender = self.game_state.players()[defender_idx]
                            .player_type()
                            == &PlayerType::Human;

                        if is_human_defender {
                            debug("Human needs to defend, ending AI processing");
                            return;
                        } else {
                            // AI needs to defend against itself - this should be rare but possible
                            debug("AI needs to defend against itself, continuing");
                            continue; // Process the defense in the next iteration
                        }
                    } else if *self.game_state.game_phase() == GamePhase::Attack {
                        // If still in Attack phase, AI passed - move to Drawing
                        debug("AI passed attack, transitioning to drawing phase");
                        self.game_state.draw_cards();
                    }
                }
                GamePhase::Defense => {
                    // AI decides to defend or take
                    debug("AI attempting to defend");
                    let defense_result = self.handle_ai_defense(current_player_idx);
                    debug(format!("AI defense result: {:?}", defense_result));

                    // After AI defense, check the current state
                    // First, double-check if we're still in Defense phase but roles have changed
                    if *self.game_state.game_phase() == GamePhase::Defense {
                        // Check if the AI is still the defender
                        let current_defender = self.game_state.current_defender();
                        let is_ai_defender = self.game_state.players()[current_defender].player_type() 
                            == &PlayerType::Computer;
                        
                        // Different defender means a pass occurred
                        if current_defender != current_player_idx {
                            debug("Pass occurred, roles have changed");
                            
                            if is_ai_defender {
                                // If AI is still the defender (AI passed to AI), continue processing
                                debug("AI passed to AI, continuing defense");
                                continue;
                            } else {
                                // AI passed to human, end AI processing
                                debug("AI passed to human, ending AI processing");
                                return;
                            }
                        } else {
                            // Still the same defender, transition to Drawing
                            debug("AI defense incomplete, forcing draw phase");
                            self.game_state.draw_cards();
                        }
                    } else if *self.game_state.game_phase() == GamePhase::Drawing {
                        // Already in drawing phase, just draw
                        debug("AI defense complete, already in drawing phase");
                        self.game_state.draw_cards();
                    }
                }
                GamePhase::Drawing => {
                    // Drawing phase logic
                    debug("AI processing drawing phase");
                    self.game_state.draw_cards();

                    // After drawing, check the new phase and player
                    if *self.game_state.game_phase() == GamePhase::Attack {
                        let attacker_idx = self.game_state.current_attacker();
                        let is_human_attacker = self.game_state.players()[attacker_idx]
                            .player_type()
                            == &PlayerType::Human;

                        if is_human_attacker {
                            debug("Human's turn after drawing, ending AI processing");
                            return;
                        } else {
                            debug(
                                "AI's turn to attack after drawing, continuing to next iteration",
                            );
                            continue; // Process the attack in the next iteration
                        }
                    }
                }
                _ => {
                    debug("AI turn in unhandled game phase, ending AI processing");
                    return;
                }
            }

            // Safety check for phase transitions
            if *self.game_state.game_phase() == GamePhase::Drawing {
                debug("Handling drawing phase transition");
                self.game_state.draw_cards(); // Process the draw

                // Check if we successfully moved to Attack phase
                if *self.game_state.game_phase() == GamePhase::Drawing {
                    // If still in drawing phase, force to attack phase
                    debug("Forcing transition from Drawing to Attack phase");
                    self.game_state = GameState::force_attack_phase(self.game_state.clone());
                }

                // Check if it's now a human player's turn
                if *self.game_state.game_phase() == GamePhase::Attack {
                    let attacker_idx = self.game_state.current_attacker();
                    let is_human_attacker =
                        self.game_state.players()[attacker_idx].player_type() == &PlayerType::Human;

                    if is_human_attacker {
                        debug("Human's turn after drawing phase completed, ending AI processing");
                        return;
                    }
                }
            }

            // Check for game over after each action - this also sets the winner
            if self.game_state.check_game_over() {
                self.app_state = AppState::GameOver;
                return;
            }

            // If we've reached the iteration limit, force a return to prevent issues
            if turn_counter >= MAX_TURNS - 1 {
                debug("Reached maximum AI turn iterations, forcing end to prevent issues");
                if *self.game_state.game_phase() == GamePhase::Drawing {
                    self.game_state = GameState::force_attack_phase(self.game_state.clone());
                }
                return;
            }
        }
    }

    fn handle_ai_attack(&mut self, player_idx: usize) -> Result<(), String> {
        debug("AI is attacking");

        // Verify we're in the correct phase
        if *self.game_state.game_phase() != GamePhase::Attack {
            debug("AI called to attack but not in attack phase");
            return Ok(());
        }

        // Validate the current attacker
        if self.game_state.current_attacker() != player_idx {
            debug(format!(
                "Wrong player attacking: expected {}, got {}",
                self.game_state.current_attacker(),
                player_idx
            ));
            return Ok(());
        }

        // Get attack moves from AI
        let attack_cards = self
            .ai_player
            .make_attack_move(&self.game_state, player_idx);

        if let Some(cards) = attack_cards {
            if cards.is_empty() {
                debug("AI decided to pass");
                return Ok(()); // AI passes
            }

            // Sort and make attacks (highest index first to prevent shifting)
            let mut sorted_indices: Vec<usize> = cards.iter().map(|(idx, _)| *idx).collect();
            sorted_indices.sort_by(|a, b| b.cmp(a));

            let mut attack_successful = false;

            for &idx in sorted_indices.iter() {
                match self.game_state.attack(idx, player_idx) {
                    Ok(_) => {
                        attack_successful = true;
                        debug(format!("AI successfully attacked with card {}", idx));
                    }
                    Err(e) => {
                        debug(format!("AI attack failed: {}", e));
                        return Err(e.to_string());
                    }
                }
            }

            if attack_successful {
                debug(format!(
                    "AI successfully attacked with {} cards",
                    sorted_indices.len()
                ));

                // Verify we've transitioned to defense phase
                if *self.game_state.game_phase() != GamePhase::Defense {
                    debug(
                        "Warning: Game did not transition to Defense phase after successful attack",
                    );
                    let defender_idx = (player_idx + 1) % self.game_state.players().len();
                    self.game_state
                        .set_phase_to_defense(player_idx, defender_idx);
                }
            }
        } else {
            debug("AI decided to pass (no attacks)");
        }

        Ok(())
    }

    fn handle_ai_defense(&mut self, player_idx: usize) -> Result<(), String> {
        debug("AI is defending");

        // Check if the game state is valid for defense
        if *self.game_state.game_phase() != GamePhase::Defense {
            debug("AI called to defend but not in defense phase");
            return Ok(());
        }

        // Verify the correct player is defending
        if self.game_state.current_defender() != player_idx {
            debug(format!(
                "Wrong player defending: expected {}, got {}",
                self.game_state.current_defender(),
                player_idx
            ));
            return Ok(());
        }

        // Check if AI should take cards instead of defending
        if self
            .ai_player
            .should_take_cards(&self.game_state, player_idx)
        {
            debug("AI decided to take cards");
            if let Err(e) = self.game_state.take_cards() {
                return Err(e.to_string());
            }
            return Ok(());
        }

        // Get the table state
        let table_cards = self.game_state.table_cards();
        if table_cards.is_empty() {
            debug("No cards to defend against");
            return Ok(());
        }

        // Check for undefended attacks
        let has_undefended = table_cards.iter().any(|(_, defense)| defense.is_none());
        if !has_undefended {
            debug("All attacks already defended");
            return Ok(());
        }

        // Try to defend each undefended attack one at a time
        let mut defense_failed = false;

        while !defense_failed
            && self
                .game_state
                .table_cards()
                .iter()
                .any(|(_, d)| d.is_none())
        {
            if let Some(defense_cards) = self
                .ai_player
                .make_defense_move(&self.game_state, player_idx)
            {
                debug(format!("AI defending with cards: {:?}", defense_cards));

                // Process each defense card
                for (_table_idx, card) in &defense_cards {
                    // We need to find the hand index of this card
                    if let Some(hand_idx) = self.find_card_index_in_hand(player_idx, *card) {
                        match self.game_state.defend(hand_idx) {
                            Ok(_) => {
                                debug(format!("AI successfully defended with card {}", hand_idx));

                                // Check if a pass occurred by looking at the defender change
                                if self.game_state.current_defender() != player_idx {
                                    debug("AI passed the card to a different player");
                                    
                                    // A pass occurred, we're done with this defense turn
                                    // The next player will need to handle these cards
                                    return Ok(());
                                }

                                // Check if all cards are defended now
                                let all_defended = !self
                                    .game_state
                                    .table_cards()
                                    .iter()
                                    .any(|(_, defense)| defense.is_none());

                                if all_defended {
                                    debug("AI successfully defended all attacks");

                                    // Get all cards from the table for discarding
                                    let cards_to_discard: Vec<(usize, Card)> = self
                                        .game_state
                                        .table_cards()
                                        .iter()
                                        .enumerate()
                                        .filter_map(|(idx, (_, defense))| {
                                            defense.map(|card| (idx, card))
                                        })
                                        .collect();

                                    // Discard the cards
                                    self.game_state.discard_cards(cards_to_discard);
                                    return Ok(());
                                }
                            }
                            Err(e) => {
                                debug(format!("AI defense failed: {}", e));
                                defense_failed = true;
                                break;
                            }
                        }
                    } else {
                        debug("Card not found in AI's hand");
                        defense_failed = true;
                        break;
                    }
                }
            } else {
                debug("AI cannot defend further");
                defense_failed = true;
            }
        }

        // If AI couldn't defend everything, take cards
        if defense_failed
            || self
                .game_state
                .table_cards()
                .iter()
                .any(|(_, d)| d.is_none())
        {
            debug("AI taking cards after failed defense");
            if let Err(e) = self.game_state.take_cards() {
                return Err(e.to_string());
            }
        }

        Ok(())
    }

    // Helper method to find a card's index in a player's hand
    fn find_card_index_in_hand(&self, player_idx: usize, card: Card) -> Option<usize> {
        let player = &self.game_state.players()[player_idx];
        player
            .hand()
            .iter()
            .enumerate()
            .find(|(_, &c)| c == card)
            .map(|(idx, _)| idx)
    }

    pub fn on_key(&mut self, key: KeyCode) {
        trace(format!(
            "Key: {:?}, State: {:?}, Phase: {:?}",
            key,
            self.app_state,
            self.game_state.game_phase(),
        ));
        if let Some(action) = handle_key_input(&self.app_state, self.game_state.game_phase(), key) {
            self.process_action(action);
        } else {
            trace("No action mapped for key");
        }
    }

    fn process_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => self.quit(),
            AppAction::ToggleDebug => self.toggle_debug(),
            AppAction::ShowRules => self.show_rules(),
            AppAction::ShowDifficultySelect => self.show_difficulty_select(),
            AppAction::SelectEasyDifficulty => self.select_difficulty(AiDifficulty::Easy),
            AppAction::SelectMediumDifficulty => self.select_difficulty(AiDifficulty::Medium),
            AppAction::SelectHardDifficulty => self.select_difficulty(AiDifficulty::Hard),
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
            AppAction::StartNewGame => self.start_game_action(), // Restart current game
            AppAction::AcknowledgeDraw => self.acknowledge_draw_action(),
        }
    }

    fn start_game_action(&mut self) {
        self.app_state = AppState::Playing;
        self.game_state.setup_game();
        // clear cards just in case
        self.selected_card_idx = None;
        self.selected_cards.clear();
        self.multiple_selection_mode = false;
        
        // Create a new AI player with the selected difficulty
        self.ai_player = AiPlayer::new(self.selected_difficulty);
        debug(format!("Starting game with AI difficulty: {}", self.selected_difficulty));

        // Log the AI difficulty level characteristics
        match self.selected_difficulty {
            AiDifficulty::Easy => {
                debug("Easy AI: Will play lowest cards, often take cards instead of defending");
            },
            AiDifficulty::Medium => {
                debug("Medium AI: Will use basic strategy, manage trumps, and sometimes pass cards");
            },
            AiDifficulty::Hard => {
                debug("Hard AI: Will strategically track cards, exploit weaknesses, and plan ahead");
            }
        }

        debug("Game started!");

        // Process AI turn if AI goes first
        let current_player_idx = self.current_player_index();
        let is_ai_turn =
            self.game_state.players()[current_player_idx].player_type() == &PlayerType::Computer;

        if is_ai_turn {
            debug("AI goes first");
            self.process_ai_turn();
        }
    }

    fn select_next_card(&mut self) {
        if let Some(player) = self.game_state.players().get(self.current_player_index()) {
            if player.player_type() == &PlayerType::Human {
                let hand_size = player.hand_size();
                if hand_size > 0 {
                    let old_idx = self.selected_card_idx;
                    self.selected_card_idx = match self.selected_card_idx {
                        Some(idx) if idx < hand_size - 1 => Some(idx + 1),
                        None => Some(0),
                        Some(_) => Some(0), // Wrap around
                    };
                    debug(format!(
                        "Select next: {:?} -> {:?}",
                        old_idx, self.selected_card_idx
                    ));
                }
            }
        }
    }

    fn select_prev_card(&mut self) {
        if let Some(player) = self.game_state.players().get(self.current_player_index()) {
            if player.player_type() == &PlayerType::Human {
                let hand_size = player.hand_size();
                if hand_size > 0 {
                    let old_idx = self.selected_card_idx;
                    self.selected_card_idx = match self.selected_card_idx {
                        Some(idx) if idx > 0 => Some(idx - 1),
                        None => Some(hand_size - 1), // Wrap around
                        Some(_) => Some(hand_size - 1),
                    };
                    debug(format!(
                        "Select prev: {:?} -> {:?}",
                        old_idx, self.selected_card_idx
                    ));
                }
            }
        }
    }

    fn play_card_action(&mut self) {
        // Main entry point for Player attack and defense key options.
        let current_player_idx = self.current_player_index();
        if self.game_state.players()[current_player_idx].player_type() == &PlayerType::Human {
            match *self.game_state.game_phase() {
                GamePhase::Attack => {
                    if let Err(e) = self.handle_attack_phase(current_player_idx) {
                        debug(format!("Attack failed: {}", e));
                        return;
                    }
                    // If successful attack, game will transition to Defense phase
                    // Process AI's turn if they are the defender
                    self.process_ai_turn();
                }
                GamePhase::Defense => {
                    if let Err(e) = self.handle_defense_phase(current_player_idx) {
                        debug(format!("Defense failed: {}", e));
                        return;
                    }

                    // After defense, check game state
                    if *self.game_state.game_phase() == GamePhase::Drawing {
                        // If drawing phase, proceed with drawing
                        self.game_state.draw_cards();
                        // After drawing, process AI's turn if they are next
                        self.process_ai_turn();
                    } else if *self.game_state.game_phase() == GamePhase::Defense {
                        // Check if a different player is now defending (pass occurred)
                        let current_defender = self.game_state.current_defender();
                        if current_defender != current_player_idx {
                            debug("Detected pass - different player now defending");
                            
                            // Check if AI is now the defender
                            let is_ai_defender = self.game_state.players()[current_defender].player_type() 
                                == &PlayerType::Computer;
                            
                            if is_ai_defender {
                                debug("AI is now defending after pass, processing AI turn");
                                self.process_ai_turn();
                            }
                        }
                    }
                }
                _ => {
                    return;
                }
            }
        }
    }

    fn pass_turn_action(&mut self) {
        let player_idx = self.current_player_index();
        if *self.game_state.game_phase() == GamePhase::Attack
            && self.game_state.players()[player_idx].player_type() == &PlayerType::Human
        {
            debug("Human player passed attack");
            // Pass involves skipping to draw phase
            self.game_state.draw_cards();

            // After drawing, let AI play if it's their turn
            self.process_ai_turn();
        } else {
            debug("Cannot pass - not in attack phase or not human player's turn");
        }
    }

    fn take_cards_action(&mut self) {
        let player_idx = self.current_player_index();
        if *self.game_state.game_phase() == GamePhase::Defense
            && self.game_state.players()[player_idx].player_type() == &PlayerType::Human
        {
            debug("Human player taking cards");
            if let Err(e) = self.game_state.take_cards() {
                debug(format!("Error taking cards: {}", e));
                return;
            }
            // After taking cards, move to drawing phase
            self.game_state.draw_cards();

            // Process AI's turn if they're next
            self.process_ai_turn();
        } else {
            debug("Ignoring take cards action - not in Defense phase or not human player's turn");
        }
    }

    fn acknowledge_draw_action(&mut self) {
        debug("Acknowledging draw phase");

        if *self.game_state.game_phase() == GamePhase::Drawing {
            self.game_state.draw_cards();

            if *self.game_state.game_phase() == GamePhase::Drawing {
                debug("Drawing phase stuck, forcing Attack");
                self.game_state = GameState::force_attack_phase(self.game_state.clone());
            }

            // Check game over - this also sets the winner
            if self.game_state.check_game_over() {
                self.app_state = AppState::GameOver;
                return;
            }

            // Let AI play if it's their turn
            self.process_ai_turn();
        }
    }

    fn handle_attack_phase(&mut self, player_idx: usize) -> Result<(), String> {
        // Handle attack phase for human player only
        if self.game_state.players()[player_idx].player_type() != &PlayerType::Human {
            return Err("Not human player's turn".to_string());
        }

        debug("Human is attacking");

        // Handle multi-card attack or single card attack
        if self.multiple_selection_mode && !self.selected_cards.is_empty() {
            debug(format!(
                "Multi-attack with {} cards",
                self.selected_cards.len()
            ));
            let result = self.multi_attack(player_idx);
            self.selected_cards.clear();
            result
        } else if let Some(idx) = self.selected_card_idx {
            debug(format!("Single attack with card {}", idx));
            match self.game_state.attack(idx, player_idx) {
                Ok(()) => Ok(()),
                Err(e) => Err(e.to_string()),
            }
        } else {
            Err("No card selected for attack".to_string())
        }
    }

    fn handle_defense_phase(&mut self, player_idx: usize) -> Result<(), String> {
        // Validate we're in defense phase and the correct player is defending
        if self.game_state.game_phase() != &GamePhase::Defense {
            return Err("Not in defense phase".to_string());
        }

        if self.game_state.current_defender() != player_idx {
            return Err("Wrong player defending".to_string());
        }

        let defender_type = self.game_state.players()[player_idx].player_type();
        match defender_type {
            PlayerType::Human => {
                if !self.multiple_selection_mode || self.selected_cards.is_empty() {
                    // Single card selection mode or no selections
                    if let Some(idx) = self.selected_card_idx {
                        if let Ok(_) = self.game_state.defend(idx) {
                            debug(format!("Successfully defended with card {}", idx));

                            // Check if a pass occurred by looking at the defender change
                            if self.game_state.current_defender() != player_idx {
                                debug("Player passed the card to a different player!");
                                
                                // Process AI's turn if they're now the defender after the pass
                                let new_defender = self.game_state.current_defender();
                                let is_ai_defender = self.game_state.players()[new_defender].player_type() 
                                    == &PlayerType::Computer;
                                
                                if is_ai_defender {
                                    // Don't return yet, let the calling function handle AI processing
                                    debug("AI needs to defend after player's pass");
                                }
                                return Ok(());
                            }

                            // Check if all attacks are defended
                            let all_defended = !self
                                .game_state
                                .table_cards()
                                .iter()
                                .any(|(_, defense)| defense.is_none());

                            if all_defended {
                                debug("All attacks defended - discarding cards from table");

                                // Get all cards from the table for discarding
                                let cards_to_discard: Vec<(usize, Card)> = self
                                    .game_state
                                    .table_cards()
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(idx, (_, defense))| {
                                        defense.map(|card| (idx, card))
                                    })
                                    .collect();

                                // Discard the cards, which will also update game phase
                                self.game_state.discard_cards(cards_to_discard);

                                debug("All attacks defended!");
                                return Ok(());
                            }
                            return Ok(());
                        } else {
                            return Err("Invalid defense".to_string());
                        }
                    } else {
                        return Err("No card selected".to_string());
                    }
                } else {
                    // Multi-card defense
                    return self.handle_multi_defense(player_idx);
                }
            }
            _ => Err("Not human player's turn".to_string()),
        }
    }

    fn handle_multi_defense(&mut self, player_idx: usize) -> Result<(), String> {
        // Get undefended attacks and player's hand
        let table_cards = self.game_state.table_cards();
        let player = &self.game_state.players()[player_idx];
        let trump_suit = self.game_state.trump_suit();

        // Count undefended attacks
        let undefended_attacks: Vec<(usize, &Card)> = table_cards
            .iter()
            .enumerate()
            .filter(|(_, (_, defense))| defense.is_none())
            .map(|(idx, (attack, _))| (idx, attack))
            .collect();

        // If no undefended attacks, nothing to do - defense is complete
        if undefended_attacks.is_empty() {
            debug("No undefended attacks left, defense complete");
            return Ok(());
        }

        // Check if we have enough selected cards
        if self.selected_cards.len() != undefended_attacks.len() {
            debug(format!(
                "Selected {} cards but need to defend against {} attacks",
                self.selected_cards.len(),
                undefended_attacks.len()
            ));
            return Ok(());
        }

        // Create a mapping of attack cards to selected defense cards
        let mut defense_mapping = Vec::new();
        let mut used_card_indices = Vec::new();

        // Try to find valid defenses for each attack
        for (table_idx, attack_card) in undefended_attacks {
            // Try to find a valid defense from selected cards
            if let Some((hand_idx, defense_card)) = self
                .selected_cards
                .iter()
                .filter(|&&idx| !used_card_indices.contains(&idx))
                .map(|&idx| (idx, player.hand()[idx]))
                .find(|(_, card)| {
                    // Check if card is valid for defense
                    match trump_suit {
                        Some(trump) => {
                            if attack_card.suit == trump {
                                // If attacking with trump, must defend with higher trump
                                card.suit == trump && card.rank > attack_card.rank
                            } else if card.suit == trump {
                                // Any trump can beat a non-trump
                                true
                            } else {
                                // Otherwise must be same suit and higher rank
                                card.suit == attack_card.suit && card.rank > attack_card.rank
                            }
                        }
                        None => {
                            // No trump, just same suit and higher rank
                            card.suit == attack_card.suit && card.rank > attack_card.rank
                        }
                    }
                })
            {
                // Found a valid defense
                defense_mapping.push((table_idx, hand_idx, defense_card));
                used_card_indices.push(hand_idx);
            } else {
                // Couldn't find a defense for this attack
                debug("Cannot defend all attacks with selected cards");
                let _ = self.game_state.take_cards();
                self.selected_cards.clear();
                self.selected_card_idx = None;
                return Ok(());
            }
        }

        // Successfully mapped all attacks to defenses
        debug(format!("Applying {} valid defenses", defense_mapping.len()));

        // Format required by discard_cards: Vec<(usize, Card)> where usize is the index in the table
        let cards_to_discard: Vec<(usize, Card)> = defense_mapping
            .iter()
            .map(|&(table_idx, _, card)| (table_idx, card))
            .collect();

        // Remove the cards from player's hand using a mutable reference
        let game_state = &mut self.game_state;
        for &(_, hand_idx, _) in &defense_mapping {
            let _ = game_state.players_mut()[player_idx].remove_card(hand_idx);
        }

        game_state.discard_cards(cards_to_discard);

        // Clear selections
        self.selected_cards.clear();
        self.selected_card_idx = None;

        // Check if all cards are defended now
        let all_defended = !self
            .game_state
            .table_cards()
            .iter()
            .any(|(_, defense)| defense.is_none());

        if all_defended {
            debug("All attacks successfully defended");
            return Ok(());
        }

        Err("Not all cards defended".to_string())
    }

    fn valid_multi_attack(&self, player_idx: usize) -> bool {
        let hand = self.game_state.players()[player_idx].hand();
        let first_rank = hand[self.selected_cards[0]].rank;
        let sorted_indexes = self.selected_cards.len();
        let defender_hand_size =
            self.game_state.players()[self.game_state.current_defender()].hand_size();
        if sorted_indexes > defender_hand_size {
            // Make sure we don't attack with more cards than the defender has
            return false;
        }
        // Return false if selected cards don't all have the same rank, true otherwise
        return self
            .selected_cards
            .iter()
            .all(|&idx| hand[idx].rank == first_rank);
    }

    fn multi_attack(&mut self, player_idx: usize) -> Result<(), String> {
        // General logic for either computer or human attack with multiple cards.
        if self.selected_cards.is_empty() {
            return Err("No cards selected".to_string());
        }
        if self.game_state.game_phase() != &GamePhase::Attack {
            return Err("Not in attack phase".to_string());
        }
        // sort selected cards
        let mut sorted_indexes = self.selected_cards.clone();
        sorted_indexes.sort_by(|a, b| b.cmp(a));
        // Get the hand, validate the indexes.
        if !self.valid_multi_attack(player_idx) {
            return Err("Selected cards have different ranks".to_string());
        }
        for (_, &idx) in sorted_indexes.iter().enumerate() {
            match self.game_state.attack(idx, player_idx) {
                Ok(_) => {}
                Err(e) => return Err(format!("Multi-attack failed: {}", &e)),
            }
        }
        return Ok(());
    }

    pub fn render<B: Backend>(&self, terminal: &mut Terminal<B>) -> io::Result<()> {
        terminal.draw(|f| render_ui(self, f))?;
        Ok(())
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        while !self.should_quit {
            self.render(terminal)?;
            // Read user input every 100ms
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

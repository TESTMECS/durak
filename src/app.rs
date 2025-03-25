use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use log::{
    debug as log_debug, error as log_error, info as log_info, trace as log_trace, warn as log_warn,
};
use ratatui::backend::Backend;
use ratatui::Terminal;
use std::io;

use crate::game::{AiDifficulty, AiPlayer, GamePhase, GameState, PlayerType};
use crate::ui::{debug, error, info, trace, warn, DebugOverlay, GameUI};

#[derive(Debug)]
pub enum AppState {
    MainMenu,
    Playing,
    GameOver,
}

pub struct App {
    game_state: GameState,
    app_state: AppState,
    selected_card_idx: Option<usize>,
    ai_player: AiPlayer,
    should_quit: bool,
    show_debug: bool,
}

impl App {
    pub fn new() -> Self {
        info("Creating new App instance");
        log_info!("Creating new App instance");
        let mut game_state = GameState::new();

        // Add a human player
        game_state.add_player("Player".to_string(), PlayerType::Human);

        // Add an AI opponent
        game_state.add_player("Computer".to_string(), PlayerType::Computer);

        Self {
            game_state,
            app_state: AppState::MainMenu,
            selected_card_idx: None,
            ai_player: AiPlayer::new(AiDifficulty::Medium),
            should_quit: false,
            show_debug: false,
        }
    }

    pub fn toggle_debug(&mut self) {
        self.show_debug = !self.show_debug;
        info(format!(
            "Debug overlay {}",
            if self.show_debug {
                "enabled"
            } else {
                "disabled"
            }
        ));
    }

    pub fn quit(&mut self) {
        info("User quit the game");
        log_info!("User quit the game");
        self.should_quit = true;
    }

    #[allow(dead_code)]
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn start_game(&mut self) {
        info("Starting new game");
        log_info!("Starting new game");
        self.game_state.setup_game();
        self.app_state = AppState::Playing;
        self.selected_card_idx = None;

        // If AI goes first, make its move
        debug("Checking if AI goes first");
        log_debug!("Checking if AI goes first");
        self.process_ai_turn();
    }

    pub fn process_ai_turn(&mut self) {
        if *self.game_state.game_phase() == GamePhase::GameOver {
            debug("Game is over, setting app state to GameOver");
            log_debug!("Game is over, setting app state to GameOver");
            self.app_state = AppState::GameOver;
            return;
        }

        // Ensure we're in a proper phase for AI to act
        if !matches!(*self.game_state.game_phase(), GamePhase::Attack | GamePhase::Defense) {
            debug(format!("Not in Attack/Defense phase, skipping AI processing: {:?}", 
                self.game_state.game_phase()));
            log_debug!("Not in Attack/Defense phase, skipping AI processing: {:?}", 
                self.game_state.game_phase());
            return;
        }

        // Use an iterative approach to prevent stack overflow
        let continue_processing = true;
        debug("Starting AI turn processing loop");
        log_debug!("Starting AI turn processing loop");

        // Check deck state
        if self.game_state.deck().is_empty() {
            trace("Deck is empty, checking if game should end");
            log_trace!("Deck is empty, checking if game should end");
        }

        while continue_processing {
            // Check the current table state
            let table_count = self.game_state.table_cards().len();
            trace(format!("Current table has {} cards/pairs", table_count));
            log_trace!("Current table has {} cards/pairs", table_count);

            // Get the current player index based on game phase
            let current_player_idx = match *self.game_state.game_phase() {
                GamePhase::Attack => self.game_state.current_attacker(),
                GamePhase::Defense => self.game_state.current_defender(),
                _ => {
                    // Not in a phase where a player makes a move
                    debug(format!(
                        "Not in Attack/Defense phase, stopping AI processing: {:?}",
                        self.game_state.game_phase()
                    ));
                    log_debug!(
                        "Not in Attack/Defense phase, stopping AI processing: {:?}",
                        self.game_state.game_phase()
                    );
                    break;
                }
            };

            // Check if it's the AI's turn
            let is_ai_turn = self.game_state.players()[current_player_idx].player_type()
                == &PlayerType::Computer;

            if !is_ai_turn {
                // Not AI's turn, exit the loop
                debug(format!(
                    "Not AI's turn, current player {} is human",
                    current_player_idx
                ));
                log_debug!("Not AI's turn, current player {} is human", current_player_idx);
                break;
            }

            // Check if game is over after the last move
            if *self.game_state.game_phase() == GamePhase::GameOver {
                info("Game is over during AI processing, updating app state");
                log_info!("Game is over during AI processing, updating app state");
                self.app_state = AppState::GameOver;
                break;
            }

            // AI makes its move based on the current phase
            if *self.game_state.game_phase() == GamePhase::Attack {
                debug(format!("AI is attacking (player {})", current_player_idx));
                log_debug!("AI is attacking (player {})", current_player_idx);
                let player_hand_size = self.game_state.players()[current_player_idx].hand_size();
                debug(format!("AI attacker has {} cards in hand", player_hand_size));
                log_debug!("AI attacker has {} cards in hand", player_hand_size);
                
                if let Some(card_idx) = self
                    .ai_player
                    .make_attack_move(&self.game_state, current_player_idx)
                {
                    // AI makes an attack
                    info(format!("AI chose to attack with card index {}", card_idx));
                    log_info!("AI chose to attack with card index {}", card_idx);
                    let result = self.game_state.attack(card_idx);
                    if let Err(e) = result {
                        error(format!("AI attack failed: {}", e));
                        log_error!("AI attack failed: {}", e);
                        // If attack failed, we should exit to prevent infinite loop
                        debug("Breaking AI processing loop due to failed attack");
                        log_debug!("Breaking AI processing loop due to failed attack");
                        break;
                    } else {
                        debug("AI attack successful, continuing processing");
                        log_debug!("AI attack successful, continuing processing");
                    }
                } else {
                    // AI passes
                    info("AI chose to pass (no valid attack)");
                    log_info!("AI chose to pass (no valid attack)");
                    
                    // Check if we should go to drawing phase
                    let need_to_draw = self.game_state.players().iter().any(|p| p.hand_size() < 6) && 
                                     !self.game_state.deck().is_empty();
                    
                    if need_to_draw {
                        debug("AI passing and there are cards to draw");
                        log_debug!("AI passing and there are cards to draw");
                        self.game_state.draw_cards();
                    } else {
                        debug("AI passing but no cards need to be drawn, directly changing turns");
                        log_debug!("AI passing but no cards need to be drawn, directly changing turns");
                        // Skip drawing phase and just change the turns
                        // Logic to advance to next player without drawing
                        // This simulates what draw_cards would do without trying to draw
                        if *self.game_state.game_phase() == GamePhase::Attack {
                            // Here we skip ahead
                            debug("Advancing directly to next attacker without drawing");
                            log_debug!("Advancing directly to next attacker without drawing");
                            // This will ensure turns change properly
                            self.game_state.draw_cards();
                        }
                    }
                    // Break after passing to give human a turn
                    break;
                }
            } else if *self.game_state.game_phase() == GamePhase::Defense {
                debug(format!("AI is defending (player {})", current_player_idx));
                log_debug!("AI is defending (player {})", current_player_idx);
                let player_hand_size = self.game_state.players()[current_player_idx].hand_size();
                debug(format!("AI defender has {} cards in hand", player_hand_size));
                log_debug!("AI defender has {} cards in hand", player_hand_size);
                
                if self
                    .ai_player
                    .should_take_cards(&self.game_state, current_player_idx)
                {
                    // AI takes cards
                    info("AI chose to take cards");
                    log_info!("AI chose to take cards");
                    let result = self.game_state.take_cards();
                    if let Err(e) = result {
                        error(format!("AI failed to take cards: {}", e));
                        log_error!("AI failed to take cards: {}", e);
                        // Break after error to prevent infinite loop
                        break;
                    } else {
                        // Check if we need to draw cards
                        let need_to_draw = self.game_state.players().iter().any(|p| p.hand_size() < 6) && 
                                           !self.game_state.deck().is_empty();
                        
                        if need_to_draw {
                            debug("AI took cards and there are cards to draw");
                            log_debug!("AI took cards and there are cards to draw");
                            self.game_state.draw_cards();
                        } else {
                            debug("AI took cards but no cards need to be drawn, directly changing turns");
                            log_debug!("AI took cards but no cards need to be drawn, directly changing turns");
                            // Still call draw_cards to handle turn changes
                            self.game_state.draw_cards();
                        }
                        // After taking cards, break to allow next player to move
                        break;
                    }
                } else if let Some(card_idx) = self
                    .ai_player
                    .make_defense_move(&self.game_state, current_player_idx)
                {
                    // AI defends
                    info(format!("AI chose to defend with card index {}", card_idx));
                    log_info!("AI chose to defend with card index {}", card_idx);
                    let result = self.game_state.defend(card_idx);
                    if let Err(e) = result {
                        error(format!("AI defense failed: {}", e));
                        log_error!("AI defense failed: {}", e);

                        // If the AI can't defend, it must take cards
                        warn("AI defense failed, attempting to take cards instead");
                        log_warn!("AI defense failed, attempting to take cards instead");
                        let take_result = self.game_state.take_cards();
                        if let Err(take_err) = take_result {
                            error(format!("AI also failed to take cards: {}", take_err));
                            log_error!("AI also failed to take cards: {}", take_err);
                            // Break to prevent infinite loop after error
                            break;
                        } else {
                            // Check if players need to draw cards
                            let need_to_draw = self.game_state.players().iter().any(|p| p.hand_size() < 6) && 
                                               !self.game_state.deck().is_empty();
                            
                            if need_to_draw {
                                debug("AI defense failed, took cards and there are cards to draw");
                                log_debug!("AI defense failed, took cards and there are cards to draw");
                            } else {
                                debug("AI defense failed, took cards but no cards need to be drawn");
                                log_debug!("AI defense failed, took cards but no cards need to be drawn");
                            }
                            self.game_state.draw_cards();
                            break; // Break after taking cards
                        }
                    } else {
                        debug("AI successfully defended, checking for more attacks");
                        log_debug!("AI successfully defended, checking for more attacks");
                        // Continue the loop to see if there are more attacks to defend
                    }
                } else {
                    // AI has no valid defense and should take cards
                    warn("AI has no valid defense but didn't choose to take cards - forcing take cards");
                    log_warn!("AI has no valid defense but didn't choose to take cards - forcing take cards");
                    let result = self.game_state.take_cards();
                    if let Err(e) = result {
                        error(format!("AI forced take cards failed: {}", e));
                        log_error!("AI forced take cards failed: {}", e);
                        // Break on error
                        break;
                    } else {
                        // Check if we need to draw cards
                        let need_to_draw = self.game_state.players().iter().any(|p| p.hand_size() < 6) && 
                                           !self.game_state.deck().is_empty();
                        
                        if need_to_draw {
                            debug("AI was forced to take cards and there are cards to draw");
                            log_debug!("AI was forced to take cards and there are cards to draw");
                            self.game_state.draw_cards();
                        } else {
                            debug("AI was forced to take cards but no cards need to be drawn, directly changing turns");
                            log_debug!("AI was forced to take cards but no cards need to be drawn, directly changing turns");
                            // Still call draw_cards to handle turn changes
                            self.game_state.draw_cards();
                        }
                        // Break to avoid infinite loop after forced take
                        break;
                    }
                }
            }

            trace(format!(
                "AI turn complete, game phase now: {:?}",
                self.game_state.game_phase()
            ));
            log_trace!(
                "AI turn complete, game phase now: {:?}",
                self.game_state.game_phase()
            );

            // Check if game is over due to empty deck and empty hand
            if self.game_state.deck().is_empty() {
                // Re-check if someone won
                for (idx, player) in self.game_state.players().iter().enumerate() {
                    if player.is_empty_hand() {
                        info(format!(
                            "Game ending - player {} has no cards left and deck is empty!",
                            idx
                        ));
                        log_info!(
                            "Game ending - player {} has no cards left and deck is empty!",
                            idx
                        );
                        self.app_state = AppState::GameOver;
                        break;
                    }
                }
            }
            
            // After a complete AI action, check if it's still the AI's turn
            if self.game_state.players()[self.current_player_index()].player_type() == &PlayerType::Computer {
                debug("It's still AI's turn, continuing the processing loop");
                log_debug!("It's still AI's turn, continuing the processing loop");
            } else {
                debug("AI turn complete, now it's human's turn");
                log_debug!("AI turn complete, now it's human's turn");
                break;
            }
        }

        debug("AI processing complete");
        log_debug!("AI processing complete");
    }

    // Helper to get current player index based on game phase
    fn current_player_index(&self) -> usize {
        match *self.game_state.game_phase() {
            GamePhase::Attack => self.game_state.current_attacker(),
            GamePhase::Defense => self.game_state.current_defender(),
            _ => self.game_state.current_attacker() // Default to attacker for other phases
        }
    }

    pub fn on_key(&mut self, key: KeyCode) {
        trace(format!(
            "Key pressed: {:?}, current state: {:?}",
            key, self.app_state
        ));
        log_trace!(
            "Key pressed: {:?}, current state: {:?}",
            key,
            self.app_state
        );

        // Handle Drawing phase first - any key will exit it
        if *self.game_state.game_phase() == GamePhase::Drawing {
            debug("Key pressed during Drawing phase - processing draw cards");
            log_debug!("Key pressed during Drawing phase - processing draw cards");
            
            // Call draw_cards to attempt to exit the Drawing phase
            self.game_state.draw_cards();
            
            // If we're still in Drawing phase, something is wrong - force it
            if *self.game_state.game_phase() == GamePhase::Drawing {
                warn("Drawing phase is stuck, forcing transition to Attack phase");
                self.game_state = GameState::force_attack_phase(self.game_state.clone());
            }
            
            // Process AI turn after exiting Drawing phase, regardless of how we exited
            debug("Checking if AI should play after exiting Drawing phase");
            log_debug!("Checking if AI should play after exiting Drawing phase");
            
            if *self.game_state.game_phase() == GamePhase::Attack &&
               self.game_state.players()[self.game_state.current_attacker()].player_type() == &PlayerType::Computer {
                debug("Current attacker is AI - processing AI turn after Drawing phase");
                log_debug!("Current attacker is AI - processing AI turn after Drawing phase");
                self.process_ai_turn();
            }
            
            return;
        }

        // Handle debug toggle regardless of game state
        if key == KeyCode::Char('d') || key == KeyCode::Char('D') {
            self.toggle_debug();
            return;
        }
        
        match self.app_state {
            AppState::MainMenu => match key {
                KeyCode::Char('s') => {
                    debug("Start game key pressed");
                    log_debug!("Start game key pressed");
                    self.start_game();
                }
                KeyCode::Char('q') => {
                    debug("Quit key pressed from main menu");
                    log_debug!("Quit key pressed from main menu");
                    self.quit();
                }
                _ => {}
            },
            AppState::Playing => {
                match *self.game_state.game_phase() {
                    GamePhase::Attack | GamePhase::Defense => {
                        let current_player_idx =
                            if *self.game_state.game_phase() == GamePhase::Attack {
                                self.game_state.current_attacker()
                            } else {
                                self.game_state.current_defender()
                            };

                        let is_human_turn = self.game_state.players()[current_player_idx]
                            .player_type()
                            == &PlayerType::Human;

                        if is_human_turn {
                            let player = &self.game_state.players()[current_player_idx];
                            let hand_size = player.hand_size();
                            trace(format!("Human turn, hand size: {}", hand_size));
                            log_trace!("Human turn, hand size: {}", hand_size);

                            match key {
                                KeyCode::Up | KeyCode::Left => {
                                    // Move selection left
                                    let old_idx = self.selected_card_idx;
                                    self.selected_card_idx = match self.selected_card_idx {
                                        Some(idx) if idx > 0 => Some(idx - 1),
                                        None if hand_size > 0 => Some(0),
                                        _ => self.selected_card_idx,
                                    };
                                    debug(format!(
                                        "Card selection changed: {:?} -> {:?}",
                                        old_idx, self.selected_card_idx
                                    ));
                                    log_debug!(
                                        "Card selection changed: {:?} -> {:?}",
                                        old_idx,
                                        self.selected_card_idx
                                    );
                                }
                                KeyCode::Down | KeyCode::Right => {
                                    // Move selection right
                                    let old_idx = self.selected_card_idx;
                                    self.selected_card_idx = match self.selected_card_idx {
                                        Some(idx) if idx < hand_size - 1 => Some(idx + 1),
                                        None if hand_size > 0 => Some(0),
                                        _ => self.selected_card_idx,
                                    };
                                    debug(format!(
                                        "Card selection changed: {:?} -> {:?}",
                                        old_idx, self.selected_card_idx
                                    ));
                                    log_debug!(
                                        "Card selection changed: {:?} -> {:?}",
                                        old_idx,
                                        self.selected_card_idx
                                    );
                                }
                                KeyCode::Enter => {
                                    // Play selected card
                                    if let Some(idx) = self.selected_card_idx {
                                        debug(format!("Playing card at index {}", idx));
                                        log_debug!("Playing card at index {}", idx);

                                        if *self.game_state.game_phase() == GamePhase::Attack {
                                            info(format!("Human attacking with card {}", idx));
                                            log_info!("Human attacking with card {}", idx);
                                            let result = self.game_state.attack(idx);
                                            if let Err(e) = result {
                                                warn(format!("Human attack failed: {}", e));
                                                log_warn!("Human attack failed: {}", e);
                                                // We don't change the game state, player can try again with a different card
                                            } else {
                                                // Only process AI turn after a successful attack
                                                debug("Processing AI turn after successful human attack");
                                                log_debug!("Processing AI turn after successful human attack");
                                                self.process_ai_turn();
                                            }
                                        } else if *self.game_state.game_phase()
                                            == GamePhase::Defense
                                        {
                                            // Defense
                                            let card = self.game_state.players()
                                                [self.game_state.current_defender()]
                                            .hand()[idx];
                                            info(format!(
                                                "Human defending with card {} ({})",
                                                idx, card
                                            ));
                                            log_info!(
                                                "Human defending with card {} ({})",
                                                idx,
                                                card
                                            );

                                            let result = self.game_state.defend(idx);
                                            if let Err(e) = result {
                                                warn(format!("Human defense failed: {}", e));
                                                log_warn!("Human defense failed: {}", e);
                                                error(format!(
                                                    "Can't use card at index {}: {}",
                                                    idx, e
                                                ));
                                                log_error!(
                                                    "Can't use card at index {}: {}",
                                                    idx,
                                                    e
                                                );
                                                // We don't change the game state, player can try again with a different card
                                            } else {
                                                // Only process AI turn after successful defense
                                                debug("Processing AI turn after successful human defense");
                                                log_debug!("Processing AI turn after successful human defense");

                                                // Check if game is now over after defending
                                                if self.game_state.deck().is_empty() {
                                                    // Re-check for winners
                                                    for (idx, player) in
                                                        self.game_state.players().iter().enumerate()
                                                    {
                                                        if player.is_empty_hand() {
                                                            info(format!("Game is over - player {} has emptied their hand!", idx));
                                                            log_info!("Game is over - player {} has emptied their hand!", idx);
                                                            self.app_state = AppState::GameOver;
                                                            return;
                                                        }
                                                    }
                                                }

                                                self.process_ai_turn();
                                            }
                                        } else {
                                            warn(format!(
                                                "Enter pressed in non-interactive phase: {:?}",
                                                self.game_state.game_phase()
                                            ));
                                            log_warn!(
                                                "Enter pressed in non-interactive phase: {:?}",
                                                self.game_state.game_phase()
                                            );
                                        }
                                    } else {
                                        debug("Enter pressed but no card selected");
                                        log_debug!("Enter pressed but no card selected");
                                    }
                                }
                                KeyCode::Char('t') => {
                                    // Take cards (in defense phase)
                                    if *self.game_state.game_phase() == GamePhase::Defense {
                                        info("Human chose to take cards");
                                        log_info!("Human chose to take cards");
                                        let result = self.game_state.take_cards();
                                        if let Err(e) = result {
                                            error(format!("Human take cards failed: {}", e));
                                            log_error!("Human take cards failed: {}", e);
                                            warn("Unable to take cards, please try a different action");
                                            log_warn!("Unable to take cards, please try a different action");
                                        } else {
                                            debug("Successfully took cards, now drawing new cards");
                                            log_debug!(
                                                "Successfully took cards, now drawing new cards"
                                            );
                                            
                                            // Check if players actually need to draw
                                            let need_to_draw = self.game_state.players().iter().any(|p| p.hand_size() < 6) && 
                                                              !self.game_state.deck().is_empty();
                                            
                                            if need_to_draw {
                                                debug("Human took cards and there are cards to draw");
                                                log_debug!("Human took cards and there are cards to draw");
                                            } else {
                                                debug("Human took cards but no cards need to be drawn - direct turn change");
                                                log_debug!("Human took cards but no cards need to be drawn - direct turn change");
                                            }
                                            
                                            // Check if deck is empty before drawing
                                            if self.game_state.deck().is_empty() {
                                                debug("Deck is empty when taking cards, proceeding with turn change");
                                                log_debug!("Deck is empty when taking cards, proceeding with turn change");
                                            }
                                            
                                            self.game_state.draw_cards();
                                            
                                            // Check if game is now over after taking cards
                                            if *self.game_state.game_phase() == GamePhase::GameOver {
                                                info("Game is over after taking cards");
                                                log_info!("Game is over after taking cards");
                                                self.app_state = AppState::GameOver;
                                            } else {
                                                debug("After drawing cards, processing AI turn");
                                                log_debug!("After drawing cards, processing AI turn");
                                                self.process_ai_turn();
                                            }
                                        }
                                    } else {
                                        debug("Take cards key pressed outside of defense phase");
                                        log_debug!(
                                            "Take cards key pressed outside of defense phase"
                                        );
                                    }
                                },
                                KeyCode::Char('p') => {
                                    // Pass (in attack phase)
                                    if *self.game_state.game_phase() == GamePhase::Attack {
                                        info("Human chose to pass");
                                        log_info!("Human chose to pass");
                                        
                                        // Check if we actually need to draw any cards
                                        let need_to_draw = self.game_state.players().iter().any(|p| p.hand_size() < 6) && 
                                                          !self.game_state.deck().is_empty();
                                        
                                        if need_to_draw {
                                            debug("Human passed and there are cards to draw");
                                            log_debug!("Human passed and there are cards to draw");
                                        } else {
                                            debug("Human passed but no cards need to be drawn - direct turn change");
                                            log_debug!("Human passed but no cards need to be drawn - direct turn change");
                                        }
                                        
                                        // Check if deck is empty - we still need to change turns
                                        if self.game_state.deck().is_empty() {
                                            debug("Deck is empty when passing, still managing turn change");
                                            log_debug!("Deck is empty when passing, still managing turn change");
                                        }
                                        
                                        self.game_state.draw_cards();
                                        
                                        // Check if game is now over after passing
                                        if *self.game_state.game_phase() == GamePhase::GameOver {
                                            info("Game is over after passing");
                                            log_info!("Game is over after passing");
                                            self.app_state = AppState::GameOver;
                                        } else {
                                            self.process_ai_turn();
                                        }
                                    } else {
                                        debug("Pass key pressed outside of attack phase");
                                        log_debug!("Pass key pressed outside of attack phase");
                                    }
                                },
                                KeyCode::Char('q') => {
                                    debug("Quit key pressed during game");
                                    log_debug!("Quit key pressed during game");
                                    self.quit();
                                }
                                _ => {}
                            }
                        } else {
                            debug("Key pressed during AI turn, ignoring");
                            log_debug!("Key pressed during AI turn, ignoring");
                        }
                    }
                    GamePhase::GameOver => match key {
                        KeyCode::Char('n') => {
                            debug("New game key pressed after game over");
                            log_debug!("New game key pressed after game over");
                            self.start_game();
                        }
                        KeyCode::Char('q') => {
                            debug("Quit key pressed after game over");
                            log_debug!("Quit key pressed after game over");
                            self.quit();
                        }
                        _ => {}
                    },
                    _ => {
                        trace(format!(
                            "Key pressed in non-interactive phase: {:?}",
                            self.game_state.game_phase()
                        ));
                        log_trace!(
                            "Key pressed in non-interactive phase: {:?}",
                            self.game_state.game_phase()
                        );
                    }
                }
            }
            AppState::GameOver => match key {
                KeyCode::Char('n') => {
                    debug("New game key pressed at game over screen");
                    log_debug!("New game key pressed at game over screen");
                    self.start_game();
                }
                KeyCode::Char('q') => {
                    debug("Quit key pressed at game over screen");
                    log_debug!("Quit key pressed at game over screen");
                    self.quit();
                }
                _ => {}
            },
        }
    }

    pub fn render<B: Backend>(&self, terminal: &mut Terminal<B>) -> io::Result<()> {
        terminal.draw(|f| {
            let area = f.size();

            match self.app_state {
                AppState::MainMenu => {
                    // Render main menu
                    let title = ratatui::widgets::Paragraph::new("Durak Card Game")
                        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Green))
                        .alignment(ratatui::layout::Alignment::Center)
                        .block(
                            ratatui::widgets::Block::default()
                                .borders(ratatui::widgets::Borders::ALL),
                        );

                    let menu = ratatui::widgets::Paragraph::new(vec![
                        ratatui::text::Line::from("Press 's' to start a new game"),
                        ratatui::text::Line::from("Press 'q' to quit"),
                        ratatui::text::Line::from("Press 'd' to toggle debug overlay"),
                    ])
                    .style(ratatui::style::Style::default().fg(ratatui::style::Color::White))
                    .alignment(ratatui::layout::Alignment::Center);

                    let layout = ratatui::layout::Layout::default()
                        .direction(ratatui::layout::Direction::Vertical)
                        .constraints([
                            ratatui::layout::Constraint::Percentage(40),
                            ratatui::layout::Constraint::Length(3),
                            ratatui::layout::Constraint::Length(4),
                            ratatui::layout::Constraint::Percentage(40),
                        ])
                        .split(area);

                    f.render_widget(title, layout[1]);
                    f.render_widget(menu, layout[2]);
                }
                AppState::Playing | AppState::GameOver => {
                    // Render game UI
                    let game_ui = GameUI::new(&self.game_state).select_card(self.selected_card_idx);
                    f.render_widget(game_ui, area);
                }
            }

            if self.show_debug {
                let debug_overlay = DebugOverlay::new();
                f.render_widget(debug_overlay, area);
            }
        })?;

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

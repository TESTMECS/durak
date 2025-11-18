use super::ai_handler::process_ai_turn;
use super::app_core::App;
use super::state::AppState;
use crate::game::{AiDifficulty, AiPlayer, GamePhase, GameState, PlayerType, card::Card};
use crate::ui::debug_overlay::{debug, error};
impl App {
    pub fn start_game_action(&mut self) {
        self.app_state = AppState::Playing;
        self.game_state.setup_game();
        self.selected_card_idx = None;
        self.selected_cards.clear();
        self.multiple_selection_mode = false;
        self.ai_player = AiPlayer::new(self.selected_difficulty);
        debug(format!(
            "Starting game with AI difficulty: {}",
            self.selected_difficulty
        ));
        match self.selected_difficulty {
            AiDifficulty::Easy => {
                debug("Easy AI: Will play lowest cards, often take cards instead of defending");
            }
            AiDifficulty::Medium => {
                debug(
                    "Medium AI: Will use basic strategy, manage trumps, and sometimes pass cards",
                );
            }
            AiDifficulty::Hard => {
                debug(
                    "Hard AI: Will strategically track cards, exploit weaknesses, and plan ahead",
                );
            }
        }
        debug("Game started!");
        if self.is_ai() {
            debug("AI goes first");
            process_ai_turn(self);
        }
    }
    pub fn select_next_card(&mut self) {
        if let Some(player) = self.game_state.players().get(self.current_player_index()) {
            if player.is_human() {
                let hand_size = player.hand_size();
                if hand_size > 0 {
                    let old_idx = self.selected_card_idx;
                    self.selected_card_idx = Some(match self.selected_card_idx {
                        Some(i) => (i + 1) % hand_size,
                        None => 0,
                    });
                    debug(format!(
                        "Select next: {:?} -> {:?}",
                        old_idx, self.selected_card_idx
                    ));
                }
            }
        }
    }
    pub fn select_prev_card(&mut self) {
        if let Some(player) = self.game_state.players().get(self.current_player_index()) {
            if player.is_human() {
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
    pub fn play_card_action(&mut self) {
        let idx = self.current_player_index();
        let players = self.game_state.players();
        let player = &players[idx];

        if player.player_type() == &PlayerType::Human {
            match *self.game_state.game_phase() {
                GamePhase::Attack => {
                    if self.handle_attack_phase(idx).is_ok() {
                        process_ai_turn(self);
                    }
                }
                GamePhase::Defense => {
                    match self.handle_defense_phase(idx) {
                        Ok(_) => {
                            if *self.game_state.game_phase() == GamePhase::Drawing {
                                self.game_state.draw_cards();
                                process_ai_turn(self);
                            } else if *self.game_state.game_phase() == GamePhase::Defense {
                                let current_defender = self.game_state.current_defender();
                                if current_defender != idx {
                                    debug("Detected pass - different player now defending");
                                    let is_ai_defender =
                                        self.game_state.players()[current_defender].player_type()
                                            == &PlayerType::Computer;
                                    if is_ai_defender {
                                        debug("AI is now defending after pass, processing AI turn");
                                        process_ai_turn(self);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            debug(format!("Defense failed: {}", e));
                            // Not a fatal error, just log it and continue
                            // Only non-fatal game rule errors should reach here
                        }
                    }
                }
                _ => {}
            }
        }
    }
    /// Skip to the draw phase if either player gets stuck.  
    /// TODO: Initally had this for debugging but should be removed
    pub fn pass_turn_action(&mut self) {
        let player_idx = self.current_player_index();
        if *self.game_state.game_phase() == GamePhase::Attack
            && self.game_state.players()[player_idx].player_type() == &PlayerType::Human
        {
            debug("Human player passed attack");
            self.game_state.draw_cards();
            process_ai_turn(self);
        } else {
            debug("Cannot pass - not in attack phase or not human player's turn");
        }
    }
    /// Handles the case where the player cannot defend against an attack
    /// Player presses 't' to take cards
    /// Calls `game_state.take_cards`
    pub fn take_cards_action(&mut self) {
        let player_idx = self.current_player_index();
        if *self.game_state.game_phase() == GamePhase::Defense
            && self.game_state.players()[player_idx].player_type() == &PlayerType::Human
        {
            debug("Human player taking cards");
            if let Err(e) = self.game_state.take_cards() {
                debug(format!("Error taking cards: {}", e));
                return;
            }
            self.game_state.draw_cards();
            process_ai_turn(self);
        } else {
            debug("Ignoring take cards action - not in Defense phase or not human player's turn");
        }
    }
    /// Handles the drawing phase.
    /// Attempts to force the attack phase if this is called from GamePhase::Drawing
    /// Since this is still called when there are no cards left,
    /// GamePhase::GameOver is checked HERE
    pub fn acknowledge_draw_action(&mut self) {
        debug("Acknowledging draw phase");
        if *self.game_state.game_phase() == GamePhase::Drawing {
            self.game_state.draw_cards();
            if *self.game_state.game_phase() == GamePhase::Drawing {
                debug("Drawing phase stuck, forcing Attack");
                GameState::force_attack_phase(&mut self.game_state);
            }
            if self.game_state.check_game_over() {
                self.app_state = super::state::AppState::GameOver;
                return;
            }
            process_ai_turn(self);
        }
    }
    /// Handles the attack phase for single and multi-card attack the human player.
    pub fn handle_attack_phase(&mut self, player_idx: usize) -> Result<(), String> {
        if self.game_state.players()[player_idx].player_type() != &PlayerType::Human {
            return Err("Not human player's turn".to_string());
        }
        debug("Human is attacking");
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
    /// Handles the defense phase for the human player.
    pub fn handle_defense_phase(&mut self, player_idx: usize) -> Result<(), String> {
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
                        if self.game_state.defend(idx).is_ok() {
                            debug(format!("Successfully defended with card {}", idx));

                            // Check if a pass occurred by looking at the defender change
                            if self.game_state.current_defender() != player_idx {
                                debug("Player passed the card to a different player!");

                                // Process AI's turn if they're now the defender after the pass
                                let new_defender = self.game_state.current_defender();
                                let is_ai_defender = self.game_state.players()[new_defender]
                                    .player_type()
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
                                let cards_to_discard: Vec<(usize, Card)> = self
                                    .game_state
                                    .table_cards()
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(idx, (_, defense))| {
                                        defense.map(|card| (idx, card))
                                    })
                                    .collect();
                                self.game_state.discard_cards(cards_to_discard);
                                debug("All attacks defended!");
                                Ok(())
                            } else {
                                Ok(())
                            }
                        } else {
                            Err("Invalid defense".to_string())
                        }
                    } else {
                        Err("No card selected".to_string())
                    }
                } else {
                    // Multi-card defense
                    self.handle_multi_defense(player_idx)
                }
            }
            _ => Err("Not human player's turn".to_string()),
        }
    }
    /// Seperate function for multi-card defense for the human player.
    /// Called by `handle_defense_phase` above.
    pub fn handle_multi_defense(&mut self, player_idx: usize) -> Result<(), String> {
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
    /// Validates the selected cards for a multi-card attack by the human player
    pub fn valid_multi_attack(&self, player_idx: usize) -> bool {
        if self.selected_cards.is_empty() {
            return false;
        }
        let hand = self.game_state.players()[player_idx].hand();
        // Check for out of bounds indices
        if self.selected_cards.iter().any(|&idx| idx >= hand.len()) {
            error(format!(
                "Out of bounds index in valid_multi_attack: selected cards: {:?}, hand size: {}",
                self.selected_cards,
                hand.len()
            ));
            return false;
        }
        let first_idx = self.selected_cards[0];
        if first_idx >= hand.len() {
            error(format!(
                "First card index out of bounds: {}, hand size: {}",
                first_idx,
                hand.len()
            ));
            return false;
        }
        let first_rank = hand[first_idx].rank;
        let cards_count = self.selected_cards.len();
        // Make sure we don't attack with more cards than the defender has
        let defender = self.game_state.current_defender();
        if defender >= self.game_state.players().len() {
            error(format!("Invalid defender index: {}", defender));
            return false;
        }
        let defender_hand_size = self.game_state.players()[defender].hand_size();
        if cards_count > defender_hand_size {
            // Make sure we don't attack with more cards than the defender has
            return false;
        }
        // Return false if selected cards don't all have the same rank, true otherwise
        self.selected_cards
            .iter()
            .all(|&idx| idx < hand.len() && hand[idx].rank == first_rank)
    }
    /// Performs a multi-card attack with the human player.
    pub fn multi_attack(&mut self, player_idx: usize) -> Result<(), String> {
        // General logic for either computer or human attack with multiple cards.
        if self.selected_cards.is_empty() {
            return Err("No cards selected".to_string());
        }
        if self.game_state.game_phase() != &GamePhase::Attack {
            return Err("Not in attack phase".to_string());
        }
        // Safely clone the selected cards to avoid any potential index issues
        if self
            .selected_cards
            .iter()
            .any(|&idx| idx >= self.game_state.players()[player_idx].hand_size())
        {
            error("Index out of bounds in multi_attack");
            let err_msg = "Invalid card index";
            if let Err(e) = self.safe_exit(Some(err_msg)) {
                error(format!("Failed to restore terminal: {}", e));
            }
            return Err(err_msg.to_string());
        }
        // Sort selected cards (highest index first to avoid shifting issues)
        let mut sorted_indexes = self.selected_cards.clone();
        sorted_indexes.sort_by(|a, b| b.cmp(a));
        // Get the hand, validate the indexes.
        if !self.valid_multi_attack(player_idx) {
            return Err("Selected cards have different ranks".to_string());
        }
        // Perform the attacks
        for &idx in sorted_indexes.iter() {
            // Double-check index bounds before each attack
            if idx >= self.game_state.players()[player_idx].hand_size() {
                let err_msg = format!(
                    "Card index {} out of bounds (hand size: {})",
                    idx,
                    self.game_state.players()[player_idx].hand_size()
                );
                if let Err(e) = self.safe_exit(Some(&err_msg)) {
                    error(format!("Failed to restore terminal: {}", e));
                }
                return Err(err_msg);
            }
            match self.game_state.attack(idx, player_idx) {
                Ok(_) => {}
                Err(e) => return Err(format!("Multi-attack failed: {}", &e)),
            }
        }
        Ok(())
    }
    /// Helper method to find a card's index in a player's hand
    pub fn find_card_index_in_hand(&self, player_idx: usize, card: Card) -> Option<usize> {
        let player = &self.game_state.players()[player_idx];
        player
            .hand()
            .iter()
            .enumerate()
            .find(|&(_, &c)| c == card)
            .map(|(idx, _)| idx)
    }

    pub fn is_ai(&self) -> bool {
        self.game_state.players()[self.current_player_index()].player_type()
            == &PlayerType::Computer
    }
}

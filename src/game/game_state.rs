use super::card::{Card, Suit};
use super::deck::Deck;
use super::player::{Player, PlayerType};
use log::{debug, info, trace, warn};
use std::collections::VecDeque;

// Counter to detect when the drawing phase might be stuck
static mut STUCK_COUNTER: usize = 0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GamePhase {
    Setup,
    Attack,
    Defense,
    Drawing,
    GameOver,
}

#[derive(Debug, Clone)]
pub struct GameState {
    players: Vec<Player>,
    deck: Deck,
    discard_pile: Vec<Card>,
    table_cards: Vec<(Card, Option<Card>)>, // (attacking card, defending card)
    current_attacker: usize,
    current_defender: usize,
    trump_suit: Option<Suit>,
    game_phase: GamePhase,
    winner: Option<usize>,
}

impl GameState {
    pub fn new() -> Self {
        debug!("Creating new GameState");
        Self {
            players: Vec::new(),
            deck: Deck::new(),
            discard_pile: Vec::new(),
            table_cards: Vec::new(),
            current_attacker: 0,
            current_defender: 0,
            trump_suit: None,
            game_phase: GamePhase::Setup,
            winner: None,
        }
    }

    pub fn add_player(&mut self, name: String, player_type: PlayerType) {
        debug!("Adding player: {} (type: {:?})", name, player_type);
        self.players.push(Player::new(name, player_type));
    }

    pub fn setup_game(&mut self) {
        info!("Setting up new game with {} players", self.players.len());
        assert!(
            self.players.len() >= 2,
            "Need at least 2 players to start the game"
        );

        // Initialize and shuffle the deck
        self.deck = Deck::new();
        self.deck.shuffle();
        self.trump_suit = self.deck.trump_suit();
        info!("Deck shuffled, trump suit: {:?}", self.trump_suit);

        // Deal cards to players (6 cards each)
        for player in &mut self.players {
            let cards = self.deck.deal(6);
            debug!("Dealing 6 cards to {}", player.name());
            player.add_cards(cards);
        }

        // Determine who goes first (player with lowest trump card)
        self.determine_first_player();
        self.current_defender = (self.current_attacker + 1) % self.players.len();
        info!(
            "First attacker: {} ({}), first defender: {} ({})",
            self.current_attacker,
            self.players[self.current_attacker].name(),
            self.current_defender,
            self.players[self.current_defender].name()
        );

        self.game_phase = GamePhase::Attack;
    }

    fn determine_first_player(&mut self) {
        debug!("Determining first player");
        if let Some(trump_suit) = self.trump_suit {
            // Find the player with the lowest trump card
            let mut lowest_player = 0;
            let mut lowest_rank = None;

            for (i, player) in self.players.iter().enumerate() {
                if let Some((_, card)) = player.get_lowest_trump(trump_suit) {
                    debug!("Player {} has trump card: {:?}", i, card);
                    if lowest_rank.is_none() || card.rank < lowest_rank.unwrap() {
                        lowest_rank = Some(card.rank);
                        lowest_player = i;
                    }
                }
            }
            // If someone has a trump card, they go first
            if lowest_rank.is_some() {
                self.current_attacker = lowest_player;
                debug!("First player is {} with lowest trump", lowest_player);
                return;
            }
        }
        // If no one has a trump card or there's no trump suit, just start with player 0
        debug!("No one has trump cards, starting with player 0");
        self.current_attacker = 0;
    }

    pub fn attack(&mut self, card_idx: usize) -> Result<(), &'static str> {
        if self.game_phase != GamePhase::Attack {
            warn!("Attack attempt outside attack phase: {:?}", self.game_phase);
            return Err("Not in attack phase");
        }

        let attacker = &mut self.players[self.current_attacker];
        debug!(
            "Player {} attempting to attack with card index {}",
            self.current_attacker, card_idx
        );

        // Check if the card can be played
        if self.table_cards.is_empty() {
            // First attack - any card is valid
            if let Some(card) = attacker.remove_card(card_idx) {
                info!(
                    "First attack: Player {} played {}",
                    self.current_attacker, card
                );
                self.table_cards.push((card, None));
                self.game_phase = GamePhase::Defense;
                return Ok(());
            } else {
                warn!(
                    "Invalid card index {} for attacker with {} cards",
                    card_idx,
                    attacker.hand_size()
                );
            }
        } else {
            // Additional attacks - must match a rank on the table
            let card = match attacker.hand().get(card_idx) {
                Some(c) => *c,
                None => {
                    warn!(
                        "Invalid card index {} for attacker with {} cards",
                        card_idx,
                        attacker.hand_size()
                    );
                    return Err("Invalid card index");
                }
            };

            // Check if the rank is already on the table
            let is_valid = self.table_cards.iter().any(|(attack, defense)| {
                attack.rank == card.rank || defense.map_or(false, |d| d.rank == card.rank)
            });

            if is_valid {
                let card = attacker.remove_card(card_idx).unwrap();
                info!(
                    "Additional attack: Player {} played {}",
                    self.current_attacker, card
                );
                self.table_cards.push((card, None));
                self.game_phase = GamePhase::Defense;
                return Ok(());
            } else {
                warn!("Invalid attack card: rank doesn't match any card on the table");
                return Err("Card rank does not match any card on the table");
            }
        }

        warn!("Invalid card index {} for attack", card_idx);
        Err("Invalid card index")
    }

    pub fn defend(&mut self, card_idx: usize) -> Result<(), &'static str> {
        if self.game_phase != GamePhase::Defense {
            warn!(
                "Defense attempt outside defense phase: {:?}",
                self.game_phase
            );
            return Err("Not in defense phase");
        }

        let defender = &mut self.players[self.current_defender];
        debug!(
            "Player {} attempting to defend with card index {}",
            self.current_defender, card_idx
        );

        if defender.hand_size() == 0 {
            warn!("Defender has no cards to defend with");
            return Err("No cards to defend with");
        }

        if card_idx >= defender.hand_size() {
            warn!(
                "Invalid card index {} for defender with {} cards",
                card_idx,
                defender.hand_size()
            );
            return Err("Invalid card index");
        }

        // Find the last undefended attack
        let attack_idx = match self
            .table_cards
            .iter()
            .position(|(_, defense)| defense.is_none())
        {
            Some(idx) => idx,
            None => {
                warn!("No undefended attacks found to defend against");
                return Err("No attacks to defend against");
            }
        };

        let attacking_card = self.table_cards[attack_idx].0;
        debug!(
            "Defending against attacking card {} at table position {}",
            attacking_card, attack_idx
        );

        let card = match defender.hand().get(card_idx) {
            Some(c) => *c,
            None => {
                warn!(
                    "Invalid card index {} for defender with {} cards",
                    card_idx,
                    defender.hand_size()
                );
                return Err("Invalid card index");
            }
        };

        debug!(
            "Attempting to defend with card {} against {}",
            card, attacking_card
        );

        // Check if the defending card can beat the attacking card
        if let Some(trump_suit) = self.trump_suit {
            if card.can_beat(&attacking_card, trump_suit) {
                let card = defender.remove_card(card_idx).unwrap();
                info!(
                    "Defense: Player {} used {} to beat {}",
                    self.current_defender, card, attacking_card
                );
                self.table_cards[attack_idx].1 = Some(card);

                // Check if all attacks are defended
                let all_defended = !self
                    .table_cards
                    .iter()
                    .any(|(_, defense)| defense.is_none());

                if all_defended {
                    debug!("All attacks defended");

                    // Move defended cards to discard pile
                    let card_count = self.table_cards.len();
                    debug!("Moving {} defended card pairs to discard pile", card_count);

                    let mut cards_to_discard = Vec::new();
                    for (attack, defense) in self.table_cards.drain(..) {
                        cards_to_discard.push(attack);
                        if let Some(def_card) = defense {
                            cards_to_discard.push(def_card);
                        }
                    }

                    debug!("Discarding cards: {:?}", cards_to_discard);
                    self.discard_pile.extend(cards_to_discard);
                    debug!("Discard pile now has {} cards", self.discard_pile.len());

                    // Reset the table
                    self.table_cards = Vec::new();
                    debug!("Table cleared after successful defense");

                    // Immediately make the defender the new attacker after successful defense
                    let prev_attacker = self.current_attacker;
                    let prev_defender = self.current_defender;
                    
                    // Swap roles: defender becomes attacker
                    self.current_attacker = prev_defender;
                    self.current_defender = (self.current_attacker + 1) % self.players.len();
                    
                    info!("After successful defense: player {} becomes attacker, player {} becomes defender",
                          self.current_attacker, self.current_defender);
                    
                    // Check if we need to draw cards before changing phase
                    let need_to_draw = (self.players[prev_attacker].hand_size() < 6 || 
                                       self.players[prev_defender].hand_size() < 6) && 
                                       !self.deck.is_empty();
                    
                    if need_to_draw {
                        info!("Moving to drawing phase after successful defense");
                        self.game_phase = GamePhase::Drawing;
                    } else {
                        // Skip drawing phase, go directly to attack
                        info!("Moving directly to attack phase with new attacker - no cards need to be drawn");
                        self.game_phase = GamePhase::Attack;
                    }
                    
                    // Check for empty hands/game over condition
                    self.check_game_over();
                } else {
                    debug!("More undefended attacks remain");
                }

                return Ok(());
            } else {
                warn!(
                    "Card {} cannot beat attacking card {}",
                    card, attacking_card
                );
                return Err("Card cannot beat the attacking card");
            }
        }

        warn!("No trump suit defined");
        Err("No trump suit defined")
    }

    pub fn take_cards(&mut self) -> Result<(), &'static str> {
        if self.game_phase != GamePhase::Defense {
            warn!(
                "Take cards attempt outside defense phase: {:?}",
                self.game_phase
            );
            return Err("Not in defense phase");
        }

        if self.table_cards.is_empty() {
            warn!("No cards on table to take");
            return Err("No cards on table to take");
        }

        let defender = &mut self.players[self.current_defender];
        let card_count = self.table_cards.iter().count();
        info!(
            "Player {} taking {} cards from table",
            self.current_defender, card_count
        );

        // Collect all cards from the table
        let mut cards_to_take = Vec::new();
        for (attack, defense) in &self.table_cards {
            cards_to_take.push(*attack);
            if let Some(card) = defense {
                cards_to_take.push(*card);
            }
        }

        debug!("Total cards taken: {}", cards_to_take.len());
        debug!("Cards being taken: {:?}", cards_to_take);
        defender.add_cards(cards_to_take);
        self.table_cards.clear();

        // Move to drawing phase
        self.game_phase = GamePhase::Drawing;
        debug!("Game phase changed to Drawing after taking cards");

        Ok(())
    }

    pub fn draw_cards(&mut self) {
        if self.game_phase != GamePhase::Drawing {
            debug!("Called draw_cards but not in Drawing phase (current phase: {:?})", self.game_phase);
            return;
        }
        
        // Increment stuck counter to detect infinite loops
        unsafe {
            STUCK_COUNTER += 1;
            if STUCK_COUNTER > 5 {
                warn!("Drawing phase appears stuck! Counter: {}", STUCK_COUNTER);
                warn!("Emergency action: Forcing transition to Attack phase");
                
                // Reset game phase and counter
                self.game_phase = GamePhase::Attack;
                STUCK_COUNTER = 0;
                
                // Clear the table if needed
                if !self.table_cards.is_empty() {
                    warn!("Clearing {} cards from table due to stuck state", self.table_cards.len() * 2);
                    self.discard_pile.extend(self.table_cards.drain(..).flat_map(|(a, d)| {
                        let mut cards = vec![a];
                        if let Some(def) = d {
                            cards.push(def);
                        }
                        cards
                    }));
                }
                
                debug!("Game state reset to Attack phase with Attacker: {}, Defender: {}", 
                    self.current_attacker, self.current_defender);
                return;
            }
            debug!("Draw cards called, stuck counter: {}", STUCK_COUNTER);
        }
        
        // Early return if there are no players who need cards
        let players_need_cards = self.players.iter().any(|p| p.hand_size() < 6 && !self.deck.is_empty());
        if !players_need_cards {
            debug!("No players need cards and/or deck is empty, skipping drawing phase");
            self.game_phase = GamePhase::Attack;
            unsafe { STUCK_COUNTER = 0; }
            return;
        }

        debug!("Drawing cards for players who have less than 6 cards");
        let mut cards_drawn = false;
        
        // Drawing logic - first attacker draws, then defender, then others
        if !self.deck.is_empty() {
            debug!("Deck has {} cards remaining, drawing cards", self.deck.remaining());
            let player_count = self.players.len();
            let mut drawing_order = VecDeque::new();

            // Start with attacker
            let mut idx = self.current_attacker;
            for _ in 0..player_count {
                drawing_order.push_back(idx);
                idx = (idx + 1) % player_count;
            }

            // Draw cards to bring each hand back to 6
            trace!("Drawing order: {:?}", drawing_order);
            
            while let Some(player_idx) = drawing_order.pop_front() {
                let player = &mut self.players[player_idx];
                let cards_needed = 6usize.saturating_sub(player.hand_size());

                if cards_needed > 0 && !self.deck.is_empty() {
                    debug!(
                        "Player {} drawing {} cards to replenish hand",
                        player_idx, cards_needed
                    );
                    let new_cards = self.deck.deal(cards_needed);
                    if !new_cards.is_empty() {
                        cards_drawn = true;
                    }
                    debug!("Player {} drew cards: {:?}", player_idx, new_cards);
                    player.add_cards(new_cards);
                } else if cards_needed > 0 {
                    debug!("Player {} needs {} cards but deck is empty", player_idx, cards_needed);
                } else {
                    debug!("Player {} already has full hand", player_idx);
                }
            }
            
            // If no cards were drawn at all, log it
            if !cards_drawn {
                info!("No cards were drawn by any player");
            }

            // Check if any player has run out of cards and the game is over
            self.check_game_over();
            debug!("After drawing, checking game phase: {:?}", self.game_phase);

            if self.game_phase != GamePhase::GameOver {
                // Only change attacker/defender if the table is NOT empty
                // If table is empty, the defender already became the attacker in the defend method
                if !self.table_cards.is_empty() {
                    let prev_attacker = self.current_attacker;
                    let prev_defender = self.current_defender;

                    // If the defender took cards, they're skipped
                    info!("Defender took cards, is skipped for next attack");
                    self.current_attacker = (self.current_defender + 1) % self.players.len();
                    self.current_defender = (self.current_attacker + 1) % self.players.len();
                    
                    debug!(
                        "After player rotation, current attacker: {}, current defender: {}",
                        self.current_attacker, self.current_defender
                    );
                    info!(
                        "Next round: attacker {} (was {}), defender {} (was {})",
                        self.current_attacker, prev_attacker, self.current_defender, prev_defender
                    );
                } else {
                    debug!("Table already empty, keeping the current attacker/defender roles");
                }

                // Set the game phase back to Attack
                debug!("Changing game phase from Drawing to Attack");
                self.game_phase = GamePhase::Attack;
            } else {
                debug!("Game over detected during draw phase");
            }
        } else {
            debug!("Deck is empty, skipping drawing phase");
            // Check for game over condition
            self.check_game_over();
            
            if self.game_phase != GamePhase::GameOver {
                debug!("Changing phase to Attack since deck is empty");
                self.game_phase = GamePhase::Attack;
            }
        }

        // At the end of draw_cards, reset the stuck counter if we successfully transitioned
        if self.game_phase == GamePhase::Attack || self.game_phase == GamePhase::GameOver {
            unsafe { STUCK_COUNTER = 0; }
            debug!("Reset stuck counter - phase is now {:?}", self.game_phase);
        }
    }

    fn check_game_over(&mut self) {
        if self.deck.is_empty() {
            debug!("Deck is empty, checking for winners");
            for (idx, player) in self.players.iter().enumerate() {
                if player.is_empty_hand() {
                    info!(
                        "Game over! Player {} ({}) is the winner!",
                        idx,
                        player.name()
                    );
                    self.set_winner(idx);
                    return;
                }
            }
            debug!("No winner yet, all players still have cards");
        }
    }

    // Getters
    pub fn players(&self) -> &[Player] {
        &self.players
    }

    pub fn deck(&self) -> &Deck {
        &self.deck
    }

    pub fn trump_suit(&self) -> Option<Suit> {
        self.trump_suit
    }

    pub fn table_cards(&self) -> &[(Card, Option<Card>)] {
        &self.table_cards
    }

    pub fn current_attacker(&self) -> usize {
        self.current_attacker
    }

    pub fn current_defender(&self) -> usize {
        self.current_defender
    }

    pub fn game_phase(&self) -> &GamePhase {
        &self.game_phase
    }

    pub fn winner(&self) -> Option<usize> {
        self.winner
    }

    #[allow(dead_code)]
    pub fn discard_pile(&self) -> &[Card] {
        &self.discard_pile
    }

    // Helper method to force the game state into Attack phase
    // Only used as an emergency measure to prevent freezes
    pub fn force_attack_phase(mut state: GameState) -> GameState {
        warn!("EMERGENCY: Forcing game to Attack phase");
        state.game_phase = GamePhase::Attack;
        
        // Clear the table if needed
        if !state.table_cards.is_empty() {
            let discarded = state.table_cards.len();
            warn!("Emergency discard: {} card pairs moved to discard pile", discarded);
            
            // Move cards to discard pile
            let mut cards_to_discard = Vec::new();
            for (attack, defense) in state.table_cards.drain(..) {
                cards_to_discard.push(attack);
                if let Some(def_card) = defense {
                    cards_to_discard.push(def_card);
                }
            }
            
            state.discard_pile.extend(cards_to_discard);
        }
        
        debug!("Game forced to Attack phase - Attacker: {}, Defender: {}", 
            state.current_attacker, state.current_defender);
            
        state
    }

    // Set the winner and update game phase
    pub fn set_winner(&mut self, player_idx: usize) {
        if player_idx < self.players.len() {
            info!("Setting player {} ({}) as the winner", player_idx, self.players[player_idx].name());
            self.winner = Some(player_idx);
            self.game_phase = GamePhase::GameOver;
        } else {
            warn!("Attempted to set invalid player index {} as winner", player_idx);
        }
    }
}

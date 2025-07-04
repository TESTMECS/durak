use super::card::{Card, Suit};
use super::deck::Deck;
use super::player::{Player, PlayerType};
use std::collections::VecDeque;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GamePhase {
    Setup,
    Attack,
    Defense,
    Drawing,
    GameOver,
}
impl Display for GamePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub players: Vec<Player>,
    pub deck: Deck,
    pub discard_pile: Vec<Card>,
    pub table_cards: Vec<(Card, Option<Card>)>, // (attacking card, defending card)
    pub current_attacker: usize,
    pub current_defender: usize,
    pub trump_suit: Option<Suit>,
    pub game_phase: GamePhase,
    pub winner: Option<usize>,
    pub stuck_counter: usize, // Add this field to track stuck states
}

impl GameState {
    /// Constructor for the GameState struct
    /// Called by the `app_core.rs` when creating a new game.
    /// Important initalitzations are `Deck::new()` and `GamePhase::Setup`
    pub fn new() -> Self {
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
            stuck_counter: 0, // Initialize counter
        }
    }

    /// Adds a new player to the `players` vector of the GameState
    pub fn add_player(&mut self, name: String, player_type: PlayerType) {
        self.players.push(Player::new(name, player_type));
    }
    /// Sets up the game by creating a new deck, shuffling it, and dealing 6 cards to each player.
    /// The player with the lowest trump card delt is determined as the starting attacker.
    pub fn setup_game(&mut self) {
        self.deck = Deck::new();
        self.deck.shuffle();
        self.trump_suit = self.deck.trump_suit();
        for player in &mut self.players {
            let cards = self.deck.deal(6);
            player.add_cards(cards);
        }
        self.determine_first_player();
        self.current_defender = (self.current_attacker + 1) % self.players.len();
        self.game_phase = GamePhase::Attack;
        self.stuck_counter = 0; // Reset stuck counter when starting a new game
    }
    /// The player with the lowest trump card is determined as the starting attacker.
    /// If no trump suit is present, the player is chosen.
    fn determine_first_player(&mut self) {
        if let Some(trump_suit) = self.trump_suit {
            // Find the player with the lowest trump card
            let mut lowest_player = 0;
            let mut lowest_rank = None;
            for (i, player) in self.players.iter().enumerate() {
                if let Some((_, card)) = player.get_lowest_trump(trump_suit) {
                    if lowest_rank.is_none() || card.rank < lowest_rank.unwrap() {
                        lowest_rank = Some(card.rank);
                        lowest_player = i;
                    }
                }
            }
            // If someone has a trump card, they go first
            if lowest_rank.is_some() {
                self.current_attacker = lowest_player;
                return;
            }
        }
        // If no one has a trump card or there's no trump suit, just start with player 0
        self.current_attacker = 0;
    }
    /// General attack logic
    pub fn attack(&mut self, card_idx: usize, player_idx: usize) -> Result<(), &'static str> {
        let attacker = &mut self.players[player_idx];
        if let Some(card) = attacker.remove_card(card_idx) {
            self.table_cards.push((card, None));
            // Transition to Defense phase after successful attack
            self.game_phase = GamePhase::Defense;
            // Set the attacker and defender roles properly
            self.current_attacker = player_idx;
            self.current_defender = (player_idx + 1) % self.players.len();
            return Ok(());
        }
        Err("Invalid card index")
    }

    /// Handle passing an attack to the next player if cards are the same rank
    pub fn pass_attack(&mut self, card_idx: usize, _attack_idx: usize) -> Result<(), &'static str> {
        let defender = &mut self.players[self.current_defender];
        // Remove the card from defender's hand
        if let Some(card) = defender.remove_card(card_idx) {
            // Add a new attack card to the table
            self.table_cards.push((card, None));
            // Swap the roles - the current defender becomes the attacker
            let old_defender = self.current_defender;
            self.current_attacker = old_defender;
            // The original attacker becomes the defender
            self.current_defender = (old_defender + 1) % self.players.len();
            // Stay in Defense phase
            self.game_phase = GamePhase::Defense;
            return Ok(());
        }
        Err("Failed to remove card from hand during pass")
    }
    /// General defense logic
    pub fn defend(&mut self, card_idx: usize) -> Result<(), &'static str> {
        // Find the first undefended attack card
        let undefended_idx = self
            .table_cards
            .iter()
            .position(|(_, defense)| defense.is_none());
        if let Some(attack_idx) = undefended_idx {
            let defender = &mut self.players[self.current_defender];
            if card_idx >= defender.hand().len() {
                return Err("Invalid card index");
            }
            let defense_card = defender.hand()[card_idx];
            let attack_card = self.table_cards[attack_idx].0;
            // First check if this is a pass (podkidnoy variant)
            // Check for same rank (passing condition)
            if defense_card.can_pass(&attack_card) {
                // This is a pass - handle differently from a regular defense
                return self.pass_attack(card_idx, attack_idx);
            }
            // Check if defense is valid
            let is_valid = if let Some(trump) = self.trump_suit {
                if attack_card.suit == trump {
                    // If attacking with trump, must defend with higher trump
                    defense_card.suit == trump && defense_card.rank > attack_card.rank
                } else if defense_card.suit == trump {
                    // Trump can beat any non-trump
                    true
                } else {
                    // Same suit, higher rank
                    defense_card.suit == attack_card.suit && defense_card.rank > attack_card.rank
                }
            } else {
                // No trump suit - just check for same suit and higher rank
                defense_card.suit == attack_card.suit && defense_card.rank > attack_card.rank
            };
            if is_valid {
                // Remove the card from defender's hand
                if let Some(card) = defender.remove_card(card_idx) {
                    // Add as defense card
                    self.table_cards[attack_idx].1 = Some(card);
                    return Ok(());
                }
                Err("Failed to remove card from hand")
            } else {
                Err("Invalid defense - card cannot beat the attack")
            }
        } else {
            Err("No undefended attacks to defend against")
        }
    }
    /// Checks defense then puts cards into the table.
    pub fn discard_cards(&mut self, cards: Vec<(usize, Card)>) {
        cards.iter().for_each(|(idx, card)| {
            self.table_cards[*idx].1 = Some(*card); // add card to defended table
        });
        // Check if all attacks are defended
        let all_defended = !self
            .table_cards
            .iter()
            .any(|(_, defense)| defense.is_none());
        if all_defended {
            // All attacks successfully defended
            // Move cards from table to discard pile
            let mut cards_to_discard = Vec::new();
            for (attack, defense) in std::mem::take(&mut self.table_cards) {
                cards_to_discard.push(attack);
                if let Some(def_card) = defense {
                    cards_to_discard.push(def_card);
                }
            }
            self.discard_pile.extend(cards_to_discard);
            // Successful defense - swap attacker and defender roles
            // After successful defense, defender becomes new attacker
            let old_defender = self.current_defender;
            self.current_attacker = old_defender;
            self.current_defender = (old_defender + 1) % self.players.len();
            // Move to drawing phase
            self.game_phase = GamePhase::Drawing;
        }
    }
    /// Take cards from the table and put them into the player's hand.
    pub fn take_cards(&mut self) -> Result<(), &'static str> {
        assert!(self.game_phase == GamePhase::Defense);
        if self.table_cards.is_empty() {
            return Err("No cards on table to take");
        }
        let defender = &mut self.players[self.current_defender];
        let table_cards = std::mem::take(&mut self.table_cards);
        let mut cards_to_take = Vec::new();
        for (attack, defense) in table_cards {
            cards_to_take.push(attack);
            if let Some(card) = defense {
                cards_to_take.push(card);
            }
        }
        // adding cards to defender hand.
        defender.add_cards(cards_to_take);
        // Move to drawing phase
        self.game_phase = GamePhase::Drawing;
        Ok(())
    }
    /// General Draw cards logic.
    pub fn draw_cards(&mut self) {
        if self.game_phase != GamePhase::Drawing {
            return;
        }
        // Increment stuck counter to detect infinite loops
        self.stuck_counter += 1;
        if self.stuck_counter > 5 {
            // Reset game phase and counter
            self.game_phase = GamePhase::Attack;
            self.stuck_counter = 0;
            // Clear the table if needed
            if !self.table_cards.is_empty() {
                self.discard_pile
                    .extend(self.table_cards.drain(..).flat_map(|(a, d)| {
                        let mut cards = vec![a];
                        if let Some(def) = d {
                            cards.push(def);
                        }
                        cards
                    }));
            }
            return;
        }
        // Early return if there are no players who need cards
        let players_need_cards = self
            .players
            .iter()
            .any(|p| p.hand_size() < 6 && !self.deck.is_empty());
        if !players_need_cards {
            self.game_phase = GamePhase::Attack;
            self.stuck_counter = 0;
            return;
        }
        // Drawing logic - first attacker draws, then defender, then others
        if !self.deck.is_empty() {
            let player_count = self.players.len();
            let mut drawing_order = VecDeque::new();
            // Start with attacker
            let mut idx = self.current_attacker;
            for _ in 0..player_count {
                drawing_order.push_back(idx);
                idx = (idx + 1) % player_count;
            }
            // Draw cards to bring each hand back to 6
            while let Some(player_idx) = drawing_order.pop_front() {
                let player = &mut self.players[player_idx];
                let cards_needed = 6usize.saturating_sub(player.hand_size());
                if cards_needed > 0 && !self.deck.is_empty() {
                    let new_cards = self.deck.deal(cards_needed);
                    // No need to track if cards are drawn
                    player.add_cards(new_cards);
                }
            }
            // Check if any player has run out of cards and the game is over
            self.check_game_over();
            if self.game_phase != GamePhase::GameOver {
                // Only change attacker/defender if the table is NOT empty
                // If table is empty, the defender already became the attacker in the defend method
                if !self.table_cards.is_empty() {
                    let _prev_attacker = self.current_attacker;
                    let _prev_defender = self.current_defender;
                    // If the defender took cards, they're skipped
                    self.current_attacker = (self.current_defender + 1) % self.players.len();
                    self.current_defender = (self.current_attacker + 1) % self.players.len();
                }
                // Set the game phase back to Attack
                self.game_phase = GamePhase::Attack;
            }
        } else {
            // Check for game over condition
            self.check_game_over();
            if self.game_phase != GamePhase::GameOver {
                self.game_phase = GamePhase::Attack;
            }
        }
        // At the end of draw_cards, reset the stuck counter if we successfully transitioned
        if self.game_phase == GamePhase::Attack || self.game_phase == GamePhase::GameOver {
            self.stuck_counter = 0;
        }
    }
    /// Check game over logic.
    pub fn check_game_over(&mut self) -> bool {
        if self.deck.is_empty() {
            let mut players_with_cards = 0;
            let mut last_player_with_cards = None;
            // Count players with cards and remember the last one with cards
            for (idx, player) in self.players.iter().enumerate() {
                if !player.is_empty_hand() {
                    players_with_cards += 1;
                    last_player_with_cards = Some(idx);
                }
            }
            // Game ends when only one player (or zero) has cards left
            if players_with_cards <= 1 {
                // If there's one player with cards, they're the "durak" (loser)
                // In Durak, the winner is the player who gets rid of cards first
                if players_with_cards == 1 {
                    if let Some(loser_idx) = last_player_with_cards {
                        // In a 2-player game, if player 1 is the loser, then player 0 is the winner
                        let winner_idx = if loser_idx == 1 { 0 } else { 1 };
                        self.winner = Some(winner_idx);
                        self.game_phase = GamePhase::GameOver;
                    }
                } else if players_with_cards == 0 {
                    // Draw or edge case - no real winner in Durak, but let's handle it
                    self.game_phase = GamePhase::GameOver;
                }
                return true;
            }
            false // More than one player has cards
        } else {
            false // Deck is not empty
        }
    }
    // Getters
    pub fn players(&self) -> &[Player] {
        &self.players
    }
    pub fn players_mut(&mut self) -> &mut Vec<Player> {
        &mut self.players
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
    /// Helper method to force the game state into Attack phase
    /// Only used as an emergency measure to prevent freezes
    pub fn force_attack_phase(mut state: GameState) -> GameState {
        //warn!("EMERGENCY: Forcing game to Attack phase");
        state.game_phase = GamePhase::Attack;
        state.stuck_counter = 0; // Reset stuck counter when forcing attack phase
                                 // Clear the table if needed
        if !state.table_cards.is_empty() {
            // No need to track the number of discarded cards
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
        state
    }
    /// Helper method to set the game to defense phase
    pub fn set_phase_to_defense(&mut self, attacker_idx: usize, defender_idx: usize) {
        self.game_phase = GamePhase::Defense;
        self.current_attacker = attacker_idx;
        self.current_defender = defender_idx;
    }
}

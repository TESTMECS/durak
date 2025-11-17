use super::card::{Card, Suit};
use super::deck::Deck;
use super::player::{Player, PlayerType};
use arrayvec::ArrayVec;
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
    pub players: [Player; 2],
    pub deck: Deck,
    pub discard_pile: ArrayVec<Card, 36>,
    pub table_cards: ArrayVec<(Card, Option<Card>), 12>, // (attacking card, defending card)
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
            players: [
                Player::new("Player".to_string(), PlayerType::Human),
                Player::new("Computer".to_string(), PlayerType::Computer),
            ],
            deck: Deck::new(),
            discard_pile: ArrayVec::new(),
            table_cards: ArrayVec::new(),
            current_attacker: 0,
            current_defender: 0,
            trump_suit: None,
            game_phase: GamePhase::Setup,
            winner: None,
            stuck_counter: 0, // Initialize counter
        }
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
        for (idx, card) in cards {
            self.table_cards[idx].1 = Some(card); // add card to defended table
        }
        // Check if all attacks are defended
        let all_defended = !self
            .table_cards
            .iter()
            .any(|(_, defense)| defense.is_none());
        if !all_defended {
            return;
        }
        for (a, d) in self.table_cards.drain(..) {
            self.discard_pile.push(a);
            if let Some(x) = d {
                self.discard_pile.push(x);
            }
        }
        let old_defender = self.current_defender;
        self.current_attacker = old_defender;
        self.current_defender = (old_defender + 1) % self.players.len();
        self.game_phase = GamePhase::Drawing;
    }
    /// Take cards from the table and put them into the player's hand.
    pub fn take_cards(&mut self) -> Result<(), &'static str> {
        assert!(self.game_phase == GamePhase::Defense);
        if self.table_cards.is_empty() {
            return Err("No cards on table to take");
        }
        let defender = &mut self.players[self.current_defender];
        let mut taken = Vec::with_capacity(self.table_cards.len() * 2);
        for (a, d) in self.table_cards.drain(..) {
            taken.push(a);
            if let Some(x) = d {
                taken.push(x);
            }
        }
        defender.add_cards(taken);
        self.game_phase = GamePhase::Drawing;
        Ok(())
    }
    /// General Draw cards logic.
    pub fn draw_cards(&mut self) {
        if self.game_phase != GamePhase::Drawing {
            return;
        }
        self.stuck_counter += 1;
        if self.stuck_counter > 5 {
            self.game_phase = GamePhase::Attack;
            self.stuck_counter = 0;
            for (a, d) in self.table_cards.drain(..) {
                self.discard_pile.push(a);
                if let Some(x) = d {
                    self.discard_pile.push(x);
                }
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
        let n = self.players.len();
        let mut idx = self.current_attacker;
        for _ in 0..n {
            if self.deck.is_empty() {
                break;
            }
            let p = &mut self.players[idx];
            let need = 6usize.saturating_sub(p.hand_size());
            if need > 0 {
                let new_cards = self.deck.deal(need);
                p.add_cards(new_cards);
            }
            idx = (idx + 1) % n;
        }
        self.check_game_over();
        if self.game_phase != GamePhase::GameOver {
            // If table not empty defender took cards → skip them
            if !self.table_cards.is_empty() {
                self.current_attacker = (self.current_defender + 1) % n;
                self.current_defender = (self.current_attacker + 1) % n;
            }

            self.game_phase = GamePhase::Attack;
        }

        if matches!(self.game_phase, GamePhase::Attack | GamePhase::GameOver) {
            self.stuck_counter = 0;
        }
    }
    /// Check game over logic.
    pub fn check_game_over(&mut self) -> bool {
        if !self.deck.is_empty() {
            return false;
        }
        let mut count = 0;
        let mut last = None;
        for (i, p) in self.players.iter().enumerate() {
            if !p.is_empty_hand() {
                count += 1;
                last = Some(i);
            }
        }
        if count <= 1 {
            match (count, last) {
                (1, Some(loser)) => {
                    let winner = if loser == 1 { 0 } else { 1 };
                    self.winner = Some(winner);
                }
                _ => {}
            }
            self.game_phase = GamePhase::GameOver;
            return true;
        }
        false
    }
    // Getters
    #[inline]
    pub fn players(&self) -> &[Player] {
        &self.players
    }
    #[inline]
    pub fn players_mut(&mut self) -> &mut [Player; 2] {
        &mut self.players
    }
    #[inline]
    pub fn deck(&self) -> &Deck {
        &self.deck
    }
    #[inline]
    pub fn trump_suit(&self) -> Option<Suit> {
        self.trump_suit
    }
    #[inline]
    pub fn table_cards(&self) -> &[(Card, Option<Card>)] {
        &self.table_cards
    }
    #[inline]
    pub fn current_attacker(&self) -> usize {
        self.current_attacker
    }
    #[inline]
    pub fn current_defender(&self) -> usize {
        self.current_defender
    }
    #[inline]
    pub fn game_phase(&self) -> &GamePhase {
        &self.game_phase
    }
    #[inline]
    pub fn winner(&self) -> Option<usize> {
        self.winner
    }
    #[inline]
    #[allow(dead_code)]
    pub fn discard_pile(&self) -> &[Card] {
        &self.discard_pile
    }
    /// Helper method to force the game state into Attack phase
    /// Only used as an emergency measure to prevent freezes
    pub fn force_attack_phase(mut state: GameState) -> GameState {
        state.game_phase = GamePhase::Attack;
        state.stuck_counter = 0;
        if !state.table_cards.is_empty() {
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
    #[inline]
    pub fn set_phase_to_defense(&mut self, attacker_idx: usize, defender_idx: usize) {
        self.game_phase = GamePhase::Defense;
        self.current_attacker = attacker_idx;
        self.current_defender = defender_idx;
    }
}

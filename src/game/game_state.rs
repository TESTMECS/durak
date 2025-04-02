use super::card::{Card, Suit};
use super::deck::Deck;
use super::player::{Player, PlayerType};
use crate::ui::debug_overlay::debug;
use std::collections::VecDeque;
use std::fmt::Display;

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
impl Display for GamePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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
        self.players.push(Player::new(name, player_type));
    }

    pub fn setup_game(&mut self) {
        assert!(
            self.players.len() >= 2,
            "Need at least 2 players to start the game"
        );
        // Initialize and shuffle the deck
        self.deck = Deck::new();
        self.deck.shuffle();
        self.trump_suit = self.deck.trump_suit();
        // Deal cards to players (6 cards each)
        for player in &mut self.players {
            let cards = self.deck.deal(6);
            player.add_cards(cards);
        }
        // Determine who goes first (player with lowest trump card)
        self.determine_first_player();
        self.current_defender = (self.current_attacker + 1) % self.players.len();
        self.game_phase = GamePhase::Attack;
    }

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

    pub fn attack(&mut self, card_idx: usize, is_multi: bool) -> Result<(), &'static str> {
        if self.game_phase != GamePhase::Attack {
            return Err("Not in attack phase");
        }
        // is this wrong?
        let attacker = &mut self.players[self.current_attacker];
        if card_idx >= attacker.hand_size() {
            return Err("Invalid card index");
        }
        // If attacking?
        if self.table_cards.is_empty() {
            // remove card from attacker
            if let Some(card) = attacker.remove_card(card_idx) {
                // push it onto the table
                self.table_cards.push((card, None));
                // change the game state.
                if !is_multi {
                    self.game_phase = GamePhase::Defense;
                }
                return Ok(());
            }
        }
        //else {
        // I feel like this should be an error. When would the game be in attack state with
        // cards on the table?
        //debug("Touching weird spot");
        //let card = match attacker.hand().get(card_idx) {
        //    Some(c) => *c,
        //    None => {
        //        return Err("Invalid card index");
        //    }
        //};
        //// Check if the rank is already on the table
        //let table_ranks: Vec<_> = self
        //    .table_cards
        //    .iter()
        //    .flat_map(|(attack, defense)| {
        //        let mut ranks = vec![attack.rank];
        //        if let Some(def) = defense {
        //            ranks.push(def.rank);
        //        }
        //        ranks
        //    })
        //    .collect();
        //let is_valid = table_ranks.contains(&card.rank);
        //if is_valid {
        //    let card = attacker.remove_card(card_idx).unwrap();
        //    self.table_cards.push((card, None));
        //    self.game_phase = GamePhase::Defense;
        //    return Ok(());
        //} else {
        //    return Err("Card rank does not match any card on the table");
        //}
        Err("Invalid card index")
    }

    pub fn defend(&mut self, card_idx: usize) -> Result<(), &'static str> {
        if self.game_phase != GamePhase::Defense {
            return Err("Not in defense phase");
        }
        let defender = &mut self.players[self.current_defender];
        if defender.hand_size() == 0 {
            return Err("No cards to defend with");
        }
        if card_idx >= defender.hand_size() {
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
                debug("No undefended attacks found to defend against");
                return Err("No attacks to defend against");
            }
        };

        let attacking_card = self.table_cards[attack_idx].0;
        debug(format!(
            "Defending against attacking card {} at table position {}",
            attacking_card, attack_idx
        ));
        let card = match defender.hand().get(card_idx) {
            Some(c) => *c,
            None => {
                debug(format!(
                    "Invalid card index: {} (hand size: {})",
                    card_idx,
                    defender.hand_size()
                ));
                return Err("Invalid card index");
            }
        };

        debug(format!(
            "Attempting to defend with card {} against {}",
            card, attacking_card
        ));

        // Check if the defending card can beat the attacking card
        if let Some(trump_suit) = self.trump_suit {
            debug(format!("Trump suit: {:?}", trump_suit));
            let can_beat = card.can_beat(&attacking_card, trump_suit);
            debug(format!(
                "Can {} beat {}: {}",
                card, attacking_card, can_beat
            ));

            if can_beat {
                let card = defender.remove_card(card_idx).unwrap();
                debug(format!(
                    "Defense: Player {} used {} to beat {}",
                    self.current_defender, card, attacking_card
                ));

                self.table_cards[attack_idx].1 = Some(card);

                // Check if all attacks are defended
                let all_defended = !self
                    .table_cards
                    .iter()
                    .any(|(_, defense)| defense.is_none());

                debug(format!("All attacks defended: {}", all_defended));
                if all_defended {
                    // Move defended cards to discard pile
                    let _card_count = self.table_cards.len();
                    let cards_to_discard: Vec<Card> = self
                        .table_cards
                        .drain(..)
                        .flat_map(|(a, d)| vec![a, d.unwrap()])
                        .collect();
                    self.discard_pile.extend(cards_to_discard.iter());

                    // Clear table
                    self.table_cards.clear();

                    // Immediately make the defender the new attacker after successful defense
                    let prev_attacker = self.current_attacker;
                    let prev_defender = self.current_defender;

                    // Swap roles: defender becomes attacker
                    self.current_attacker = prev_defender;
                    self.current_defender = (self.current_attacker + 1) % self.players.len();

                    // Check if we need to draw cards before changing phase
                    let need_to_draw = (self.players[prev_attacker].hand_size() < 6
                        || self.players[prev_defender].hand_size() < 6)
                        && !self.deck.is_empty();

                    if need_to_draw {
                        self.game_phase = GamePhase::Drawing;
                    } else {
                        // Skip drawing phase, go directly to attack
                        self.game_phase = GamePhase::Attack;
                    }

                    // Check for empty hands/game over condition
                    self.check_game_over();
                }
                return Ok(());
            } else {
                return Err("Card cannot beat the attacking card");
            }
        }
        Err("No trump suit defined")
    }

    pub fn take_cards(&mut self) -> Result<(), &'static str> {
        if self.game_phase != GamePhase::Defense {
            debug(format!(
                "Take cards attempt outside defense phase: {:?}",
                self.game_phase
            ));
            return Err("Not in defense phase");
        }

        debug(format!(
            "Player {} attempting to take cards",
            self.current_defender
        ));
        debug(format!("Current table cards: {:?}", self.table_cards));

        if self.table_cards.is_empty() {
            debug("No cards on table to take");
            return Err("No cards on table to take");
        }

        let defender = &mut self.players[self.current_defender];
        let defender_hand_size_before = defender.hand_size();
        debug(format!(
            "Defender hand size before taking: {}",
            defender_hand_size_before
        ));

        let table_cards = std::mem::take(&mut self.table_cards);
        let card_count = table_cards.len();
        debug(format!(
            "Player {} taking {} card pairs from table",
            self.current_defender, card_count
        ));

        // Collect all cards from the table
        let mut cards_to_take = Vec::new();
        for (attack, defense) in table_cards {
            debug(format!("Taking attack card: {}", attack));
            cards_to_take.push(attack);
            if let Some(card) = defense {
                debug(format!("Taking defense card: {}", card));
                cards_to_take.push(card);
            }
        }

        debug(format!("Total cards taken: {}", cards_to_take.len()));
        debug(format!("Cards being taken: {:?}", cards_to_take));
        defender.add_cards(cards_to_take);

        debug(format!(
            "Defender hand size after taking: {}",
            defender.hand_size()
        ));

        // Move to drawing phase
        self.game_phase = GamePhase::Drawing;
        debug("Game phase changed to Drawing after taking cards");

        Ok(())
    }

    pub fn draw_cards(&mut self) {
        if self.game_phase != GamePhase::Drawing {
            return;
        }

        // Increment stuck counter to detect infinite loops
        unsafe {
            STUCK_COUNTER += 1;
            if STUCK_COUNTER > 5 {
                // Reset game phase and counter
                self.game_phase = GamePhase::Attack;
                STUCK_COUNTER = 0;

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
        }

        // Early return if there are no players who need cards
        let players_need_cards = self
            .players
            .iter()
            .any(|p| p.hand_size() < 6 && !self.deck.is_empty());
        if !players_need_cards {
            self.game_phase = GamePhase::Attack;
            unsafe {
                STUCK_COUNTER = 0;
            }
            return;
        }

        let mut cards_drawn = false;

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
                    if !new_cards.is_empty() {
                        cards_drawn = true;
                    }
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
            unsafe {
                STUCK_COUNTER = 0;
            }
        }
    }

    // Returns true if the game is over, false otherwise.
    // Does NOT change the game phase internally.
    pub fn check_game_over(&self) -> bool {
        if self.deck.is_empty() {
            let mut players_with_cards = 0;
            for player in self.players.iter() {
                if !player.is_empty_hand() {
                    players_with_cards += 1;
                }
            }

            // Game ends when only one player (or zero) has cards left
            if players_with_cards <= 1 {
                return true;
            }

            // Check if attacker OR defender ran out of cards simultaneously (less common)
            // This logic was slightly flawed, typically game ends when one player is out and deck is empty
            // Let's simplify: if deck is empty and <= 1 player has cards, it's over.
            // The specific winner/durak determination can happen elsewhere if needed.
            // debug!("Deck empty, but >1 players have cards. Game continues.");
            false // More than one player has cards
        } else {
            false // Deck is not empty
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
        //warn!("EMERGENCY: Forcing game to Attack phase");
        state.game_phase = GamePhase::Attack;

        // Clear the table if needed
        if !state.table_cards.is_empty() {
            let discarded = state.table_cards.len();
            //warn!(
            //"Emergency discard: {} card pairs moved to discard pile",
            //discarded
            //);

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

        //debug!(
        //"Game forced to Attack phase - Attacker: {}, Defender: {}",
        //state.current_attacker, state.current_defender
        //);

        state
    }

    // Set the winner and update game phase
    #[allow(dead_code)]
    pub fn set_winner(&mut self, player_idx: usize) {
        if player_idx < self.players.len() {
            self.winner = Some(player_idx);
            self.game_phase = GamePhase::GameOver;
        } else {
            //warn!(
            //    "Attempted to set invalid player index {} as winner",
            //    player_idx
            //);
        }
    }
}

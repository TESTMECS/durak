use crate::game::{Card, GameState};
use log::debug;
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet};
use crate::game::card::Rank;
use crate::game::card::Suit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiDifficulty {
    #[allow(dead_code)]
    Easy,
    Medium,
    #[allow(dead_code)]
    Hard,
}

// Define a strategy trait for AI behavior
trait AiStrategy {
    fn difficulty(&self) -> AiDifficulty;
    fn should_take_cards(&self, game_state: &GameState, player_idx: usize) -> bool;
    fn make_attack_move(&self, game_state: &GameState, player_idx: usize) -> Option<usize>;
    fn make_defense_move(&self, game_state: &GameState, player_idx: usize) -> Option<usize>;
}

// Implement strategies for each difficulty level
struct EasyStrategy;
struct MediumStrategy;
struct HardStrategy;

impl AiStrategy for EasyStrategy {
    fn difficulty(&self) -> AiDifficulty {
        AiDifficulty::Easy
    }

    fn should_take_cards(&self, game_state: &GameState, player_idx: usize) -> bool {
        let player = &game_state.players()[player_idx];
        let trump_suit = game_state.trump_suit();

        // Find the first undefended attack
        if let Some((attacking_card, _)) = game_state
            .table_cards()
            .iter()
            .find(|(_, defense)| defense.is_none())
        {
            // Easy AI: Take cards if *any* defense requires a trump card or a high-rank card (e.g., Ace, King)
            let can_defend_easily = player.hand().iter().any(|card| {
                if let Some(trump) = trump_suit {
                    card.can_beat(attacking_card, trump)
                        && (card.suit != trump || card.rank < Rank::Queen)
                } else {
                    false // Should have trump in defense phase
                }
            });

            if !can_defend_easily {
                // debug!(
                //     "Easy AI ({}) finds defense difficult or impossible, taking cards",
                //     player_idx
                // );
                true
            } else {
                // debug!(
                //     "Easy AI ({}) finds defense possible, will attempt",
                //     player_idx
                // );
                false
            }
        } else {
            false // No undefended attacks
        }
    }

    fn make_attack_move(&self, game_state: &GameState, player_idx: usize) -> Option<usize> {
        let player = &game_state.players()[player_idx];
        let hand = player.hand();

        if hand.is_empty() {
            return None;
        }

        if game_state.table_cards().is_empty() {
            // First attack: Play the first card in hand (simplest approach)
            Some(0)
        } else {
            // Additional attack: Find the first card matching any rank on the table
            let table_ranks: HashSet<_> = game_state
                .table_cards()
                .iter()
                .flat_map(|(attack, defense)| {
                    vec![Some(attack.rank), defense.map(|d| d.rank)]
                        .into_iter()
                        .flatten()
                })
                .collect();

            let attack_idx = hand
                .iter()
                .enumerate()
                .find(|(_, card)| table_ranks.contains(&card.rank))
                .map(|(idx, _)| idx);

            attack_idx
        }
    }

    fn make_defense_move(&self, game_state: &GameState, player_idx: usize) -> Option<usize> {
        let player = &game_state.players()[player_idx];
        let hand = player.hand();
        let trump_suit = game_state
            .trump_suit()
            .expect("Trump suit must be set during defense");

        // Find the first undefended attack
        if let Some((attacking_card, _)) = game_state
            .table_cards()
            .iter()
            .find(|(_, defense)| defense.is_none())
        {
            // Find the first card that can beat the attack
            let defense_idx = hand
                .iter()
                .enumerate()
                .find(|(_, card)| card.can_beat(attacking_card, trump_suit))
                .map(|(idx, _)| idx);

            if let Some(idx) = defense_idx {
                // debug!(
                //     "Easy AI ({}) defending with first possible card: {}",
                //     player_idx, hand[idx]
                // );
            }

            defense_idx
        } else {
            None
        }
    }
}

impl AiStrategy for MediumStrategy {
    fn difficulty(&self) -> AiDifficulty {
        AiDifficulty::Medium
    }

    fn should_take_cards(&self, game_state: &GameState, player_idx: usize) -> bool {
        // debug!("Medium AI ({}) deciding whether to take cards", player_idx);
        let player = &game_state.players()[player_idx];
        let trump_suit = game_state.trump_suit();

        // Find the first undefended attack
        if let Some((attacking_card, _)) = game_state
            .table_cards()
            .iter()
            .find(|(_, defense)| defense.is_none())
        {
            // Check if the player *can* defend
            let can_defend = player.hand().iter().any(|card| {
                trump_suit.is_some_and(|trump| card.can_beat(attacking_card, trump))
            });

            if !can_defend {
                // debug!(
                //     "Medium AI ({}) cannot defend against {}, taking cards",
                //     player_idx, attacking_card
                // );
                true // Must take if cannot defend
            } else {
                // Medium AI simple logic: if defense is possible, try to defend.
                // More complex logic could go here (e.g., evaluating card value).
                // debug!(
                //     "Medium AI ({}) can defend, will attempt defense",
                //     player_idx
                // );
                false
            }
        } else {
            // debug!(
            //     "Medium AI ({}) - no undefended attacks, not taking cards",
            //     player_idx
            // );
            false // No undefended attacks, no need to take
        }
    }

    fn make_attack_move(&self, game_state: &GameState, player_idx: usize) -> Option<usize> {
        // debug!("Medium AI ({}) deciding attack move", player_idx);
        let player = &game_state.players()[player_idx];
        let hand = player.hand();
        let trump_suit = game_state.trump_suit();

        if hand.is_empty() {
            // debug!("Medium AI ({}) has no cards to attack with", player_idx);
            return None;
        }

        if game_state.table_cards().is_empty() {
            // First attack: find the lowest rank non-trump card
            let best_card_idx = hand
                .iter()
                .enumerate()
                .filter(|(_, card)| Some(card.suit) != trump_suit)
                .min_by_key(|(_, card)| card.rank)
                .map(|(idx, _)| idx);

            if let Some(idx) = best_card_idx {
                // debug!(
                //     "Medium AI ({}) starting attack with lowest non-trump: {}",
                //     player_idx, hand[idx]
                // );
                Some(idx)
            } else {
                // Only trump cards left, play the lowest trump
                let lowest_trump_idx = hand
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, card)| card.rank)
                    .map(|(idx, _)| idx);
                if let Some(idx) = lowest_trump_idx {
                    // debug!(
                    //     "Medium AI ({}) starting attack with lowest trump: {}",
                    //     player_idx, hand[idx]
                    // );
                }
                lowest_trump_idx
            }
        } else {
            // Additional attack: find cards matching ranks on table
            let table_ranks: std::collections::HashSet<_> = game_state
                .table_cards()
                .iter()
                .flat_map(|(attack, defense)| {
                    vec![Some(attack.rank), defense.map(|d| d.rank)]
                        .into_iter()
                        .flatten()
                })
                .collect();

            let valid_attacks: Vec<(usize, Card)> = hand
                .iter()
                .enumerate()
                .filter(|(_, card)| table_ranks.contains(&card.rank))
                .map(|(idx, &card)| (idx, card))
                .collect();

            if valid_attacks.is_empty() {
                // debug!(
                //     "Medium AI ({}) has no valid additional attack cards",
                //     player_idx
                // );
                None // Pass
            } else {
                // Choose the lowest rank among valid attacks
                let (best_idx, best_card) = valid_attacks
                    .into_iter()
                    .min_by_key(|(_, card)| card.rank)
                    .unwrap(); // Safe unwrap because valid_attacks is not empty
                // debug!(
                //     "Medium AI ({}) adding attack with lowest matching rank: {}",
                //     player_idx, best_card
                // );
                Some(best_idx)
            }
        }
    }

    fn make_defense_move(&self, game_state: &GameState, player_idx: usize) -> Option<usize> {
        // debug!("Medium AI ({}) deciding defense move", player_idx);
        let player = &game_state.players()[player_idx];
        let hand = player.hand();
        let trump_suit = game_state
            .trump_suit()
            .expect("Trump suit must be set during defense");

        // Find the first undefended attack
        if let Some((attacking_card, _)) = game_state
            .table_cards()
            .iter()
            .find(|(_, defense)| defense.is_none())
        {
            // debug!(
            //     "Medium AI ({}) needs to defend against {}",
            //     player_idx, attacking_card
            // );
            let valid_defenses: Vec<(usize, Card)> = hand
                .iter()
                .enumerate()
                .filter(|(_, card)| card.can_beat(attacking_card, trump_suit))
                .map(|(idx, &card)| (idx, card))
                .collect();

            if valid_defenses.is_empty() {
                // debug!("Medium AI ({}) has no cards to defend with", player_idx);
                None // Cannot defend
            } else {
                // Prefer lowest non-trump defense card
                let non_trump_defense = valid_defenses
                    .iter()
                    .filter(|(_, card)| card.suit != trump_suit)
                    .min_by_key(|(_, card)| card.rank);

                if let Some(&(idx, card)) = non_trump_defense {
                    // debug!(
                    //     "Medium AI ({}) defending with lowest non-trump: {}",
                    //     player_idx, card
                    // );
                    Some(idx)
                } else {
                    // If only trump cards can defend, use the lowest trump
                    let lowest_trump_defense =
                        valid_defenses.iter().min_by_key(|(_, card)| card.rank);
                    if let Some(&(idx, card)) = lowest_trump_defense {
                        // debug!(
                        //     "Medium AI ({}) defending with lowest trump: {}",
                        //     player_idx, card
                        // );
                        Some(idx)
                    } else {
                        // Should be unreachable if valid_defenses is not empty
                        // debug!("Medium AI ({}) logical error: valid defenses found but couldn't select one?", player_idx);
                        None
                    }
                }
            }
        } else {
            // debug!(
            //     "Medium AI ({}) - no undefended attacks? Should not be in defense phase.",
            //     player_idx
            // );
            None // Should not happen in defense phase
        }
    }
}

impl AiStrategy for HardStrategy {
    fn difficulty(&self) -> AiDifficulty {
        AiDifficulty::Hard
    }

    fn should_take_cards(&self, game_state: &GameState, player_idx: usize) -> bool {
        // debug!("Hard AI ({}) deciding whether to take cards", player_idx);
        let player = &game_state.players()[player_idx];
        let trump_suit = game_state
            .trump_suit()
            .expect("Trump suit required for defense decision");

        // Find undefended attacks
        let undefended_attacks: Vec<&Card> = game_state
            .table_cards()
            .iter()
            .filter(|(_, defense)| defense.is_none())
            .map(|(attack, _)| attack)
            .collect();

        if undefended_attacks.is_empty() {
            return false;
        }

        // Evaluate the cost of defending vs. taking
        let mut potential_defenses: Vec<Vec<(usize, Card)>> = Vec::new();
        for attack_card in &undefended_attacks {
            let defenses = player.hand().iter().enumerate()
                .filter(|(_, card)| card.can_beat(attack_card, trump_suit))
                .map(|(idx, &card)| (idx, card))
                .collect::<Vec<_>>();

            if defenses.is_empty() {
                // debug!("Hard AI ({}) cannot defend against {}, must take", player_idx, attack_card);
                return true; // Cannot defend one of the attacks
            }
            potential_defenses.push(defenses);
        }

        // Hard AI: Try to keep trump cards and higher value cards if possible.
        // Take if defending requires using multiple valuable trumps or high cards.
        let defense_cost = potential_defenses
            .iter()
            .flatten()
            .filter(|(_, card)| card.suit == trump_suit || card.rank >= Rank::Jack)
            .count();

        let cards_to_take = game_state.table_cards().len(); // Number of pairs currently

        // Simple heuristic: take if defense cost is high relative to cards taken
        // Or if taking fewer cards than current hand size + table cards allows faster discard
        if defense_cost >= 2 || (cards_to_take <= 3 && player.hand_size() + cards_to_take <= 7) {
            // debug!("Hard AI ({}) finds defense costly (cost: {}) or taking is advantageous, taking cards", player_idx, defense_cost);
            true
        } else {
            // debug!(
            //     "Hard AI ({}) decides defense is feasible (cost: {})",
            //     player_idx, defense_cost
            // );
            false
        }
    }

    fn make_attack_move(&self, game_state: &GameState, player_idx: usize) -> Option<usize> {
        // debug!("Hard AI ({}) deciding attack move", player_idx);
        let player = &game_state.players()[player_idx];
        let hand = player.hand();
        let trump_suit = game_state.trump_suit();

        if hand.is_empty() {
            return None;
        }

        // Prioritize getting rid of low-value, non-trump duplicates first
        let mut card_counts: HashMap<Rank, Vec<(usize, Card)>> = HashMap::new();
        for (idx, card) in hand.iter().enumerate() {
            card_counts.entry(card.rank).or_default().push((idx, *card));
        }

        if game_state.table_cards().is_empty() {
            // First attack: Prefer lowest rank non-trump card, especially if duplicates exist
            let best_attack = card_counts
                .iter()
                .filter(|(_, cards)| cards.iter().any(|(_, card)| Some(card.suit) != trump_suit))
                .min_by_key(|(rank, _)| *rank)
                .and_then(|(_, cards)| {
                    cards
                        .iter()
                        .min_by_key(|(_, card)| card.suit == trump_suit.unwrap_or(Suit::Clubs))
                });

            if let Some(&(idx, card)) = best_attack {
                // debug!(
                //     "Hard AI ({}) starting attack with lowest preferred card: {}",
                //     player_idx, card
                // );
                return Some(idx);
            } else {
                // Only trumps left? Play lowest trump
                let lowest_trump = hand.iter().enumerate().min_by_key(|(_, card)| card.rank);
                if let Some((idx, card)) = lowest_trump {
                    // debug!(
                    //     "Hard AI ({}) starting attack with lowest trump: {}",
                    //     player_idx, card
                    // );
                    return Some(idx);
                }
            }
            None // Should not happen if hand is not empty
        } else {
            // Additional attack: Find cards matching ranks on table
            // Prioritize adding cards that the defender might struggle with (e.g., compléter pairs of low cards)
            let table_ranks: HashSet<_> = game_state
                .table_cards()
                .iter()
                .flat_map(|(attack, defense)| {
                    vec![Some(attack.rank), defense.map(|d| d.rank)]
                        .into_iter()
                        .flatten()
                })
                .collect();

            let valid_attacks: Vec<(usize, Card)> = hand
                .iter()
                .enumerate()
                .filter(|(_, card)| table_ranks.contains(&card.rank))
                .map(|(idx, &card)| (idx, card))
                .collect();

            if valid_attacks.is_empty() {
                // debug!("Hard AI ({}) has no valid additional attacks", player_idx);
                return None; // Pass
            }

            // Try to choose an attack the defender has fewer options against
            // (This requires more state analysis - simplified for now: choose lowest rank)
            let (best_idx, best_card) = valid_attacks
                .into_iter()
                .min_by_key(|(_, card)| (card.rank, card.suit == trump_suit.unwrap_or(Suit::Clubs)))
                .unwrap(); // Safe unwrap

            // debug!(
            //     "Hard AI ({}) adding attack with lowest preferred card: {}",
            //     player_idx, best_card
            // );
            Some(best_idx)
        }
    }

    fn make_defense_move(&self, game_state: &GameState, player_idx: usize) -> Option<usize> {
        // debug!("Hard AI ({}) deciding defense move", player_idx);
        let player = &game_state.players()[player_idx];
        let hand = player.hand();
        let trump_suit = game_state
            .trump_suit()
            .expect("Trump suit must be set during defense");

        // Find the first undefended attack
        if let Some((attacking_card, _)) = game_state
            .table_cards()
            .iter()
            .find(|(_, defense)| defense.is_none())
        {
            // debug!(
            //     "Hard AI ({}) needs to defend against {}",
            //     player_idx, attacking_card
            // );
            let valid_defenses: Vec<(usize, Card)> = hand
                .iter()
                .enumerate()
                .filter(|(_, card)| card.can_beat(attacking_card, trump_suit))
                .map(|(idx, &card)| (idx, card))
                .collect();

            if valid_defenses.is_empty() {
                // debug!("Hard AI ({}) cannot defend, will take cards", player_idx);
                return None; // Cannot defend
            }

            // Hard AI: Choose the *absolute minimal* card required to defend.
            // Prefer non-trumps over trumps.
            let best_defense = valid_defenses.iter().min_by(|(_, card_a), (_, card_b)| {
                let a_is_trump = card_a.suit == trump_suit;
                let b_is_trump = card_b.suit == trump_suit;
                match (a_is_trump, b_is_trump) {
                    (false, true) => std::cmp::Ordering::Less, // Prefer non-trump A
                    (true, false) => std::cmp::Ordering::Greater, // Prefer non-trump B
                    _ => card_a.rank.cmp(&card_b.rank),        // If same type, compare rank
                }
            });

            if let Some(&(idx, card)) = best_defense {
                // debug!(
                //     "Hard AI ({}) defending with minimal card: {}",
                //     player_idx, card
                // );
                Some(idx)
            } else {
                // debug!(
                //     "Hard AI ({}) logical error in defense selection",
                //     player_idx
                // );
                None // Should not happen
            }
        } else {
            // debug!(
            //     "Hard AI ({}) - no undefended attacks in defense phase?",
            //     player_idx
            // );
            None
        }
    }
}

// Update AiPlayer to use strategy pattern
pub struct AiPlayer {
    strategy: Box<dyn AiStrategy>,
}

impl AiPlayer {
    pub fn new(difficulty: AiDifficulty) -> Self {
        let strategy: Box<dyn AiStrategy> = match difficulty {
            AiDifficulty::Easy => Box::new(EasyStrategy),
            AiDifficulty::Medium => Box::new(MediumStrategy),
            AiDifficulty::Hard => Box::new(HardStrategy),
        };
        Self { strategy }
    }

    pub fn make_attack_move(&self, game_state: &GameState, player_idx: usize) -> Option<usize> {
        self.strategy.make_attack_move(game_state, player_idx)
    }

    pub fn make_defense_move(
        &mut self,
        game_state: &GameState,
        player_idx: usize,
    ) -> Option<usize> {
        self.strategy.make_defense_move(game_state, player_idx)
    }

    pub fn should_take_cards(&mut self, game_state: &GameState, player_idx: usize) -> bool {
        self.strategy.should_take_cards(game_state, player_idx)
    }

    pub fn make_multi_attack_move(&self, game_state: &GameState, player_idx: usize) -> Vec<usize> {
        // This function determines if AI should attack with multiple cards of the same rank
        // debug!("AI considering multi-card attack for player {}", player_idx);
        let player = &game_state.players()[player_idx];
        let hand = player.hand();
        let defender = &game_state.players()[game_state.current_defender()];
        let defender_hand_size = defender.hand_size();

        // No multi-attack if defender has no cards or we have no cards
        if defender_hand_size == 0 || hand.is_empty() {
            return Vec::new();
        }

        // Get the current difficulty from the strategy
        let difficulty = self.strategy.difficulty();

        // Easy AI doesn't use multi-attacks
        if difficulty == AiDifficulty::Easy {
            return Vec::new();
        }

        // First get a single attack card using the regular attack method
        let first_attack = match self.make_attack_move(game_state, player_idx) {
            Some(idx) => idx,
            None => return Vec::new(), // Can't even make a single attack
        };

        let first_card = hand[first_attack];
        let first_rank = first_card.rank;

        // Find all cards of the same rank
        let mut same_rank_cards: Vec<usize> = hand
            .iter()
            .enumerate()
            .filter(|(_, card)| card.rank == first_rank)
            .map(|(idx, _)| idx)
            .collect();

        // If we don't have multiple cards of the same rank, just return the single card
        if same_rank_cards.len() <= 1 {
            return vec![first_attack];
        }

        // Medium difficulty uses at most 2 cards
        if difficulty == AiDifficulty::Medium && same_rank_cards.len() > 2 {
            // Sort by index and take the first 2
            same_rank_cards.sort();
            same_rank_cards.truncate(2);
        }

        // Limit the number of cards to the defender's hand size
        if same_rank_cards.len() > defender_hand_size {
            // Sort by index and take first N cards
            same_rank_cards.sort();
            same_rank_cards.truncate(defender_hand_size);
        }

        // Hard difficulty AI uses multi-attack more aggressively
        if difficulty == AiDifficulty::Hard {
            // Use all available cards of the same rank (already limited to defender's hand size)
        } else {
            // Medium difficulty - add some randomness to whether we use 1 or multiple cards
            let mut rng = thread_rng();
            if rng.gen_bool(0.5) {
                // 50% chance to just use a single card
                return vec![first_attack];
            }
        }

        // debug!(
        //     "AI selected {} cards for multi-attack",
        //     same_rank_cards.len()
        // );
        same_rank_cards
    }
}

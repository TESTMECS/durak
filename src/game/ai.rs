use crate::game::card::{Card, Rank};
use crate::game::game_state::{GamePhase, GameState};
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiDifficulty {
    Easy,
    Medium,
    Hard,
}
impl Display for AiDifficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiDifficulty::Easy => write!(f, "Easy"),
            AiDifficulty::Medium => write!(f, "Medium"),
            AiDifficulty::Hard => write!(f, "Hard"),
        }
    }
}

// Define a strategy trait for AI behavior
trait AiStrategy {
    fn difficulty(&self) -> AiDifficulty;

    fn should_take_cards(&self, game_state: &GameState, player_idx: usize) -> bool;

    fn make_attack_move(
        &self,
        game_state: &GameState,
        player_idx: usize,
    ) -> Option<Vec<(usize, Card)>>; //Always will return cards to attack with or an error.

    fn make_defense_move(
        &self,
        game_state: &GameState,
        player_idx: usize,
    ) -> Option<Vec<(usize, Card)>>; //Always will return cards to attack with or an error.

    fn make_multi_attack_move(
        &self,
        game_state: &GameState,
        player_idx: usize,
    ) -> Vec<usize> {
        // Default implementation returns empty vec - no multi-attack by default
        Vec::new()
    }
}

// Implement strategies for each difficulty level
struct EasyStrategy;
struct MediumStrategy;
struct HardStrategy;

impl AiStrategy for EasyStrategy {
    fn difficulty(&self) -> AiDifficulty {
        AiDifficulty::Easy
    }

    fn should_take_cards(&self, _game_state: &GameState, _player_idx: usize) -> bool {
        // Easy AI always takes cards rather than trying to defend
        true
    }

    fn make_attack_move(
        &self,
        _game_state: &GameState,
        _player_idx: usize,
    ) -> Option<Vec<(usize, Card)>> {
        // Easy AI just plays the first valid card it finds
        let player = &_game_state.players()[_player_idx];
        let hand = player.hand();

        if hand.is_empty() {
            return None;
        }

        // Just return the first card
        Some(vec![(0, hand[0])])
    }

    fn make_defense_move(
        &self,
        _game_state: &GameState,
        _player_idx: usize,
    ) -> Option<Vec<(usize, Card)>> {
        // Easy AI always chooses to take cards instead of defending
        None
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
            let can_defend = player
                .hand()
                .iter()
                .any(|card| trump_suit.is_some_and(|trump| card.can_beat(attacking_card, trump)));

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

    fn make_attack_move(
        &self,
        game_state: &GameState,
        player_idx: usize,
    ) -> Option<Vec<(usize, Card)>> {
        let player = &game_state.players()[player_idx];
        let hand = player.hand();

        if hand.is_empty() {
            return None;
        }

        // 50% chance to just pass (only if we have cards on the table or it's not the first move)
        let table_cards = game_state.table_cards();
        if !table_cards.is_empty() && rand::random::<f32>() < 0.5 {
            return Some(vec![]);
        }

        // Try to find a card that matches what's on the table
        let valid_ranks: HashSet<Rank> = table_cards
            .iter()
            .flat_map(|(attack, defense)| {
                let mut ranks = Vec::new();
                ranks.push(attack.rank);
                if let Some(def) = defense {
                    ranks.push(def.rank);
                }
                ranks
            })
            .collect();

        // Find non-trump cards that match valid ranks
        let trump_suit = game_state.trump_suit();
        let matching_cards: Vec<(usize, Card)> = hand
            .iter()
            .enumerate()
            .filter(|(_, card)| {
                if let Some(trump) = trump_suit {
                    // Don't play trumps for medium AI if possible
                    if card.suit == trump {
                        return false;
                    }
                }
                table_cards.is_empty() || valid_ranks.contains(&card.rank)
            })
            .map(|(idx, &card)| (idx, card))
            .collect();

        if matching_cards.is_empty() {
            // No matching cards, pick the lowest non-trump card
            let lowest_non_trump = hand
                .iter()
                .enumerate()
                .filter(|(_, card)| {
                    if let Some(trump) = trump_suit {
                        card.suit != trump
                    } else {
                        true
                    }
                })
                .min_by_key(|(_, card)| card.rank);

            if let Some((idx, &card)) = lowest_non_trump {
                return Some(vec![(idx, card)]);
            }

            // If we only have trumps, play the lowest trump
            let lowest_trump_idx = hand
                .iter()
                .enumerate()
                .min_by_key(|(_, card)| card.rank)
                .map(|(idx, _)| idx);

            if let Some(idx) = lowest_trump_idx {
                return Some(vec![(idx, hand[idx])]);
            }

            // Should never reach here as we checked for empty hand
            return None;
        }

        // Medium AI: play a random matching card
        let rand_idx = rand::random::<usize>() % matching_cards.len();
        let (best_idx, _best_card) = matching_cards[rand_idx];
        Some(vec![(best_idx, hand[best_idx])])
    }

    fn make_defense_move(
        &self,
        game_state: &GameState,
        player_idx: usize,
    ) -> Option<Vec<(usize, Card)>> {
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
            let valid_defenses: Vec<(usize, Card)> = hand
                .iter()
                .enumerate()
                .filter(|(_, card)| card.can_beat(attacking_card, trump_suit))
                .map(|(idx, &card)| (idx, card))
                .collect();

            if valid_defenses.is_empty() {
                return None; // Cannot defend
            } else {
                // Prefer lowest non-trump defense card
                let non_trump_defense = valid_defenses
                    .iter()
                    .filter(|(_, card)| card.suit != trump_suit)
                    .min_by_key(|(_, card)| card.rank);

                if let Some(&(idx, card)) = non_trump_defense {
                    return Some(vec![(idx, card)]);
                } else {
                    // If only trump cards can defend, use the lowest trump
                    let lowest_trump_defense =
                        valid_defenses.iter().min_by_key(|(_, card)| card.rank);
                    if let Some(&(idx, card)) = lowest_trump_defense {
                        return Some(vec![(idx, card)]);
                    } else {
                        // Should be unreachable if valid_defenses is not empty
                        return None;
                    }
                }
            }
        } else {
            return None; // Should not happen in defense phase
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
            let defenses = player
                .hand()
                .iter()
                .enumerate()
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

    fn make_attack_move(
        &self,
        game_state: &GameState,
        player_idx: usize,
    ) -> Option<Vec<(usize, Card)>> {
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

        // Try to find a matching rank to a card already on the table
        let table_cards = game_state.table_cards();
        let valid_ranks: HashSet<Rank> = table_cards
            .iter()
            .flat_map(|(attack, defense)| {
                let mut ranks = Vec::new();
                ranks.push(attack.rank);
                if let Some(def) = defense {
                    ranks.push(def.rank);
                }
                ranks
            })
            .collect();

        // First, try for non-trump cards that match table ranks
        if !table_cards.is_empty() {
            let matching_non_trumps: Vec<(usize, Card)> = hand
                .iter()
                .enumerate()
                .filter(|(_, card)| {
                    valid_ranks.contains(&card.rank)
                        && trump_suit.map_or(true, |trump| card.suit != trump)
                })
                .map(|(idx, &card)| (idx, card))
                .collect();

            if !matching_non_trumps.is_empty() {
                // Find lowest rank non-trump match
                if let Some(&(idx, card)) = matching_non_trumps.iter().min_by_key(|(_, c)| c.rank) {
                    return Some(vec![(idx, card)]);
                }
            }
        }

        // Next, look for duplicates (pairs) in hand to play
        let mut dupes = Vec::new();
        for (rank, cards) in card_counts.iter() {
            if cards.len() >= 2 && trump_suit.map_or(true, |trump| {
                cards.iter().any(|(_, c)| c.suit != trump)
            }) {
                // If we have duplicates, prefer non-trumps
                let non_trumps: Vec<&(usize, Card)> = cards
                    .iter()
                    .filter(|(_, c)| trump_suit.map_or(true, |trump| c.suit != trump))
                    .collect();

                if !non_trumps.is_empty() {
                    dupes.push((rank, non_trumps[0]));
                } else {
                    dupes.push((rank, &cards[0]));
                }
            }
        }

        if !dupes.is_empty() {
            if let Some((_, &(idx, card))) = dupes.iter().min_by_key(|(r, _)| *r) {
                return Some(vec![(idx, card)]);
            }
        }

        // If there's nothing on the table or no matches, play lowest non-trump
        let best_attack = hand
            .iter()
            .enumerate()
            .filter(|(_, c)| trump_suit.map_or(true, |trump| c.suit != trump))
            .min_by_key(|(_, c)| c.rank);

        if let Some((idx, &card)) = best_attack {
            return Some(vec![(idx, card)]);
        }

        // If we only have trumps, play the lowest one
        let lowest_trump = hand
            .iter()
            .enumerate()
            .filter(|(_, c)| trump_suit.map_or(false, |trump| c.suit == trump))
            .min_by_key(|(_, c)| c.rank);

        if let Some((idx, &card)) = lowest_trump {
            return Some(vec![(idx, card)]);
        }

        // Hard AI should always find a valid attack if there are cards in hand
        Some(vec![])
    }

    fn make_defense_move(
        &self,
        game_state: &GameState,
        player_idx: usize,
    ) -> Option<Vec<(usize, Card)>> {
        let player = &game_state.players()[player_idx];
        let hand = player.hand();
        let trump_suit = game_state
            .trump_suit()
            .expect("Trump suit required for defense");

        // Find undefended attacks
        let undefended_attack = game_state
            .table_cards()
            .iter()
            .enumerate()
            .find(|(_, (_, defense))| defense.is_none())
            .map(|(idx, (attack, _))| (idx, attack));

        if let Some((table_idx, attack_card)) = undefended_attack {
            // Find all valid defenses for this attack
            let valid_defenses: Vec<(usize, Card)> = hand
                .iter()
                .enumerate()
                .filter(|(_, card)| card.can_beat(attack_card, trump_suit))
                .map(|(idx, &card)| (idx, card))
                .collect();

            if valid_defenses.is_empty() {
                return None;
            }

            // Hard AI strategy: Prioritize lowest valid non-trump, or lowest trump if necessary
            let non_trump_defenses: Vec<&(usize, Card)> = valid_defenses
                .iter()
                .filter(|(_, c)| c.suit != trump_suit)
                .collect();

            if !non_trump_defenses.is_empty() {
                // Use lowest non-trump defense
                let non_trump_defense = non_trump_defenses
                    .iter()
                    .min_by_key(|(_, c)| c.rank);
                    
                if let Some(&&(_idx, card)) = non_trump_defense {
                    return Some(vec![(table_idx, card)]);
                }
            } 
            
            // Have to use a trump, use lowest one
            let lowest_trump_defense = valid_defenses
                .iter()
                .filter(|(_, c)| c.suit == trump_suit)
                .min_by_key(|(_, c)| c.rank);
                
            if let Some(&(_idx, card)) = lowest_trump_defense {
                return Some(vec![(table_idx, card)]);
            }
        }

        None
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

    pub fn should_take_cards(&self, game_state: &GameState, player_idx: usize) -> bool {
        self.strategy.should_take_cards(game_state, player_idx)
    }

    pub fn make_attack_move(
        &self,
        game_state: &GameState,
        player_idx: usize,
    ) -> Option<Vec<(usize, Card)>> {
        self.strategy.make_attack_move(game_state, player_idx)
    }

    pub fn make_defense_move(
        &self,
        game_state: &GameState,
        player_idx: usize,
    ) -> Option<Vec<(usize, Card)>> {
        self.strategy.make_defense_move(game_state, player_idx)
    }

    pub fn make_multi_attack_move(&self, game_state: &GameState, player_idx: usize) -> Vec<usize> {
        self.strategy.make_multi_attack_move(game_state, player_idx)
    }
}

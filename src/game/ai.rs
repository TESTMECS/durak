use crate::game::{GameState, Suit};
use log::{debug, info, trace};
use rand::{seq::SliceRandom, thread_rng, Rng};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiDifficulty {
    #[allow(dead_code)]
    Easy,
    Medium,
    #[allow(dead_code)]
    Hard,
}

pub struct AiPlayer {
    difficulty: AiDifficulty,
}

impl AiPlayer {
    pub fn new(difficulty: AiDifficulty) -> Self {
        info!("Creating new AI player with difficulty: {:?}", difficulty);
        Self { difficulty }
    }

    pub fn make_attack_move(&self, game_state: &GameState, player_idx: usize) -> Option<usize> {
        // Always log when this function is called for debugging
        debug!(
            "AI attempting to make attack move for player {}",
            player_idx
        );

        let player = &game_state.players()[player_idx];
        let hand = player.hand();

        if hand.is_empty() {
            debug!("AI player {} has no cards to attack with", player_idx);
            return None;
        }

        // First attack - play lowest card
        if game_state.table_cards().is_empty() {
            debug!("Table is empty, AI choosing lowest card for first attack");

            // Find lowest non-trump card first if possible
            let trump_suit = game_state.trump_suit();

            // First try to find a non-trump card
            let non_trump_idx = hand
                .iter()
                .enumerate()
                .filter(|(_, card)| trump_suit.map_or(true, |t| card.suit != t))
                .min_by_key(|(_, card)| card.rank as usize)
                .map(|(idx, _)| idx);

            // If we found a non-trump card, use it
            if let Some(idx) = non_trump_idx {
                debug!("AI selected non-trump card {} for first attack", hand[idx]);
                return Some(idx);
            }

            // Otherwise use any card (which will be a trump)
            let idx = 0; // Since we already checked hand is not empty
            debug!(
                "AI selected card {} for first attack (only had trumps)",
                hand[idx]
            );
            return Some(idx);
        }

        // Additional attack - need to match ranks on the table
        let valid_ranks: Vec<_> = game_state
            .table_cards()
            .iter()
            .flat_map(|(attack, defense)| {
                let mut ranks = vec![attack.rank];
                if let Some(def) = defense {
                    ranks.push(def.rank);
                }
                ranks
            })
            .collect();

        debug!("Valid ranks for additional attack: {:?}", valid_ranks);

        // Find cards in our hand that match these ranks
        let valid_attacks: Vec<_> = hand
            .iter()
            .enumerate()
            .filter(|(_, card)| valid_ranks.contains(&card.rank))
            .collect();

        debug!("Found {} potential attack cards", valid_attacks.len());

        // If we have valid attack cards, choose the lowest one
        if !valid_attacks.is_empty() {
            // Sort by rank and prefer non-trumps
            let trump_suit = game_state.trump_suit();
            let best_idx = valid_attacks
                .iter()
                .min_by(|(_, a), (_, b)| {
                    // First compare if one is trump and other is not
                    let a_is_trump = trump_suit.map_or(false, |t| a.suit == t);
                    let b_is_trump = trump_suit.map_or(false, |t| b.suit == t);

                    if a_is_trump && !b_is_trump {
                        std::cmp::Ordering::Greater // Prefer non-trump, so a > b
                    } else if !a_is_trump && b_is_trump {
                        std::cmp::Ordering::Less // Prefer non-trump, so a < b
                    } else {
                        // Both trump or both non-trump, compare by rank
                        a.rank.cmp(&b.rank)
                    }
                })
                .map(|(idx, _)| *idx);

            if let Some(idx) = best_idx {
                debug!("AI chose to attack with card {} ({})", idx, hand[idx]);
                return Some(idx);
            }
        }
        debug!("AI found no valid attack moves, passing turn");
        None
    }

    pub fn make_defense_move(&self, game_state: &GameState, player_idx: usize) -> Option<usize> {
        debug!("AI considering defense move for player {}", player_idx);
        let result = match self.difficulty {
            AiDifficulty::Easy => self.make_easy_defense(game_state, player_idx),
            AiDifficulty::Medium => self.make_medium_defense(game_state, player_idx),
            AiDifficulty::Hard => self.make_hard_defense(game_state, player_idx),
        };

        if let Some(idx) = result {
            debug!("AI is defending!");
            debug!("AI chose defense card index: {}", idx);
        } else {
            debug!("AI couldn't find a valid defense card");
        }

        result
    }

    pub fn should_take_cards(&self, game_state: &GameState, player_idx: usize) -> bool {
        let player = &game_state.players()[player_idx];
        let table_cards = game_state.table_cards();
        let trump_suit = game_state.trump_suit().unwrap_or(Suit::Clubs);

        let undefended_attacks = table_cards
            .iter()
            .filter(|(_, defense)| defense.is_none())
            .count();

        if undefended_attacks == 0 {
            debug!("No undefended attacks, not taking cards");
            return false;
        }

        // Check if we have cards that can defend
        let undefended_card = table_cards
            .iter()
            .find(|(_, defense)| defense.is_none())
            .map(|(attack, _)| attack)
            .unwrap();

        let valid_defenses = player.get_valid_defenses(undefended_card, trump_suit);
        debug!(
            "AI has {} valid defenses for card {}",
            valid_defenses.len(),
            undefended_card
        );

        let decision = match self.difficulty {
            AiDifficulty::Easy => {
                // Easy AI will take cards ~50% of the time if it can defend
                if valid_defenses.is_empty() {
                    debug!("Easy AI has no defenses, must take cards");
                    true
                } else {
                    let mut rng = thread_rng();
                    let take_cards = rng.gen_bool(0.5);
                    debug!("Easy AI randomly decided to take cards: {}", take_cards);
                    take_cards
                }
            }
            AiDifficulty::Medium => {
                // Medium AI will take cards only if it can't defend or if there are many cards
                if valid_defenses.is_empty() {
                    debug!("Medium AI has no defenses, must take cards");
                    true
                } else if table_cards.len() >= 4 {
                    debug!(
                        "Medium AI taking cards because there are too many ({})",
                        table_cards.len()
                    );
                    true
                } else {
                    debug!("Medium AI chose to defend");
                    false
                }
            }
            AiDifficulty::Hard => {
                // Hard AI will take cards only if it can't defend
                let must_take = valid_defenses.is_empty();
                if must_take {
                    debug!("Hard AI has no defenses, must take cards");
                } else {
                    debug!("Hard AI chose to defend");
                }
                must_take
            }
        };

        info!("AI decision to take cards: {}", decision);
        decision
    }

    // Easy AI just plays random valid cards
    fn make_easy_attack(&self, game_state: &GameState, player_idx: usize) -> Option<usize> {
        trace!("Easy AI making attack decision");
        let player = &game_state.players()[player_idx];
        let table_cards = game_state.table_cards();
        let hand = player.hand();

        if hand.is_empty() {
            trace!("Hand is empty, no attack possible");
            return None;
        }

        let mut valid_idxs = Vec::new();

        if table_cards.is_empty() {
            // If table is empty, any card is valid
            valid_idxs.extend(0..hand.len());
            trace!("Table empty, all {} cards are valid attacks", hand.len());
        } else {
            // Otherwise, find cards with matching ranks on the table
            let table_ranks: Vec<_> = table_cards
                .iter()
                .flat_map(|(attack, defense)| {
                    let mut ranks = vec![attack.rank];
                    if let Some(def) = defense {
                        ranks.push(def.rank);
                    }
                    ranks
                })
                .collect();
            trace!("Table has ranks: {:?}", table_ranks);

            for (idx, card) in hand.iter().enumerate() {
                if table_ranks.contains(&card.rank) {
                    valid_idxs.push(idx);
                    trace!("Card {} at index {} is a valid attack", card, idx);
                }
            }
        }

        // Choose a random valid card
        if !valid_idxs.is_empty() {
            let mut rng = thread_rng();
            let choice = *valid_idxs.choose(&mut rng).unwrap();
            trace!(
                "Easy AI chose random attack card at index {}: {}",
                choice,
                hand[choice]
            );
            Some(choice)
        } else {
            trace!("No valid attacks found");
            None
        }
    }

    // Medium AI prefers to play lowest cards first
    fn make_medium_attack(&self, game_state: &GameState, player_idx: usize) -> Option<usize> {
        let player = &game_state.players()[player_idx];
        let table_cards = game_state.table_cards();
        let hand = player.hand();
        let trump_suit = game_state.trump_suit().unwrap_or(Suit::Clubs);

        if hand.is_empty() {
            return None;
        }

        let mut valid_moves = Vec::new();

        if table_cards.is_empty() {
            // If table is empty, prefer non-trump cards
            for (idx, card) in hand.iter().enumerate() {
                // Avoid playing trumps first if possible
                let score = if card.suit == trump_suit {
                    100
                } else {
                    card.rank as usize
                };
                valid_moves.push((idx, score));
            }
        } else {
            // Find cards with matching ranks
            let table_ranks: Vec<_> = table_cards
                .iter()
                .flat_map(|(attack, defense)| {
                    let mut ranks = vec![attack.rank];
                    if let Some(def) = defense {
                        ranks.push(def.rank);
                    }
                    ranks
                })
                .collect();

            for (idx, card) in hand.iter().enumerate() {
                if table_ranks.contains(&card.rank) {
                    // Prefer non-trump cards with lower ranks
                    let score = if card.suit == trump_suit {
                        100
                    } else {
                        card.rank as usize
                    };
                    valid_moves.push((idx, score));
                }
            }
        }

        // Sort by score (lower is better)
        valid_moves.sort_by_key(|&(_, score)| score);

        valid_moves.first().map(|&(idx, _)| idx)
    }

    // Hard AI uses more advanced strategy
    fn make_hard_attack(&self, game_state: &GameState, player_idx: usize) -> Option<usize> {
        let player = &game_state.players()[player_idx];
        let defender = &game_state.players()[game_state.current_defender()];
        let defender_hand_size = defender.hand_size();
        let table_cards = game_state.table_cards();
        let hand = player.hand();
        let trump_suit = game_state.trump_suit().unwrap_or(Suit::Clubs);

        if hand.is_empty() {
            return None;
        }

        let mut valid_moves = Vec::new();

        if table_cards.is_empty() {
            // Initial attack - prefer small non-trump cards or cards we have duplicates of
            let mut rank_counts = std::collections::HashMap::new();
            for card in hand {
                *rank_counts.entry(card.rank).or_insert(0) += 1;
            }

            for (idx, card) in hand.iter().enumerate() {
                // Prefer:
                // 1. Cards we have duplicates of (harder to defend against multiple same rank)
                // 2. Lower ranks
                // 3. Non-trump suits
                let duplicate_bonus = rank_counts.get(&card.rank).unwrap_or(&0) - 1;
                let trump_penalty = if card.suit == trump_suit { 50 } else { 0 };
                let score = card.rank as usize + trump_penalty - (duplicate_bonus * 10);
                valid_moves.push((idx, score));
            }
        } else {
            // Additional attacks - find cards with matching ranks
            let table_ranks: Vec<_> = table_cards
                .iter()
                .flat_map(|(attack, defense)| {
                    let mut ranks = vec![attack.rank];
                    if let Some(def) = defense {
                        ranks.push(def.rank);
                    }
                    ranks
                })
                .collect();

            for (idx, card) in hand.iter().enumerate() {
                if table_ranks.contains(&card.rank) {
                    // For subsequent attacks, prefer ranks that are already hard to defend
                    let duplicate_on_table = table_cards
                        .iter()
                        .filter(|(a, _)| a.rank == card.rank)
                        .count();

                    let trump_penalty = if card.suit == trump_suit { 50 } else { 0 };
                    let score = card.rank as usize + trump_penalty - (duplicate_on_table * 10);
                    valid_moves.push((idx, score));
                }
            }

            // If near the end of the game, try to force the defender to take cards
            if defender_hand_size <= 2 && game_state.deck().is_empty() {
                // Prioritize moves that make it hard to defend
                valid_moves.sort_by_key(|&(_, score)| score);
                return valid_moves.first().map(|&(idx, _)| idx);
            }
        }

        // Sort by score (lower is better)
        valid_moves.sort_by_key(|&(_, score)| score);

        valid_moves.first().map(|&(idx, _)| idx)
    }

    fn make_easy_defense(&self, game_state: &GameState, player_idx: usize) -> Option<usize> {
        let player = &game_state.players()[player_idx];
        let table_cards = game_state.table_cards();
        let trump_suit = game_state.trump_suit().unwrap_or(Suit::Clubs);

        // Find the first undefended attack
        let attacking_card = table_cards
            .iter()
            .find(|(_, defense)| defense.is_none())
            .map(|(attack, _)| attack)?;

        // Get all valid defenses
        let valid_defenses = player.get_valid_defenses(attacking_card, trump_suit);

        if valid_defenses.is_empty() {
            // No valid defenses - must take cards
            return None;
        }

        // Choose a random valid defense
        let mut rng = thread_rng();
        let (idx, _) = valid_defenses.choose(&mut rng).unwrap();
        Some(*idx)
    }

    fn make_medium_defense(&self, game_state: &GameState, player_idx: usize) -> Option<usize> {
        let player = &game_state.players()[player_idx];
        let table_cards = game_state.table_cards();
        let trump_suit = game_state.trump_suit().unwrap_or(Suit::Clubs);

        // Find the first undefended attack
        let attacking_card = table_cards
            .iter()
            .find(|(_, defense)| defense.is_none())
            .map(|(attack, _)| attack)?;

        // Get all valid defenses
        let valid_defenses = player.get_valid_defenses(attacking_card, trump_suit);

        if valid_defenses.is_empty() {
            // No valid defenses - must take cards
            return None;
        }

        // Choose the lowest valid card
        valid_defenses
            .into_iter()
            .min_by_key(|&(_, card)| {
                // Prefer non-trump cards if possible
                if card.suit == attacking_card.suit {
                    card.rank as usize
                } else {
                    // If using trump, prefer lowest trump
                    card.rank as usize + 100
                }
            })
            .map(|(idx, _)| idx)
    }

    fn make_hard_defense(&self, game_state: &GameState, player_idx: usize) -> Option<usize> {
        let player = &game_state.players()[player_idx];
        let table_cards = game_state.table_cards();
        let trump_suit = game_state.trump_suit().unwrap_or(Suit::Clubs);

        // Find the first undefended attack
        let attacking_card = table_cards
            .iter()
            .find(|(_, defense)| defense.is_none())
            .map(|(attack, _)| attack)?;

        // Get all valid defenses
        let valid_defenses = player.get_valid_defenses(attacking_card, trump_suit);

        if valid_defenses.is_empty() {
            // No valid defenses - must take cards
            return None;
        }

        // Count how many cards we have of each suit
        let mut suit_counts = std::collections::HashMap::new();
        for card in player.hand() {
            *suit_counts.entry(card.suit).or_insert(0) += 1;
        }

        // Choose the optimal defense
        valid_defenses
            .into_iter()
            .min_by_key(|&(_, card)| {
                let suit_count = *suit_counts.get(&card.suit).unwrap_or(&0);

                if card.suit == attacking_card.suit {
                    // Same suit - use the lowest card that beats it
                    card.rank as usize
                } else if card.suit == trump_suit {
                    // Using a trump - consider how many trumps we have left
                    // If we have many trumps, we're more willing to use them
                    card.rank as usize + 100 - (suit_count * 5)
                } else {
                    // This shouldn't happen - all valid defenses should be
                    // either same suit with higher rank or trump
                    1000
                }
            })
            .map(|(idx, _)| idx)
    }
}

use crate::game::card::{Card, Rank, Suit};
use crate::game::game_state::GameState;
use crate::ui::debug_overlay::debug;
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

trait AiStrategy {
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
}

struct EasyStrategy;
struct MediumStrategy;
struct HardStrategy;

impl AiStrategy for EasyStrategy {
    /// Easy AI follows the specific logic: if *any* single attacking card cannot be beaten,
    /// immediately decide to pick up all cards.
    fn should_take_cards(&self, game_state: &GameState, player_idx: usize) -> bool {
        let player = &game_state.players()[player_idx];
        let hand = player.hand();
        let trump_suit = game_state
            .trump_suit()
            .unwrap_or_else(|| panic!("Trump suit should be set"));
        // Find the first undefended attack
        for (attack_card, _) in game_state
            .table_cards()
            .iter()
            .filter(|(_, defense)| defense.is_none())
        {
            // AI check if it can defend against this card
            let can_defend = hand
                .iter()
                .any(|card| card.can_beat(attack_card, trump_suit));
            if !can_defend {
                // If any single card can't be beaten, take all cards
                debug(format!(
                    "Easy AI ({}) cannot defend against {}, taking cards",
                    player_idx, attack_card
                ));
                return true;
            }
        }
        // If all cards can be beaten, still 50% chance to take cards
        let random_take = rand::random::<f32>() < 0.5;
        if random_take {
            debug(format!(
                "Easy AI ({}) randomly deciding to take cards",
                player_idx
            ));
            return true;
        }
        false
    }
    /// Easy AI plays the lowest-ranking playable card (non-trump preferred)
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
        // If there are cards on the table, check for cards of matching rank to add
        let table_cards = game_state.table_cards();
        if !table_cards.is_empty() {
            // Get ranks of cards on the table
            let table_ranks: HashSet<Rank> = table_cards
                .iter()
                .flat_map(|(attack, defense)| {
                    let mut ranks = vec![attack.rank];
                    if let Some(def) = defense {
                        ranks.push(def.rank);
                    }
                    ranks
                })
                .collect();
            // Find any card in hand that matches a rank on the table
            for (idx, card) in hand.iter().enumerate() {
                if table_ranks.contains(&card.rank) {
                    debug(format!(
                        "Easy AI adding matching card {} to the attack",
                        card
                    ));
                    return Some(vec![(idx, *card)]);
                }
            }
        }
        // For initial attack, find the lowest non-trump card
        let mut lowest_card = None;
        let mut lowest_idx = 0;
        let mut lowest_value = u8::MAX;
        for (idx, card) in hand.iter().enumerate() {
            // Calculate card value - non-trumps are lower value than trumps
            let is_trump = trump_suit == Some(card.suit);
            let card_value = if is_trump {
                100 + card.rank as u8
            } else {
                card.rank as u8
            };

            if card_value < lowest_value {
                lowest_value = card_value;
                lowest_card = Some(*card);
                lowest_idx = idx;
            }
        }
        if let Some(card) = lowest_card {
            debug(format!("Easy AI attacking with lowest card: {}", card));
            return Some(vec![(lowest_idx, card)]);
        }
        // Should never reach here if hand is not empty
        Some(vec![(0, hand[0])])
    }
    /// Easy AI logic: simply use the lowest possible card that can defend. It doesn't save trumps strategically
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

        // Find the first undefended attack
        if let Some((attack_card, _)) = game_state
            .table_cards()
            .iter()
            .find(|(_, defense)| defense.is_none())
        {
            // First check if we can beat with a non-trump of the same suit
            let non_trump_defenses: Vec<(usize, Card)> = hand
                .iter()
                .enumerate()
                .filter(|(_, card)| {
                    card.suit == attack_card.suit
                        && card.suit != trump_suit
                        && card.rank > attack_card.rank
                })
                .map(|(idx, &card)| (idx, card))
                .collect();
            // Use the lowest non-trump if available
            if !non_trump_defenses.is_empty() {
                if let Some(&(idx, card)) = non_trump_defenses.iter().min_by_key(|(_, c)| c.rank) {
                    debug(format!("Easy AI defending with non-trump: {}", card));
                    return Some(vec![(idx, card)]);
                }
            }
            // If no non-trump defense, check for any trump that can beat it
            let trump_defenses: Vec<(usize, Card)> = hand
                .iter()
                .enumerate()
                .filter(|(_, card)| {
                    (card.suit == trump_suit && attack_card.suit != trump_suit)
                        || (card.suit == trump_suit
                            && attack_card.suit == trump_suit
                            && card.rank > attack_card.rank)
                })
                .map(|(idx, &card)| (idx, card))
                .collect();
            // Use the lowest trump if available
            if !trump_defenses.is_empty() {
                if let Some(&(idx, card)) = trump_defenses.iter().min_by_key(|(_, c)| c.rank) {
                    debug(format!("Easy AI defending with trump: {}", card));
                    return Some(vec![(idx, card)]);
                }
            }
        }
        // Cannot defend - will need to take cards
        None
    }
}

impl AiStrategy for MediumStrategy {
    /// Medium AI evaluates all attacking cards before playing any defense.
    /// Medium AI will take cards if:
    /// 1. Multiple valuable trumps are required (2 or more)
    /// 2. Any high trumps (Jack+) are required
    /// 3. There are 4 or more attacks to defend against
    /// 4. Random 40% chance to take cards if 2+ trumps are needed
    fn should_take_cards(&self, game_state: &GameState, player_idx: usize) -> bool {
        let player = &game_state.players()[player_idx];
        let hand = player.hand();
        let trump_suit = game_state
            .trump_suit()
            .expect("Trump suit required for defense decision");
        // Get all undefended attacks
        let undefended_attacks: Vec<&Card> = game_state
            .table_cards()
            .iter()
            .filter(|(_, defense)| defense.is_none())
            .map(|(attack, _)| attack)
            .collect();
        if undefended_attacks.is_empty() {
            return false;
        }
        // Count how many trump cards would be needed to defend
        let mut trump_cards_needed = 0;
        let mut high_trumps_needed = 0; // Trumps higher than 10
        //
        // Check if any card cannot be beaten
        for attack_card in &undefended_attacks {
            // Try to find a defense for this attack
            let possible_defenses: Vec<Card> = hand
                .iter()
                .filter(|card| card.can_beat(attack_card, trump_suit))
                .cloned()
                .collect();

            if possible_defenses.is_empty() {
                debug(format!(
                    "Medium AI ({}) cannot defend against {}, taking cards",
                    player_idx, attack_card
                ));
                return true; // Cannot defend this card, must take all
            }
            // Check if defense requires a trump
            let requires_trump = possible_defenses.iter().all(|card| card.suit == trump_suit);
            if requires_trump {
                trump_cards_needed += 1;
                // Check if it requires a high trump (Jack or higher)
                let requires_high_trump = possible_defenses
                    .iter()
                    .filter(|card| card.suit == trump_suit)
                    .all(|card| card.rank >= Rank::Jack);
                if requires_high_trump {
                    high_trumps_needed += 1;
                }
            }
        }
        if high_trumps_needed > 0 {
            debug(format!(
                "Medium AI ({}) taking cards to save high trumps",
                player_idx
            ));
            return true;
        }
        if trump_cards_needed >= 2 {
            let random_take = rand::random::<f32>() < 0.4;
            if random_take {
                debug(format!(
                    "Medium AI ({}) taking cards to save multiple trumps",
                    player_idx
                ));
                return true;
            }
        }
        if undefended_attacks.len() >= 4 {
            debug(format!(
                "Medium AI ({}) taking cards due to too many attacks ({})",
                player_idx,
                undefended_attacks.len()
            ));
            return true;
        }
        debug(format!("Medium AI ({}) will try to defend", player_idx));
        false
    }
    /// Attack with some probability of dropping.  
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
        // Get cards on the table for potential additional attacks
        let table_cards = game_state.table_cards();
        // If this is an additional attack (not the first card played)
        if !table_cards.is_empty() {
            // Get all ranks already on the table
            let valid_ranks: HashSet<Rank> = table_cards
                .iter()
                .flat_map(|(attack, defense)| {
                    let mut ranks = vec![attack.rank];
                    if let Some(def) = defense {
                        ranks.push(def.rank);
                    }
                    ranks
                })
                .collect();
            // Medium AI has a 30% chance to stop adding cards
            let stop_adding = rand::random::<f32>() < 0.3;
            if stop_adding {
                debug(format!(
                    "Medium AI ({}) decided to stop adding cards",
                    player_idx
                ));
                return Some(vec![]);
            }
            // Look for matching non-trump cards first
            let matching_non_trumps: Vec<(usize, Card)> = hand
                .iter()
                .enumerate()
                .filter(|(_, card)| {
                    valid_ranks.contains(&card.rank) && trump_suit != Some(card.suit)
                })
                .map(|(idx, &card)| (idx, card))
                .collect();
            if !matching_non_trumps.is_empty() {
                // Find the lowest matching non-trump
                if let Some((idx, card)) = matching_non_trumps
                    .iter()
                    .min_by_key(|(_, card)| card.rank)
                    .map(|(idx, card)| (*idx, *card))
                {
                    debug(format!(
                        "Medium AI adding non-trump card {} to attack",
                        card
                    ));
                    return Some(vec![(idx, card)]);
                }
            }
            // If defender used a trump, medium AI might add a matching trump (30% chance)
            let defender_used_trump = table_cards.iter().any(
                |(_, defense)| matches!((defense, trump_suit), (Some(d), Some(t)) if d.suit == t),
            );
            if defender_used_trump {
                let add_trump = rand::random::<f32>() < 0.3;
                if add_trump {
                    // Look for matching trump cards
                    let matching_trumps: Vec<(usize, Card)> = hand
                        .iter()
                        .enumerate()
                        .filter(|(_, card)| {
                            valid_ranks.contains(&card.rank) && trump_suit == Some(card.suit)
                        })
                        .map(|(idx, &card)| (idx, card))
                        .collect();

                    if !matching_trumps.is_empty() {
                        // Find the lowest matching trump
                        if let Some((idx, card)) = matching_trumps
                            .iter()
                            .min_by_key(|(_, card)| card.rank)
                            .map(|(idx, card)| (*idx, *card))
                        {
                            debug(format!("Medium AI adding trump card {} to attack", card));
                            return Some(vec![(idx, card)]);
                        }
                    }
                }
            }

            // No good additional cards to play
            return Some(vec![]);
        }
        // Initial attack logic - prioritize low non-trump cards
        // First, check for pairs that might be useful for future attacks
        let mut rank_counts: HashMap<Rank, Vec<(usize, Card)>> = HashMap::new();
        for (idx, card) in hand.iter().enumerate() {
            rank_counts.entry(card.rank).or_default().push((idx, *card));
        }
        // Find pairs of non-trumps
        let non_trump_pairs: Vec<(&Rank, &Vec<(usize, Card)>)> = rank_counts
            .iter()
            .filter(|(_, cards)| {
                cards.len() >= 2
                    && cards.iter().any(|(_, c)| {
                        matches!(trump_suit, Some(t) if c.suit != t) || trump_suit.is_none()
                    })
            })
            .collect();
        // Try to play a card from the lowest pair
        if !non_trump_pairs.is_empty() {
            if let Some((_, cards)) = non_trump_pairs.iter().min_by_key(|(rank, _)| *rank) {
                // Find the lowest non-trump in this group
                if let Some((idx, card)) = cards
                    .iter()
                    .filter(|(_, c)| trump_suit != Some(c.suit))
                    .min_by_key(|(_, c)| c.rank)
                {
                    debug(format!("Medium AI playing from pair: {}", card));
                    return Some(vec![(*idx, *card)]);
                }
            }
        }
        // If no pairs, play the lowest non-trump card
        let lowest_non_trump = hand
            .iter()
            .enumerate()
            .filter(|(_, card)| trump_suit != Some(card.suit))
            .min_by_key(|(_, card)| card.rank);
        if let Some((idx, &card)) = lowest_non_trump {
            debug(format!("Medium AI playing lowest non-trump: {}", card));
            return Some(vec![(idx, card)]);
        }
        // If we only have trumps, play the lowest one (if AI has several)
        let trump_cards: Vec<(usize, Card)> = hand
            .iter()
            .enumerate()
            .filter(|(_, card)| trump_suit != Some(card.suit))
            .map(|(idx, &card)| (idx, card))
            .collect();
        if trump_cards.len() > 1 {
            if let Some((idx, card)) = trump_cards
                .iter()
                .min_by_key(|(_, c)| c.rank)
                .map(|(i, c)| (*i, *c))
            {
                debug(format!(
                    "Medium AI playing lowest trump (has multiple): {}",
                    card
                ));
                return Some(vec![(idx, card)]);
            }
        }
        // Last resort - play any card (lowest by rank)
        if let Some((idx, &card)) = hand.iter().enumerate().min_by_key(|(_, c)| c.rank) {
            debug(format!("Medium AI playing lowest card: {}", card));
            return Some(vec![(idx, card)]);
        }
        // Should never reach here
        None
    }
    /// Medium AI strategy:
    /// 1. If the attack card is high value, might use a trump strategically
    /// 2. Otherwise, prefer non-trump defenses
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
        // Find the first undefended attack
        if let Some((attacking_card, _)) = game_state
            .table_cards()
            .iter()
            .find(|(_, defense)| defense.is_none())
        {
            // Medium AI: 30% chance to pass if possible
            let possible_passes: Vec<(usize, Card)> = hand
                .iter()
                .enumerate()
                .filter(|(_, card)| card.can_pass(attacking_card))
                .map(|(idx, &card)| (idx, card))
                .collect();
            if !possible_passes.is_empty() {
                let pass_chance = rand::random::<f32>();
                if pass_chance < 0.3 {
                    // Choose the lowest pass card
                    let lowest_pass = possible_passes.iter().min_by_key(|(_, card)| {
                        // Prefer non-trumps for passing
                        if card.suit == trump_suit {
                            100 + card.rank as u8
                        } else {
                            card.rank as u8
                        }
                    });
                    if let Some(&(hand_idx, pass_card)) = lowest_pass {
                        debug(format!(
                            "Medium AI choosing to PASS with {} (same rank as {})",
                            pass_card, attacking_card,
                        ));
                        return Some(vec![(hand_idx, pass_card)]);
                    }
                }
            }
            // Find all valid defenses
            let valid_defenses: Vec<(usize, Card)> = hand
                .iter()
                .enumerate()
                .filter(|(_, card)| card.can_beat(attacking_card, trump_suit))
                .map(|(idx, &card)| (idx, card))
                .collect();
            if valid_defenses.is_empty() {
                return None; // Cannot defend
            }
            // Determine if this is a high-value card that's worth using a trump on
            let is_high_value = attacking_card.rank >= Rank::Jack
                || (attacking_card.suit == trump_suit && attacking_card.rank >= Rank::Ten);
            if is_high_value {
                // For high-value attacks, might use a trump (70% chance)
                let use_trump_strategically = rand::random::<f32>() < 0.7;
                if use_trump_strategically {
                    let trump_defenses: Vec<&(usize, Card)> = valid_defenses
                        .iter()
                        .filter(|(_, card)| card.suit == trump_suit)
                        .collect();
                    if !trump_defenses.is_empty() {
                        // Use the lowest trump that can beat it
                        let lowest_trump = trump_defenses.iter().min_by_key(|(_, card)| card.rank);
                        if let Some(&&(idx, card)) = lowest_trump {
                            debug(format!(
                                "Medium AI using trump {} to beat high value card {}",
                                card, attacking_card
                            ));
                            return Some(vec![(idx, card)]);
                        }
                    }
                }
            }
            // Try to find a non-trump defense first
            let non_trump_defenses: Vec<&(usize, Card)> = valid_defenses
                .iter()
                .filter(|(_, card)| card.suit != trump_suit)
                .collect();
            if !non_trump_defenses.is_empty() {
                // Use the lowest non-trump defense
                if let Some(&&(idx, card)) =
                    non_trump_defenses.iter().min_by_key(|(_, card)| card.rank)
                {
                    debug(format!("Medium AI defending with non-trump {}", card));
                    return Some(vec![(idx, card)]);
                }
            }
            // If forced to use a trump, use the lowest one
            if let Some(&(idx, card)) = valid_defenses
                .iter()
                .filter(|(_, c)| c.suit == trump_suit)
                .min_by_key(|(_, c)| c.rank)
            {
                debug(format!(
                    "Medium AI forced to use trump {} (lowest available)",
                    card
                ));
                return Some(vec![(idx, card)]);
            }
            // Should be unreachable if valid_defenses is not empty
            return None;
        }
        // No undefended attacks
        None
    }
}

impl AiStrategy for HardStrategy {
    /// To calculate the cost-benefit of picking up the AI will evaluate the number of valuable cards 
    /// where it considers trump cards bigger than Jack to be valuable. In the future, I want to
    /// implement a more dynamic valueable calculation.  
    fn should_take_cards(&self, game_state: &GameState, player_idx: usize) -> bool {
        // Hard AI makes a strategic decision weighing multiple factors
        let player = &game_state.players()[player_idx];
        let hand = player.hand();
        let trump_suit = game_state
            .trump_suit()
            .expect("Trump suit required for defense decision");
        // Track played cards to better understand the game state
        let table_cards = game_state.table_cards();
        let discard_pile = game_state.discard_pile();
        let deck_empty = game_state.deck().is_empty();
        // Find all undefended attacks
        let undefended_attacks: Vec<&Card> = table_cards
            .iter()
            .filter(|(_, defense)| defense.is_none())
            .map(|(attack, _)| attack)
            .collect();
        if undefended_attacks.is_empty() {
            return false;
        }
        // First check if we are able to defend at all
        for attack_card in &undefended_attacks {
            let defenses = hand
                .iter()
                .enumerate()
                .filter(|(_, card)| card.can_beat(attack_card, trump_suit))
                .map(|(idx, &card)| (idx, card))
                .collect::<Vec<_>>();
            if defenses.is_empty() {
                debug(format!(
                    "Hard AI ({}) cannot defend against {}, must take",
                    player_idx, attack_card
                ));
                return true; // Cannot defend one of the attacks, must take
            }
        }
        // Calculate the cost of defending vs. the benefit of picking up
        // 1. Evaluate defense cost: How many valuable cards would be spent?
        let mut defense_plan: HashMap<usize, Card> = HashMap::new(); // Maps attack index -> defense card
        let mut valuable_cards_used = 0;
        let mut high_trumps_used = 0;
        // For each attack, find the optimal defense card
        for (attack_idx, attack_card) in undefended_attacks.iter().enumerate() {
            // Get all possible defenses for this attack
            let possible_defenses: Vec<(usize, Card)> = hand
                .iter()
                .enumerate()
                .filter(|(_idx, card)|
                    // Skip cards already assigned to defend other attacks
                    !defense_plan.values().any(|c| *c == **card) &&
                    card.can_beat(attack_card, trump_suit))
                .map(|(idx, &card)| (idx, card))
                .collect();
            if possible_defenses.is_empty() {
                // If we can't defend with remaining cards, must take
                return true;
            }
            // First try to find a non-trump defense
            let non_trump_defense = possible_defenses
                .iter()
                .filter(|(_, card)|card.suit != trump_suit)
                .min_by_key(|(_, card)| card.rank);
            if let Some(&(_idx, card)) = non_trump_defense {
                // Use this non-trump card
                defense_plan.insert(attack_idx, card);
                // Count valuable non-trump cards (Jack or higher)
                if card.rank >= Rank::Jack {
                    valuable_cards_used += 1;
                }
            } else {
                // Must use a trump
                let trump_defense = possible_defenses
                    .iter()
                    .filter(|(_, card)|card.suit == trump_suit)
                    .min_by_key(|(_, card)| card.rank);
                if let Some(&(_idx, card)) = trump_defense {
                    defense_plan.insert(attack_idx, card);
                    // Any trump is valuable, higher trumps are more so
                    valuable_cards_used += 1;
                    if card.rank >= Rank::Jack {
                        high_trumps_used += 1;
                    }
                }
            }
        }
        // 2. Endgame considerations
        let is_endgame = deck_empty || game_state.deck().size() <= 2;
        // In endgame, conserving high trumps is critical for winning
        if is_endgame && high_trumps_used > 0 {
            // Count how many high trumps might still be unplayed
            let high_trumps_played = discard_pile
                .iter()
                .filter(|card| card.suit == trump_suit && card.rank >= Rank::Jack)
                .count();
            // If we'd use our last high trump, consider picking up instead
            let holding_last_high_trumps = high_trumps_used
                >= hand
                    .iter()
                    .filter(|card| card.suit == trump_suit && card.rank >= Rank::Jack)
                    .count();
            if holding_last_high_trumps && high_trumps_played < 4 {
                debug(format!(
                    "Hard AI ({}) preserving last high trumps in endgame",
                    player_idx
                ));
                return true;
            }
        }
        // 3. Cards to pick up vs hand overload
        let cards_to_take = table_cards.len();
        let new_hand_size = player.hand_size() + cards_to_take;
        let hand_limit = 6; // Standard hand size
        // Only take cards if it doesn't overload our hand too much
        if new_hand_size > hand_limit + 2 && valuable_cards_used < 2 {
            debug(format!(
                "Hard AI ({}) avoiding taking too many cards ({})",
                player_idx, cards_to_take
            ));
            return false;
        }
        // 4. Opponent hand analysis
        let defender_idx = game_state.current_defender();
        let attacker_idx = game_state.current_attacker();
        let opponent_idx = if player_idx == defender_idx {
            attacker_idx
        } else {
            defender_idx
        };
        let opponent = &game_state.players()[opponent_idx];
        let opponent_card_count = opponent.hand_size();
        // If opponent is almost out of cards, defend more aggressively
        if is_endgame && opponent_card_count <= 2 && valuable_cards_used <= 1 {
            debug(format!(
                "Hard AI ({}) defending aggressively against nearly-empty opponent",
                player_idx
            ));
            return false;
        }
        // Final decision balancing all factors
        let strategic_take = (valuable_cards_used >= 2)
            || (high_trumps_used >= 1 && is_endgame)
            || (cards_to_take <= 2 && new_hand_size <= hand_limit);
        if strategic_take {
            debug(format!(
                "Hard AI ({}) strategically taking cards (value cards: {}, high trumps: {})",
                player_idx, valuable_cards_used, high_trumps_used
            ));
            return true;
        }
        debug(format!(
            "Hard AI ({}) decides to defend (cards used: {})",
            player_idx, valuable_cards_used
        ));
        false
    }
    /// Hard AI has various strategies for making attacking moves. 
    /// Plan A. If it is late into the game the AI will analyze the player's hand, discard pile, and
    ///    table cards to determine the best attack move. 
    /// Plan B. If it is an inital attack during the endgame, the AI will try to prevent the opponent
    ///    from discarding cards. Otherwise, the AI will try to play the lowest-ranking card that can
    ///    beat the attacker. 
    /// A fallback strategy is also implemented in case the AI cannot find a good attack move.
    fn make_attack_move(
        &self,
        game_state: &GameState,
        player_idx: usize,
    ) -> Option<Vec<(usize, Card)>> {
        let player = &game_state.players()[player_idx];
        let hand = player.hand();
        let trump_suit = game_state.trump_suit();
        let deck_empty = game_state.deck().is_empty();
        let discard_pile = game_state.discard_pile();
        let table_cards = game_state.table_cards();
        // Get information about the defender
        let defender_idx = game_state.current_defender();
        let defender = &game_state.players()[defender_idx];
        let defender_hand_size = defender.hand_size();
        if hand.is_empty() {
            return None;
        }
        // ### Plan A: Not the first card attack ###
        if !table_cards.is_empty() {
            // Get ranks of cards already on the table
            let valid_ranks: HashSet<Rank> = table_cards
                .iter()
                .flat_map(|(attack, defense)| {
                    let mut ranks = vec![attack.rank];
                    if let Some(def) = defense {
                        ranks.push(def.rank);
                    }
                    ranks
                })
                .collect();
            // Track played cards of each rank to guide our attack strategy
            let mut rank_card_count: HashMap<Rank, usize> = HashMap::new();
            // Count cards in discard pile by rank
            for card in discard_pile {
                *rank_card_count.entry(card.rank).or_insert(0) += 1;
            }
            // Count cards on table by rank
            for (attack, defense) in table_cards {
                *rank_card_count.entry(attack.rank).or_insert(0) += 1;
                if let Some(def) = defense {
                    *rank_card_count.entry(def.rank).or_insert(0) += 1;
                }
            }
            let defender_used_trump = table_cards.iter().any(|(_, defense)| {
                matches!((defense, trump_suit), (Some(d), Some(t)) if d.suit == t)
            });
            // Find weaknesses in defender's hand
            let mut probable_weak_ranks: Vec<Rank> = Vec::new();
            // Ranks where many cards are already out are likely weak points
            for (rank, count) in &rank_card_count {
                if valid_ranks.contains(rank) && *count >= 2 {
                    // If defender has defended against this rank, but many cards of this rank
                    // are already played, they likely have few or no more of this rank
                    probable_weak_ranks.push(*rank);
                }
            }
            // Try adding cards of ranks that are likely weak points for defender
            if !probable_weak_ranks.is_empty() {
                let matching_cards: Vec<(usize, Card)> = hand
                    .iter()
                    .enumerate()
                    .filter(|(_, card)| 
                        probable_weak_ranks.contains(&card.rank) &&
                        // Don't waste high trumps on additional attacks
                        !(card.suit == trump_suit.unwrap_or(Suit::Spades) && card.rank >= Rank::Jack)
                    )
                    .map(|(idx, &card)| (idx, card))
                    .collect();
                if !matching_cards.is_empty() {
                    // Choose lowest card from weak ranks
                    if let Some(&(idx, card)) = matching_cards.iter().min_by_key(|(_, c)| {
                        // Non-trumps first, then by rank
                        if c.suit == trump_suit.unwrap_or(Suit::Spades) {
                            100 + c.rank as u8
                        } else {
                            c.rank as u8
                        }
                    }) {
                        debug(format!("Hard AI exploiting weak rank with {}", card));
                        return Some(vec![(idx, card)]);
                    }
                }
            }
            // Find any matching cards
            let matching_cards: Vec<(usize, Card)> = hand
                .iter()
                .enumerate()
                .filter(|(_, card)| valid_ranks.contains(&card.rank))
                .map(|(idx, &card)| (idx, card))
                .collect();
            if !matching_cards.is_empty() {
                // If defender is struggling (using trumps), consider adding a trump to force more trumps
                if defender_used_trump {
                    let matching_trumps: Vec<&(usize, Card)> = matching_cards
                        .iter()
                        .filter(|(_, card)| trump_suit == Some(card.suit))
                        .collect();
                    // Hard AI will strategically add trumps 70% of the time if defender used trumps
                    let add_trump = !matching_trumps.is_empty() && rand::random::<f32>() < 0.7;
                    if add_trump {
                        // Use lowest matching trump
                        if let Some(&&(idx, card)) =
                            matching_trumps.iter().min_by_key(|(_, c)| c.rank)
                        {
                            debug(format!(
                                "Hard AI strategically adding trump {} to pressure defender",
                                card
                            ));
                            return Some(vec![(idx, card)]);
                        }
                    }
                }
                // Otherwise prefer non-trumps
                let non_trump_matches: Vec<&(usize, Card)> = matching_cards
                    .iter()
                    .filter(|(_, card)| trump_suit != Some(card.suit))
                    .collect();
                if !non_trump_matches.is_empty() {
                    // Choose the lowest non-trump match
                    if let Some(&&(idx, card)) =
                        non_trump_matches.iter().min_by_key(|(_, c)| c.rank)
                    {
                        debug(format!("Hard AI adding non-trump {} to attack", card));
                        return Some(vec![(idx, card)]);
                    }
                }
                // If no non-trumps, use lowest matching card of any type
                if let Some(&(idx, card)) = matching_cards.iter().min_by_key(|(_, c)| c.rank) {
                    debug(format!(
                        "Hard AI adding lowest matching card {} to attack",
                        card
                    ));
                    return Some(vec![(idx, card)]);
                }
            }
            // Hard AI knows when not to add cards - if defender beat earlier attacks easily
            // or if there are already many cards on the table
            let easy_defense = table_cards.iter().all(|(_, defense)| defense.is_some());
            if easy_defense || table_cards.len() >= 3 {
                let stop_chance = if easy_defense { 0.8 } else { 0.5 };
                if rand::random::<f32>() < stop_chance {
                    debug(format!(
                        "Hard AI strategically stops adding cards (easy defense: {})",
                        easy_defense
                    ));
                    return Some(vec![]);
                }
            }
            // No good cards to add
            return Some(vec![]);
        }
        // ### Plan B: Initial attack strategy - varies based on game phase ###
        let is_endgame = deck_empty;
        if is_endgame {
            // Endgame strategy: force opponent to use trumps or pick up
            // If defender has few cards, try to prevent them from discarding
            if defender_hand_size <= 2 {
                // Check if we have high cards or trumps that might force pickup
                let forcing_cards: Vec<(usize, Card)> = hand
                    .iter()
                    .enumerate()
                    .filter(|(_, card)| {
                        (card.suit == trump_suit.unwrap_or(Suit::Spades) && card.rank >= Rank::Ten)
                            || card.rank >= Rank::Ace
                    })
                    .map(|(idx, &card)| (idx, card))
                    .collect();
                if !forcing_cards.is_empty() {
                    // Use a threatening card to prevent easy discard
                    if let Some(&(idx, card)) = forcing_cards.iter().min_by_key(|(_, c)| c.rank) {
                        debug(format!("Hard AI playing forcing card {} in endgame", card));
                        return Some(vec![(idx, card)]);
                    }
                }
            }
        }
        // Check for duplicate ranks (pairs) for strategic play
        let mut rank_counts: HashMap<Rank, Vec<(usize, Card)>> = HashMap::new();
        for (idx, card) in hand.iter().enumerate() {
            rank_counts.entry(card.rank).or_default().push((idx, *card));
        }
        // Hard AI prefers to lead with cards where it has multiple of the same rank
        let pairs: Vec<(Rank, &Vec<(usize, Card)>)> = rank_counts
            .iter()
            .filter(|(_, cards)| cards.len() >= 2)
            .map(|(rank, cards)| (*rank, cards))
            .collect();
        if !pairs.is_empty() {
            // Use the lowest pair that's not high trumps
            let non_high_trump_pairs: Vec<(Rank, &Vec<(usize, Card)>)> = pairs
                .iter()
                .filter(|(_, cards)| {
                    !cards.iter().all(|(_, c)| {
                        c.suit == trump_suit.unwrap_or(Suit::Spades) && c.rank >= Rank::Jack
                    })
                })
                .map(|(r, c)| (*r, *c))
                .collect();
            if !non_high_trump_pairs.is_empty() {
                if let Some((_, cards)) = non_high_trump_pairs.iter().min_by_key(|(rank, _)| *rank)
                {
                    // Find a non-trump from this pair if possible
                    let non_trump = cards
                        .iter()
                        .filter(|(_, c)| trump_suit != Some(c.suit))
                        .min_by_key(|(_, c)| c.rank);

                    if let Some(&(idx, card)) = non_trump {
                        debug(format!("Hard AI playing from pair: {}", card));
                        return Some(vec![(idx, card)]);
                    } else {
                        // Use lowest card from the pair
                        let lowest = cards.iter().min_by_key(|(_, c)| c.rank);
                        if let Some(&(idx, card)) = lowest {
                            debug(format!("Hard AI playing from pair: {}", card));
                            return Some(vec![(idx, card)]);
                        }
                    }
                }
            }
        }
        // Regular strategy - play lowest non-trump probing card
        let non_trumps: Vec<(usize, Card)> = hand
            .iter()
            .enumerate()
            .filter(|(_, card)| trump_suit != Some(card.suit))
            .map(|(idx, &card)| (idx, card))
            .collect();
        if !non_trumps.is_empty() {
            // Use lowest non-trump
            if let Some(&(idx, card)) = non_trumps.iter().min_by_key(|(_, c)| c.rank) {
                debug(format!("Hard AI playing lowest non-trump: {}", card));
                return Some(vec![(idx, card)]);
            }
        }
        // If only trumps are available, use lowest one
        if let Some((idx, &card)) = hand
            .iter()
            .enumerate()
            .filter(|(_, c)| trump_suit == Some(c.suit))
            .min_by_key(|(_, c)| c.rank)
        {
            debug(format!("Hard AI playing lowest trump: {}", card));
            return Some(vec![(idx, card)]);
        }
        // Fallback - play any card
        let (idx, &card) = hand.iter().enumerate().min_by_key(|(_, c)| c.rank).unwrap(); // Safe because we checked for empty hand
        debug(format!("Hard AI playing lowest card: {}", card));
        Some(vec![(idx, card)])
    }
    /// First the AI considers passing with 60% probability because don't want to pass trumps or
    /// value cards, then AI tries to find the lowest card it can beat the attacker with. If it is
    /// a trump suit, it will try to find the lowest trump that can beat the attacker.
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
        // These variables are used by hard AI for tracking
        let _table_cards = game_state.table_cards();
        let _discard_pile = game_state.discard_pile();
        let is_endgame = game_state.deck().is_empty();
        // Find the first undefended attack
        if let Some((_attack_idx, attack_card)) = game_state
            .table_cards()
            .iter()
            .enumerate()
            .find(|(_, (_, defense))| defense.is_none())
            .map(|(idx, (attack, _))| (idx, attack))
        {
            // Hard AI first considers passing as a strategic option
            let possible_passes: Vec<(usize, Card)> = hand
                .iter()
                .enumerate()
                .filter(|(_, card)| card.can_pass(attack_card))
                .map(|(idx, &card)| (idx, card))
                .collect();
            // Hard AI is aggressive with passing (60% chance if available)
            // but won't pass high trumps or valuable cards
            if !possible_passes.is_empty() {
                // Filter out valuable cards to avoid passing them
                let safe_passes: Vec<(usize, Card)> = possible_passes
                    .iter()
                    .filter(|(_, card)|
                        // Don't pass high trumps or aces
                        !(card.suit == trump_suit && card.rank >= Rank::Jack) &&
                        card.rank != Rank::Ace)
                    .map(|&(idx, card)| (idx, card))
                    .collect();
                if !safe_passes.is_empty() && rand::random::<f32>() < 0.6 {
                    // Choose the best pass card - prefer non-trumps
                    let best_pass = safe_passes.iter().min_by_key(|(_, card)| {
                        if card.suit == trump_suit {
                            100 + card.rank as u8
                        } else {
                            card.rank as u8
                        }
                    });
                    if let Some(&(hand_idx, pass_card)) = best_pass {
                        debug(format!(
                            "Hard AI strategically passing with {} (same rank as {})",
                            pass_card, attack_card
                        ));
                        return Some(vec![(hand_idx, pass_card)]);
                    }
                }
                // In emergency (no other defense), pass with anything
                if !possible_passes.is_empty() && safe_passes.is_empty() {
                    // Find the lowest value pass card
                    if let Some(&(hand_idx, pass_card)) =
                        possible_passes.iter().min_by_key(|(_, card)| card.rank)
                    {
                        debug(format!(
                            "Hard AI forced to pass with {} as last resort",
                            pass_card
                        ));
                        return Some(vec![(hand_idx, pass_card)]);
                    }
                }
            }
            // Find all cards that can beat this attack
            let valid_defenses: Vec<(usize, Card)> = hand
                .iter()
                .enumerate()
                .filter(|(_, card)|card.can_beat(attack_card, trump_suit))
                .map(|(idx, &card)| (idx, card))
                .collect();
            if valid_defenses.is_empty() {
                // If we have no defense cards but do have pass cards, force a pass
                if !possible_passes.is_empty() {
                    let (hand_idx, pass_card) = possible_passes[0];
                    debug(format!(
                        "Hard AI forced to pass with {} (no other defense)",
                        pass_card
                    ));
                    return Some(vec![(hand_idx, pass_card)]);
                }
                return None; // Can't defend at all
            }
            // Hard AI strategy: Use the absolute lowest card that can beat the attack
            // First, try to use a non-trump defense if possible
            let non_trump_defenses: Vec<&(usize, Card)> = valid_defenses
                .iter()
                .filter(|(_, card)|card.suit != trump_suit)
                .collect();
            if !non_trump_defenses.is_empty() {
                // Use the lowest non-trump that beats it
                if let Some(&&(hand_idx, card)) =
                    non_trump_defenses.iter().min_by_key(|(_, card)| card.rank)
                {
                    debug(format!("Hard AI defending with lowest non-trump: {}", card));
                    return Some(vec![(hand_idx, card)]);
                }
            }
            // If forced to use a trump, use the lowest possible one
            let trump_defenses: Vec<&(usize, Card)> = valid_defenses
                .iter()
                .filter(|(_, card)| card.suit == trump_suit)
                .collect();
            if !trump_defenses.is_empty() {
                // In endgame, think hard about using high trumps
                if is_endgame {
                    let is_high_value_attack = attack_card.rank >= Rank::Queen
                        || (attack_card.suit == trump_suit && attack_card.rank >= Rank::Ten);
                    // Only use high trumps against high-value cards in endgame
                    if !is_high_value_attack {
                        // Find the lowest trump that's not too valuable (less than Jack)
                        let low_trump_defense = trump_defenses
                            .iter()
                            .filter(|(_, card)| card.rank < Rank::Jack)
                            .min_by_key(|(_, card)| card.rank);

                        if let Some(&&(hand_idx, card)) = low_trump_defense {
                            debug(format!(
                                "Hard AI using low trump {} to conserve high trumps",
                                card
                            ));
                            return Some(vec![(hand_idx, card)]);
                        }
                    }
                }
                // Use absolute lowest trump that can beat it
                if let Some(&&(hand_idx, card)) =
                    trump_defenses.iter().min_by_key(|(_, card)| card.rank)
                {
                    debug(format!("Hard AI using lowest possible trump: {}", card));
                    return Some(vec![(hand_idx, card)]);
                }
            }
        }
        // If we reach here, something went wrong
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
}

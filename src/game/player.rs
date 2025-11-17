use super::card::{Card, Suit};
use arrayvec::ArrayVec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerType {
    Human,
    Computer,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    pub player_type: PlayerType,
    pub hand: ArrayVec<Card, 36>,
}

impl Player {
    pub fn new(name: String, player_type: PlayerType) -> Self {
        Self {
            name,
            player_type,
            hand: ArrayVec::new(),
        }
    }
    /*
     * Get the player name
     * returns: &str
     * */
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }
    /*
     * Get the player type
     * returns: &PlayerType: Human | Computer
     * */
    #[inline]
    pub fn player_type(&self) -> &PlayerType {
        &self.player_type
    }

    /*
     * Get the player hand
     * returns: &Vec<Card> where a card is a struct of rank and suit
     * */
    #[inline]
    pub fn hand(&self) -> &[Card] {
        &self.hand
    }
    // Can pass Vec or Arr T.
    pub fn add_cards<I: IntoIterator<Item = Card>>(&mut self, cards: I) {
        self.hand.extend(cards);
        self.sort_hand();
    }
    #[inline]
    pub fn sort_hand(&mut self) {
        // Sort by suit and then by rank
        // self.hand.sort_by(|a, b| {
        //     if a.suit == b.suit {
        //         a.rank.cmp(&b.rank)
        //     } else {
        //         // Sort by suit enum order
        //         (a.suit as usize).cmp(&(b.suit as usize))
        //     }
        // });
        self.hand.sort_by_key(|c| (c.suit as u8, c.rank as u8));
    }

    #[inline]
    pub fn remove_card(&mut self, index: usize) -> Option<Card> {
        self.hand.get(index)?;
        Some(self.hand.remove(index))
    }

    #[inline]
    pub fn hand_size(&self) -> usize {
        self.hand.len()
    }

    #[inline]
    pub fn is_empty_hand(&self) -> bool {
        self.hand.is_empty()
    }

    pub fn get_lowest_trump(&self, trump_suit: Suit) -> Option<(usize, Card)> {
        // self.hand
        //     .iter()
        //     .enumerate()
        //     .filter(|(_, card)| card.suit == trump_suit)
        //     .min_by_key(|(_, card)| card.rank)
        //     .map(|(idx, &card)| (idx, card))
        let mut best: Option<(usize, Card)> = None;
        for (idx, &card) in self.hand.iter().enumerate() {
            if card.suit != trump_suit {
                continue;
            }
            match best {
                None => best = Some((idx, card)),
                Some((_, best_card)) if card.rank < best_card.rank => {
                    best = Some((idx, card));
                }
                _ => {}
            }
        }
        best
    }

    #[allow(dead_code)]
    pub fn get_valid_defenses(
        &self,
        attacking_card: &Card,
        trump_suit: Suit,
    ) -> Vec<(usize, Card)> {
        // self.hand
        //     .iter()
        //     .enumerate()
        //     .filter(|(_, card)| card.can_beat(attacking_card, trump_suit))
        //     .map(|(idx, &card)| (idx, card))
        //     .collect()
        let mut out = Vec::new();
        for (idx, &card) in self.hand.iter().enumerate() {
            if card.can_beat(attacking_card, trump_suit) {
                out.push((idx, card));
            }
        }
        out
    }

    #[allow(dead_code)]
    pub fn get_lowest_card(&self) -> Option<(usize, Card)> {
        if self.hand.is_empty() {
            return None;
        }
        // // Find lowest non-trump card, or lowest trump if that's all we have
        // self.hand
        //     .iter()
        //     .enumerate()
        //     .min_by_key(|(_, card)| (card.rank as usize, card.suit as usize))
        //     .map(|(idx, &card)| (idx, card))
        let mut best: Option<(usize, Card)> = None;
        for (idx, &card) in self.hand.iter().enumerate() {
            match best {
                None => best = Some((idx, card)),
                Some((_, best_card)) if card.rank < best_card.rank => {
                    best = Some((idx, card));
                }
                _ => {}
            }
        }
        best
    }
}

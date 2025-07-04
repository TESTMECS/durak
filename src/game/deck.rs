use rand::seq::SliceRandom;
use rand::thread_rng;

use super::card::{Card, Rank, Suit};

#[derive(Debug, Clone)]
pub struct Deck {
    pub cards: Vec<Card>,
    pub trump_suit: Option<Suit>,
}

impl Deck {
    pub fn new() -> Self {
        let mut cards = Vec::with_capacity(36);

        for suit in Suit::all() {
            for rank in Rank::all() {
                cards.push(Card::new(suit, rank));
            }
        }

        Self {
            cards,
            trump_suit: None,
        }
    }

    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.cards.shuffle(&mut rng);

        // The bottom card determines the trump suit
        if let Some(bottom_card) = self.cards.last() {
            self.trump_suit = Some(bottom_card.suit);
        }
    }

    pub fn deal(&mut self, count: usize) -> Vec<Card> {
        let mut hand = Vec::with_capacity(count);
        for _ in 0..count {
            if let Some(card) = self.cards.pop() {
                hand.push(card);
            } else {
                break;
            }
        }
        hand
    }

    pub fn trump_suit(&self) -> Option<Suit> {
        self.trump_suit
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn remaining(&self) -> usize {
        self.cards.len()
    }

    // Alias for remaining() to match AI implementation naming
    pub fn size(&self) -> usize {
        self.cards.len()
    }

    #[allow(dead_code)]
    pub fn bottom_card(&self) -> Option<&Card> {
        self.cards.last()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new_deck() {
        let deck = Deck::new();
        assert_eq!(deck.remaining(), 36);
    }
}

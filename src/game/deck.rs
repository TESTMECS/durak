use arrayvec::ArrayVec;
use rand::seq::SliceRandom;
use rand::thread_rng;

use super::card::{Card, Rank, Suit};

pub const fn build_full_deck() -> [Card; 36] {
    let suits = Suit::all();
    let ranks = Rank::all();

    let mut out = [Card::new(Suit::Clubs, Rank::Six); 36];
    let mut i = 0;

    let mut s = 0;
    while s < suits.len() {
        let mut r = 0;
        while r < ranks.len() {
            out[i] = Card::new(suits[s], ranks[r]);
            i += 1;
            r += 1;
        }
        s += 1;
    }

    out
}

pub const FULL_DECK: [Card; 36] = build_full_deck();

#[derive(Debug, Clone)]
pub struct Deck {
    pub cards: ArrayVec<Card, 36>,
    pub trump_suit: Option<Suit>,
}
impl Deck {
    #[inline]
    pub fn new() -> Self {
        Self {
            cards: FULL_DECK.into(),
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

    #[inline]
    pub const fn trump_suit(&self) -> Option<Suit> {
        self.trump_suit
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    #[inline]
    pub const fn remaining(&self) -> usize {
        self.cards.len()
    }
    #[inline]
    pub const fn size(&self) -> usize {
        self.cards.len()
    }
    #[allow(dead_code)]
    #[inline]
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

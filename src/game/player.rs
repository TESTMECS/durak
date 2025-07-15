use super::card::{Card, Suit};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerType {
    Human,
    Computer,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    pub player_type: PlayerType,
    pub hand: Vec<Card>,
}

impl Player {
    pub fn new(name: String, player_type: PlayerType) -> Self {
        Self {
            name,
            player_type,
            hand: Vec::new(),
        }
    }
    /*
     * Get the player name
     * returns: &str
     * */
    pub fn name(&self) -> &str {
        &self.name
    }

    /*
     * Get the player type
     * returns: &PlayerType: Human | Computer
     * */
    pub fn player_type(&self) -> &PlayerType {
        &self.player_type
    }

    /*
     * Get the player hand
     * returns: &Vec<Card> where a card is a struct of rank and suit
     * */
    pub fn hand(&self) -> &[Card] {
        &self.hand
    }

    pub fn add_cards(&mut self, cards: Vec<Card>) {
        self.hand.extend(cards);
        self.sort_hand();
    }

    pub fn sort_hand(&mut self) {
        // Sort by suit and then by rank
        self.hand.sort_by(|a, b| {
            if a.suit == b.suit {
                a.rank.cmp(&b.rank)
            } else {
                // Sort by suit enum order
                (a.suit as usize).cmp(&(b.suit as usize))
            }
        });
    }

    pub fn remove_card(&mut self, index: usize) -> Option<Card> {
        if index < self.hand.len() {
            Some(self.hand.remove(index))
        } else {
            None
        }
    }

    pub fn hand_size(&self) -> usize {
        self.hand.len()
    }

    pub fn is_empty_hand(&self) -> bool {
        self.hand.is_empty()
    }

    pub fn get_lowest_trump(&self, trump_suit: Suit) -> Option<(usize, Card)> {
        self.hand
            .iter()
            .enumerate()
            .filter(|(_, card)| card.suit == trump_suit)
            .min_by_key(|(_, card)| card.rank)
            .map(|(idx, &card)| (idx, card))
    }

    #[allow(dead_code)]
    pub fn get_valid_defenses(
        &self,
        attacking_card: &Card,
        trump_suit: Suit,
    ) -> Vec<(usize, Card)> {
        self.hand
            .iter()
            .enumerate()
            .filter(|(_, card)| card.can_beat(attacking_card, trump_suit))
            .map(|(idx, &card)| (idx, card))
            .collect()
    }

    #[allow(dead_code)]
    pub fn get_lowest_card(&self) -> Option<(usize, Card)> {
        if self.hand.is_empty() {
            return None;
        }
        // Find lowest non-trump card, or lowest trump if that's all we have
        self.hand
            .iter()
            .enumerate()
            .min_by_key(|(_, card)| (card.rank as usize, card.suit as usize))
            .map(|(idx, &card)| (idx, card))
    }
}

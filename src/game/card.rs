use std::fmt;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    pub fn all() -> Vec<Suit> {
        vec![Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades]
    }

    pub fn symbol(&self) -> &str {
        match self {
            Suit::Clubs => "♣",
            Suit::Diamonds => "♦",
            Suit::Hearts => "♥",
            Suit::Spades => "♠",
        }
    }

    pub fn is_red(&self) -> bool {
        matches!(self, Suit::Diamonds | Suit::Hearts)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rank {
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Rank {
    pub fn all() -> Vec<Rank> {
        vec![
            Rank::Six,
            Rank::Seven,
            Rank::Eight,
            Rank::Nine,
            Rank::Ten,
            Rank::Jack,
            Rank::Queen,
            Rank::King,
            Rank::Ace,
        ]
    }

    pub fn symbol(&self) -> &str {
        match self {
            Rank::Six => "6",
            Rank::Seven => "7",
            Rank::Eight => "8",
            Rank::Nine => "9",
            Rank::Ten => "10",
            Rank::Jack => "J",
            Rank::Queen => "Q",
            Rank::King => "K",
            Rank::Ace => "A",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl Card {
    pub fn new(suit: Suit, rank: Rank) -> Self {
        Self { suit, rank }
    }

    /// Determines if this card can beat another card in Durak rules
    ///
    /// A card can beat another card if:
    /// 1. It is the same suit but higher rank, OR
    /// 2. It is a trump card and the other card is not
    ///
    /// # Arguments
    /// * `other` - The attacking card to beat
    /// * `trump_suit` - The current trump suit for the game
    ///
    /// # Returns
    /// `true` if this card can beat the other card, `false` otherwise
    pub fn can_beat(&self, other: &Card, trump_suit: Suit) -> bool {
        // Case 1: Same suit - higher rank wins
        if self.suit == other.suit {
            return self.rank > other.rank;
        }

        // Case 2: Different suits - trump beats non-trump
        if self.suit == trump_suit && other.suit != trump_suit {
            return true;
        }

        // In all other cases, the card cannot be beaten by this card.
        false
    }

    /// Determines if this card can be used to pass an attack in Podkidnoy Durak
    ///
    /// A card can pass an attack if it has the same rank as the attacking card
    /// (regardless of the suit)
    ///
    /// # Arguments
    /// * `other` - The attacking card to check against
    ///
    /// # Returns
    /// `true` if this card can pass the attack, `false` otherwise
    pub fn can_pass(&self, other: &Card) -> bool {
        self.rank == other.rank
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.rank.symbol(), self.suit.symbol())
    }
}

#[cfg(test)]
mod tests {
    // We need to import the items from the parent module (the file we're in)
    // so we can use them in our tests
    use super::*;
    #[test]
    /// Basic Rank and Suit tests
    fn test_can_beat_same_suit() {
        // Create two cards of the same suit.
        let trump_suit = Suit::Spades;
        let card1 = Card::new(Suit::Hearts, Rank::Seven); // 7 of Hearts
        let card2 = Card::new(Suit::Hearts, Rank::Ten); // 10 of Hearts
                                                        // We expect card2 to beat card1 because it has a higher rank.
        let card3 = Card::new(Suit::Diamonds, Rank::Seven); // 7 of Diamonds
        assert!(card2.can_beat(&card1, trump_suit));
        assert!(!card1.can_beat(&card2, trump_suit));
        // Random other suit with lower rank
        assert!(!card3.can_beat(&card3, trump_suit));
    }
    #[test]
    /// Test that a trump card can beat a lower trump card
    fn test_can_beat_trump_to_trump() {
        let trump_suit = Suit::Spades;
        let card1 = Card::new(Suit::Spades, Rank::Six); // 6 of Spades
        let card2 = Card::new(Suit::Spades, Rank::Seven); // 7 of Spades
                                                          // We expect card2 to beat card1 because it has a higher rank.
        assert!(card2.can_beat(&card1, trump_suit));
        assert!(!card1.can_beat(&card2, trump_suit));
    }
    #[test]
    /// Test that a non-trump card cannot beat a trump card
    fn test_can_beat_trump() {
        // Create a trump card and a non-trump card.
        let trump_suit = Suit::Spades;
        let trump_card = Card::new(Suit::Spades, Rank::Six); // 6 of Spades
        let other_card = Card::new(Suit::Hearts, Rank::Ace); // Ace of Hearts
        assert!(trump_card.can_beat(&other_card, trump_suit));
        // The non-trump card cannot beat the trump card.
        assert!(!other_card.can_beat(&trump_card, trump_suit));
    }
}

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
        else if self.suit == trump_suit && other.suit != trump_suit {
            return true;
        }
        // Default: Higher rank wins
        self.rank > other.rank
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

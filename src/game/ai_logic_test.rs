#[cfg(test)]
mod tests {
    use crate::game::ai::{AiDifficulty, AiPlayer};
    use crate::game::card::{Card, Rank, Suit};
    use crate::game::deck::Deck;
    use crate::game::game_state::{GamePhase, GameState};
    use crate::game::player::{Player, PlayerType};

    // Helper function to create a game state for testing
    fn create_test_game_state(
        ai_hand: Vec<Card>,
        table_cards: Vec<(Card, Option<Card>)>,
        trump_suit: Suit,
    ) -> GameState {
        let game_state = GameState {
            players: vec![
                Player {
                    name: "AI".to_string(),
                    player_type: PlayerType::Computer,
                    hand: ai_hand,
                },
                Player {
                    name: "Human".to_string(),
                    player_type: PlayerType::Human,
                    hand: vec![],
                },
            ],
            deck: Deck {
                cards: vec![],
                trump_suit: Some(trump_suit),
            },
            discard_pile: vec![],
            table_cards,
            current_attacker: 1,
            current_defender: 0,
            trump_suit: Some(trump_suit),
            game_phase: GamePhase::Defense,
            winner: None,
            stuck_counter: 0,
        };
        game_state
    }

    #[test]
    /// Test that the Easy AI takes cards if it cannot defend
    fn test_easy_should_take_cards_cannot_defend() {
        let ai = AiPlayer::new(AiDifficulty::Easy);
        let ai_hand = vec![Card::new(Suit::Hearts, Rank::Seven)];
        let table_cards = vec![
            (Card::new(Suit::Hearts, Rank::Six), None),
            (Card::new(Suit::Hearts, Rank::Nine), None),
        ];
        let game_state = create_test_game_state(ai_hand, table_cards, Suit::Spades);

        assert!(ai.should_take_cards(&game_state, 0));
    }

    #[test]
    /// Test that the Easy AI can make an attack move with the lowest-ranking card and save the trump
    fn test_easy_make_attack_move_initial() {
        let ai = AiPlayer::new(AiDifficulty::Easy);
        let ai_hand = vec![
            Card::new(Suit::Hearts, Rank::Seven),
            Card::new(Suit::Spades, Rank::Six), // Trump
        ];
        let game_state = create_test_game_state(ai_hand, vec![], Suit::Spades);

        let attack_move = ai.make_attack_move(&game_state, 0).unwrap();
        assert_eq!(attack_move.len(), 1);
        assert_eq!(attack_move[0].1, Card::new(Suit::Hearts, Rank::Seven));
    }

    #[test]
    /// Test that the Easy AI recognizes that it can "pass" with the same card
    fn test_easy_make_attack_move_add_to_attack() {
        let ai = AiPlayer::new(AiDifficulty::Easy);
        let ai_hand = vec![
            Card::new(Suit::Hearts, Rank::Seven),
            Card::new(Suit::Diamonds, Rank::Ten),
        ];
        let table_cards = vec![(Card::new(Suit::Hearts, Rank::Ten), None)];
        let game_state = create_test_game_state(ai_hand, table_cards, Suit::Spades);

        let attack_move = ai.make_attack_move(&game_state, 0).unwrap();
        assert_eq!(attack_move.len(), 1);
        assert_eq!(attack_move[0].1, Card::new(Suit::Diamonds, Rank::Ten));
    }

    #[test]
    /// Test that the Easy AI recognizes it can defend (don't use trumps)
    fn test_easy_make_defense_move_non_trump() {
        let ai = AiPlayer::new(AiDifficulty::Easy);
        let ai_hand = vec![
            Card::new(Suit::Hearts, Rank::Ten),
            Card::new(Suit::Spades, Rank::Jack),
        ];
        let table_cards = vec![(Card::new(Suit::Hearts, Rank::Seven), None)];
        let game_state = create_test_game_state(ai_hand, table_cards, Suit::Spades);

        let defense_move = ai.make_defense_move(&game_state, 0).unwrap();
        assert_eq!(defense_move.len(), 1);
        assert_eq!(defense_move[0].1, Card::new(Suit::Hearts, Rank::Ten));
    }

    #[test]
    /// Test that the Easy AI recognizes it can't defend
    fn test_easy_make_defense_move_cannot_defend() {
        let ai = AiPlayer::new(AiDifficulty::Easy);
        let ai_hand = vec![Card::new(Suit::Hearts, Rank::Seven)];
        let table_cards = vec![(Card::new(Suit::Hearts, Rank::Ten), None)];
        let game_state = create_test_game_state(ai_hand, table_cards, Suit::Spades);

        let defense_move = ai.make_defense_move(&game_state, 0);
        assert!(defense_move.is_none());
    }

    #[test]
    /// Test that the Medium AI will take cards if there are multiple high trumps
    fn test_medium_should_take_cards_to_save_high_trumps() {
        let ai = AiPlayer::new(AiDifficulty::Medium);
        let ai_hand = vec![
            Card::new(Suit::Spades, Rank::Jack), // High trump
            Card::new(Suit::Hearts, Rank::Seven),
        ];
        let table_cards = vec![(Card::new(Suit::Diamonds, Rank::Ten), None)];
        let game_state = create_test_game_state(ai_hand, table_cards, Suit::Spades);

        assert!(ai.should_take_cards(&game_state, 0));
    }

    #[test]
    /// Test that the Medium AI can make an attack move with the lowest-ranking card and save the trump
    fn test_medium_make_attack_move_initial() {
        let ai = AiPlayer::new(AiDifficulty::Medium);
        let ai_hand = vec![
            Card::new(Suit::Hearts, Rank::Seven),
            Card::new(Suit::Hearts, Rank::Eight),
            Card::new(Suit::Spades, Rank::Six), // Trump
        ];
        let game_state = create_test_game_state(ai_hand, vec![], Suit::Spades);

        let attack_move = ai.make_attack_move(&game_state, 0).unwrap();
        assert_eq!(attack_move.len(), 1);
        assert_eq!(attack_move[0].1, Card::new(Suit::Hearts, Rank::Seven));
    }

    #[test]
    /// Test that the Hard AI will take cards if there are multiple high trumps
    fn test_hard_should_take_cards_strategically() {
        let ai = AiPlayer::new(AiDifficulty::Hard);
        let ai_hand = vec![
            Card::new(Suit::Spades, Rank::Jack), // High trump
            Card::new(Suit::Hearts, Rank::Ace),
        ];
        let table_cards = vec![
            (Card::new(Suit::Diamonds, Rank::Ten), None),
            (Card::new(Suit::Clubs, Rank::Ten), None),
        ];
        let game_state = create_test_game_state(ai_hand, table_cards, Suit::Spades);

        assert!(ai.should_take_cards(&game_state, 0));
    }

    #[test]
    /// Test that the Hard AI can make an attack move with the lowest-ranking card and save the trump
    fn test_hard_make_attack_move_exploit_weakness() {
        let ai = AiPlayer::new(AiDifficulty::Hard);
        let ai_hand = vec![
            Card::new(Suit::Hearts, Rank::Seven),
            Card::new(Suit::Diamonds, Rank::Ten),
        ];
        let table_cards = vec![(Card::new(Suit::Hearts, Rank::Ten), None)];
        let mut game_state = create_test_game_state(ai_hand, table_cards, Suit::Spades);
        game_state
            .discard_pile
            .push(Card::new(Suit::Clubs, Rank::Ten));

        let attack_move = ai.make_attack_move(&game_state, 0).unwrap();
        assert_eq!(attack_move.len(), 1);
        assert_eq!(attack_move[0].1, Card::new(Suit::Diamonds, Rank::Ten));
    }
}

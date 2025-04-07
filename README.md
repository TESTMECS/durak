# Durak Card Game

A terminal-based implementation of the popular Russian card game Durak, built with Rust and [ratatui](https://github.com/ratatui-org/ratatui).

## Game Rules

Durak is a card game played with 2-6 players using a 36-card deck (cards 6 through Ace).
[Wikipedia](https://en.wikipedia.org/wiki/Durak)

### Objective
Get rid of all your cards. The last player with cards is the "durak" (fool).

### Setup
1. Each player receives 6 cards
2. The next card determines the trump suit
3. The player with the lowest trump card goes first

### Gameplay
1. The attacker plays a card
2. The defender must either:
   - Beat it with a higher card of the same suit or a trump
   - Pass the attack by playing a card of the same rank (regardless of suit) to the next player
3. If a pass occurs, the next player must now defend against both cards
4. If defense is successful, the defender becomes the next attacker
5. If the defender can't or won't defend, they pick up all cards on the table, and the next player becomes the attacker
6. After each round, players draw back up to 6 cards (attacker draws first)
7. Once the deck is empty, players with no cards are out of the game
8. The last player with cards is the "durak"

### Multiple Card Attacks
- Players can attack with multiple cards of the same rank
- The defender must defend against each card separately
- The total number of attack cards cannot exceed the defender's hand size
- Additional attack cards can only be played if their rank already exists on the table
- Use 'M' to toggle multiple selection mode, Space to select cards, Enter to play all selected cards

## Game Controls

### Main Menu
- `S`: Start new game
- `R`: View rules
- `Q`: Quit

### During Game
- `←/→`: Select card
- `M`: Toggle multiple selection mode
- `Space`: Toggle card selection in multi-select mode
- `Enter`: Play selected card(s)
- `P`: Pass (when attacking)
- `T`: Take cards (when defending)
- `Q`: Quit to main menu
- `D`: Toggle debug overlay

### Game Over
- `N`: New game
- `Q`: Quit

## Game Flow Chart

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│             │     │             │     │             │     │             │
│  Main Menu  │────►│  Game Setup │────►│Attack Phase │────►│Defense Phase│
│             │     │             │     │             │     │             │
└─────────────┘     └─────────────┘     └──────┬──────┘     └──────┬──────┘
       ▲                                        │                   │
       │                                        │                   │
       │                                        │                   │
       │                                        ▼                   ▼
       │                                 ┌─────────────┐     ┌─────────────┐
       │                                 │             │     │             │
       │                                 │  Pass Turn  │     │  Take Cards │
       │                                 │             │     │             │
       │                                 └──────┬──────┘     └──────┬──────┘
       │                                        │                   │
       │                                        ▼                   │
       │                                 ┌─────────────┐            │
       │                                 │             │            │
       │                                 │Drawing Phase│◄───────────┘
       │                                 │             │
       │                                 └──────┬──────┘
       │                                        │
       │                                        │
       │                                        ▼
       │                                 ┌─────────────┐
       │                                 │  Check for  │
       │                                 │   Winner    │──┐
       │                                 │             │  │
       │                                 └─────────────┘  │
       │                                                  │
       │                                                  ▼
       │                                           ┌─────────────┐
       │                                           │             │
       └───────────────────────────────────────────┤  Game Over  │
                                                   │             │
                                                   └─────────────┘
```

## Building and Running

1. Install Rust: https://www.rust-lang.org/tools/install
2. Clone this repository
3. Run the game:

```bash
cargo run
```

For better performance, use the release build:

```bash
cargo run --release
```

## AI Difficulty Levels

The game includes an AI opponent with adjustable difficulty:

- **Easy**: Makes simple moves, doesn't use multi-card attacks
- **Medium**: Uses better strategy, occasionally attacks with 2 cards
- **Hard**: Uses advanced strategy, aggressively attacks with multiple cards when possible, and makes smarter defense decisions

## Development

To enable debug logging:

```bash
DURAK_DEBUG_FILE=durak_debug.log cargo run
```

## Bugs
- [x] `Safe exit` -> if an error occurs, the game will break and the terminal will be broken need to restore it like is done in `main.rs`. But if error occurs the game will break. -- FIXED
- [x] Multi attack with the last cards in your hand causes an index out of bounds error. Not exactly sure why. (Error in `logic.rs`), should get fixed when I safe exit the game. -- FIXED
- [x] Logic file needs to be refactored 1k lines is crazy. -- FIXED (Split into app_core.rs, game_actions.rs, ai_handler.rs, and game_loop.rs)


## License

MIT

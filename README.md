# Durak Card Game

A terminal-based implementation of the classic Russian card game "Durak" using Rust and Ratatui.

## Future Todos

- Rules page on Main Menu.
- Strategy Page on Main Menu.
- Multiple Card Attacking and Defending.
- More Intelligent AI Defender.
- Passing when Attacking with same rank card rule. (Difficult)

## Game Overview

Durak is a popular Russian card game where the objective is to get rid of all your cards. The player who is left with cards at the end of the game is the "durak" (fool).

## Rules

1. **Setup**: Each player is dealt 6 cards from a 36-card deck (from 6 to Ace).
2. **Trump Suit**: The bottom card of the deck determines the trump suit, which has priority over other suits.
3. **First Player**: The player with the lowest trump card goes first.
4. **Attack & Defense**:
   - The attacker plays a card; the defender must beat it with a higher card of the same suit or a trump card.
   - If successfully defended, the attacker can add cards of the same rank as cards on the table.
   - If the defender can't or doesn't want to defend, they pick up all the cards on the table.
   - After a successful defense, the defender becomes the next attacker.
5. **Drawing**: After each round, players draw from the deck to maintain 6 cards in hand, starting with the attacker.
6. **End Game**: Once the deck is empty and a player has no cards left, that player is out. The last player with cards is the "durak".

## Controls

- **Main Menu**:

  - `s`: Start a new game
  - `q`: Quit
  - `d`: Toggle debug overlay

- **During Game**:

  - `←/→` or `↑/↓`: Navigate cards in hand
  - `Enter`: Play selected card
  - `t`: Take cards (when defending)
  - `p`: Pass (when attacking)
  - `q`: Quit
  - `d`: Toggle debug overlay

- **Game Over**:
  - `n`: New game
  - `q`: Quit
  - `d`: Toggle debug overlay

## AI Opponents (WIP)

- Work in progress, will just choose lowest card to make a valid move atp.

The game includes an AI opponent with adjustable difficulty:

- Easy: Makes random valid moves
- Medium: Makes more strategic moves
- Hard: Uses advanced strategy and considers card values

## Installation

### Prerequisites

- Rust and Cargo installed

### Building and Running

```bash
# Clone the repository
git clone https://github.com/yourusername/durak.git
cd durak

# Build and run
cargo run

# For release version
cargo run --release
```

## Debugging

The game includes a built-in debug overlay that can be toggled with the `d` key during gameplay. The overlay displays recent log messages with timestamps and log levels, making it easier to diagnose issues during gameplay without disrupting the main UI.

### Running with Debug Logging to File

You can also enable file-based debug logging by setting the `DURAK_DEBUG_FILE` environment variable:

```bash
DURAK_DEBUG_FILE=durak_debug.log cargo run
```

This will create a log file with detailed information about the game's behavior.

### Log Levels

The log messages are color-coded by level:

- ERROR (Red) - Critical issues that prevent proper game function
- WARN (Yellow) - Problems that might affect gameplay but don't prevent execution
- INFO (Green) - Important game state changes and player actions
- DEBUG (Blue) - Detailed information about game flow
- TRACE (Gray) - Very detailed information for step-by-step debugging

## Common Issues and Solutions

1. **Stack Overflow**: If the game crashes with a stack overflow, this might be due to an infinite recursion in AI processing. Try running with LOG_TO_FILE=1 to capture the state before the crash.

2. **Invalid Card Plays**: The logs will show detailed information about why a card play might be rejected, helping identify issues with game rules implementation.

3. **Game State Errors**: All transitions between game phases are logged, making it easier to identify when the game gets into an invalid state.

Report any issues with the log file attached for faster resolution.

## Development

### Building from Source

```bash
cargo build --release
```

### Running Tests

```bash
cargo test
```

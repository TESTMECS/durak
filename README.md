
# [durak](https://crates.io/crates/durak)
built with Rust and [ratatui](https://github.com/ratatui-org/ratatui).
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
# Features
## Multiple Card Attacks
- The human Player can attack with multiple cards of the same rank
- The defender must defend against each card separately
- The total number of attack cards cannot exceed the defender's hand size
- Additional attack cards can only be played if their rank already exists on the table
- Use 'M' to toggle multiple selection mode, Space to select cards, Enter to play all selected cards
## AI Difficulty Levels
- Easy, Medium, Hard, encorporates statistical strategies.
## Bugs
- Please report any bugs to issues this is my first crate. 



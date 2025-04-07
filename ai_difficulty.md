**Core Concepts for AI Logic:**

1.  **Card Evaluation:** Assign value to cards (Lowest non-trump < High non-trump < Low trump < High trump).
2.  **Hand Analysis:** What cards does the AI currently hold? (Suits, ranks, trumps).
3.  **Game State Awareness:**
    * How many cards are left in the deck?
    * What cards have been played/discarded (especially trumps)? (Crucial for harder difficulties).
    * How many cards does the opponent have?
4.  **Decision Making (Attack/Defend):** Applying rules based on difficulty.

---

### Easy Difficulty AI ("The Beginner")

**Goal:** Play valid moves, often suboptimal, without much foresight. Prioritizes getting rid of *any* card.

**Attack Logic:**

1.  **Initial Attack:**
    * Always play the lowest-ranking playable card (non-trump preferred, but will play lowest trump if it's the absolute lowest card).
    * If multiple cards of the same lowest rank exist, pick one randomly or the first one found.
    * Does not consider if the opponent is likely to beat it.
2.  **Adding Cards ("Podkidnoy"):**
    * If the defender defends the initial card(s), check hand for any card whose rank matches *any* card already played in the current attack/defense round.
    * Add the *first* matching card found, regardless of its value (might waste a high trump to add).
    * Adds cards one by one as long as possible and allowed by rules (up to defender's hand size limit, usually 6 total attack cards). Does not strategize about *which* card to add.

**Defense Logic:**

1.  **Choosing Defense:**
    * For each attacking card, check if it can be beaten.
    * If beatable with a non-trump: Use the lowest possible non-trump of the same suit.
    * If only beatable with a trump: Use the lowest possible trump card.
    * Doesn't save trumps strategically. Will use a trump on a low non-trump if it's the only option, even if better non-trump defense cards exist for *other* attacking cards.
2.  **Decision to Pick Up:**
    * Evaluates cards one by one. If *any* single attacking card cannot be beaten with the cards currently in hand, immediately decide to pick up *all* cards from the round. Doesn't consider beating some cards first.

**General Strategy:**

* No card counting or tracking of played cards.
* No awareness of opponent's likely hand composition.
* Doesn't prioritize saving trumps or valuable cards.
* Essentially plays reactively based on the immediate situation and lowest available card.

---

### Medium Difficulty AI ("The Basic Player")

**Goal:** Follow basic Durak strategy, avoid obvious blunders, make some effort to manage trumps.

**Attack Logic:**

1.  **Initial Attack:**
    * Prioritize playing the lowest-ranking *non-trump* card first.
    * If holding pairs of non-trumps, might prioritize starting with one from the lowest pair to enable adding the second later.
    * Will only start with a trump if no non-trumps are available, or perhaps if it's a very low trump and the AI has several.
2.  **Adding Cards ("Podkidnoy"):**
    * Prioritize adding matching *non-trump* cards first.
    * If multiple non-trump options, add the lowest one first.
    * Will add matching trumps only if no matching non-trumps are available or if trying to pressure the opponent (e.g., if the defender used a trump).
    * Might stop adding cards even if possible, especially if it means using a high-value card the AI wants to save, or if the defender seems to be defending easily.

**Defense Logic:**

1.  **Choosing Defense:**
    * Prioritize beating non-trump attacks with the lowest possible non-trump card of the correct suit.
    * Use trumps *only if necessary* (no suitable non-trump available) or strategically against a high-value non-trump attacker.
    * If using a trump, use the lowest possible one that beats the attacker. Tries to save higher trumps.
2.  **Decision to Pick Up:**
    * Evaluates all attacking cards *before* playing any defense.
    * If *any* card cannot be beaten, *or* if defending requires using multiple valuable trumps (e.g., two trumps higher than a 10), the AI might decide to pick up to conserve resources.
    * More likely to pick up if the number of attack cards is high (e.g., 4 or more).

**General Strategy:**

* Basic trump management: Tries to save higher trumps for later or for defending against important cards.
* Might track if the main trump card (shown under the deck) has been drawn/played.
* Minimal tracking of other played cards (maybe remembers if opponent picked up specific high cards recently).
* Understands the basic goal of getting rid of cards while not picking up unnecessarily.

---

### Hard Difficulty AI ("The Strategist")

**Goal:** Play strategically, track cards, manage resources optimally, anticipate opponent, and adapt to game phase (early, mid, endgame).

**Attack Logic:**

1.  **Initial Attack:**
    * Considers opponent's hand size and known discarded cards.
    * Chooses attack cards strategically:
        * **Probing:** Start with low non-trumps, especially singles, to see what the defender uses.
        * **Pressure:** Start with cards the AI believes the opponent *might* struggle with (e.g., a suit they seem out of, or a rank they just picked up). May use pairs.
        * **Forcing Trumps:** Might attack with a high non-trump or even a medium trump to force the defender to use a higher trump.
    * Considers the endgame: If the deck is nearly empty, might attack with cards aimed at preventing the opponent from discarding their last few cards.
2.  **Adding Cards ("Podkidnoy"):**
    * Adds cards to maximize pressure based on tracked information.
    * Prioritizes adding ranks the defender has shown weakness in or likely has few of.
    * Uses card counting: Knows which ranks are depleted. If the defender plays a King, and the AI knows the other 3 Kings are out, it won't hold back adding another King if available.
    * Will add trumps strategically to force the defender to use higher trumps or pick up.
    * Knows when *not* to add cards – e.g., if the defender beat the initial attack easily with low cards, adding more might just help them discard junk. Better to end the attack and save cards.

**Defense Logic:**

1.  **Choosing Defense:**
    * Uses the *absolute lowest* possible card to beat each attacker.
    * Conserves trumps meticulously. Avoids using trumps on low/medium non-trump attacks if at all possible.
    * Tracks played cards: Knows which higher cards or trumps might still be held by attackers.
    * Considers the *entire* attack sequence before playing defense.
2.  **Decision to Pick Up:**
    * Makes a strategic decision based on:
        * Cards required to defend vs. their value (especially trumps).
        * Number of cards to potentially pick up vs. hand size limit.
        * Impact on future turns (picking up might give useful cards or overload the hand).
        * Endgame considerations (sometimes picking up is better than losing the last valuable trump).
    * May choose to pick up *even if able to defend*, if the cost of defending (e.g., using the Ace of trumps) is too high for the current situation.

**General Strategy:**

* **Card Counting:** Tracks all played cards, especially trumps and high ranks. Maintains a probability map of remaining cards.
* **Opponent Modeling:** Tries to infer opponent's hand based on their plays, discards, and pickups. Remembers which suits opponent seemed unable to defend.
* **Trump Supremacy:** Understands the critical role of trumps, especially high trumps and in the endgame. Uses them decisively.
* **Endgame Focus:** When the deck is empty, calculates the optimal path to emptying its hand, potentially forcing the opponent to pick up the final cards. Knows exactly what cards are left in play.
* **Adaptive Play:** Adjusts strategy based on whether it's early game (drawing cards) or endgame (no drawing).

---
**Implementation Notes:**

* **State Representation:** The AI needs access to its hand, the trump suit, the cards currently on the table (attack/defense), the number of cards in the deck, the number of cards in the opponent's hand, and (for Hard) a list/set of discarded cards.
* **Card Representation:** Cards need suit and rank, and a way to know if they are trump.
* **Variations:** If implementing rules like "passing the attack," the logic needs to incorporate that decision point.


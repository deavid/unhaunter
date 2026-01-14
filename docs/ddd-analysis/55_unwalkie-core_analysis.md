# DDD Analysis: `unwalkie-core`

## 1. Bounded Context

**Walkie-Talkie Logic & Guidance Domain**. This crate implements the "Director" logic that decides which voice lines
should be played, when, and with what priority.

## 2. Responsibility

- **Prioritization Logic**: Implements the `WalkiePlay` resource, which manages a priority queue for voice lines (e.g.,
  urgent warnings override gentle hints).
- **Repetition Control**: Tracks how many times an event has been played to avoid annoying the player (using
  dice-roll-based skipping).
- **Timing & Pacing**: Calculates the necessary delays between messages based on their priority and the player's current
  activity level.
- **Event Definition**: Defines the `WalkieEvent` enum, describing every possible trigger for a walkie-talkie message.

## 3. Dependencies and Appropriateness

- **Dependencies**: `unwalkie-types`, `unfoundation-core`, `untypes-core`, `ungear-core`, `bevy`, `bevy_platform`,
  `rand`.
- **Appropriateness**: High as a domain logic core. It depends on `unfoundation-core` (for random seeds) and
  `ungear-core` (to know about item usage).

## 4. Encapsulation (Data/Logic Split)

- **Data**: Centralized in the `WalkiePlay` and `WalkieEvent` types.
- **Logic**: Contains the complex algorithms for priority calculation and pacing in the `impl` blocks. It is a "Rich
  Domain Model".

## 5. Semantic & Infrastructure Leakage

- **Semantic Leakage**: Moderate. It has explicit knowledge of many game events (e.g., `GhostHunting`, `LightOn`,
  `EvidenceFound`).
- **Infrastructure Leakage**: Low. Uses Bevy for resource management but the core logic is mathematical.

---

## Technical Debt & Strategic Notes

- **Event Centralization**: The `WalkieEvent` enum is a massive central point of coupling. Ideally, different domains
  would register their own walkie-talkie events, but given the need for centralized prioritization, a single enum is a
  pragmatically acceptable solution for now.
- **The "Director" Role**: This crate essentially acts as a "Game Director" lite. It represents the logical rules of how
  the game's narrator reacts to the player.

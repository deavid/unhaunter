# DDD Analysis: `unwalkie-types`

## 1. Bounded Context

**Walkie-Talkie Metadata Domain**. This is a bottom-level crate designed to be shared between the game engine and
external tools (like the voice generator).

## 2. Responsibility

- **Voice Line Taxonomy**: Defines the `WalkieTag` enum, which categorizes voice lines into semantic types (Hint,
  Guidance, Prodding, Warning, etc.).
- **Audio Metadata**: Defines `VoiceLineData`, the structure used to store the relationship between a voice line's ID,
  its text, and its audio file path.

## 3. Dependencies and Appropriateness

- **Dependencies**: `serde`.
- **Appropriateness**: High. By removing all Bevy dependencies, this crate can be used in command-line tools without
  bringing in the entire game engine.

## 4. Encapsulation (Data/Logic Split)

- **Data**: Pure data structures.
- **Logic**: Zero logic.

## 5. Semantic & Infrastructure Leakage

- **Semantic Leakage**: Low. It defines the categories for communication, which is a domain-level concern.
- **Infrastructure Leakage**: Zero. It is engine-agnostic.

---

## Technical Debt & Strategic Notes

- **Tooling Support**: This crate is essential for the `walkie_voice_generator` tool.
- **Purity**: This is the "cleanest" crate in the workspace, being purely about data representation.

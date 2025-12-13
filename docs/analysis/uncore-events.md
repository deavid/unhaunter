# Crate Analysis: `uncore-events`

## Core Responsibilities

This crate defines all the Bevy `Message` (formerly `Event`) types used for inter-system communication within the `unhaunter` game. Events are a core part of the Bevy ECS paradigm, enabling a decoupled way for different parts of the application to react to changes and actions without direct knowledge of each other. This crate effectively defines the "messages" that flow through the game.

## Public API

The crate exposes a collection of event types, grouped by their domain:

-   **Ambient Sound:**
    -   `AmbientSoundMuteEvent`: Triggers muting of ambient sounds with configurable fade-out, mute duration, fade-in, and volume reduction.

-   **Board & Level Management:**
    -   `BoardDataToRebuild`: Signals that lighting and/or collision data for the board needs to be recomputed.
    -   `LoadLevelEvent`: Initiates the process of loading a new level from a TMX map file.
    -   `LevelLoadedEvent`: Dispatched after a level has been successfully loaded, providing map file path, layers, and floor-level mapping.
    -   `LevelReadyEvent`: Signals that a loaded level is fully prepared and ready for gameplay.
    -   `MapSelectedEvent`: Notifies that a map has been selected (e.g., in a map hub UI).
    -   `RoomChangedEvent`: Triggered when the player enters a new room or a room-related change occurs, potentially opening the van UI.

-   **Gameplay Interactions:**
    -   `GhostInteractionEvent`: Dispatched when a ghost performs an action like `Toggle`, `DoorSlam`, `Throw`, `Nudge`, `Lock`, `TripBreaker`, targeting a specific entity.
    -   `NpcHelpEvent`: Triggered when an NPC interaction occurs, usually for displaying dialogue.
    -   `SoundEvent`: Requests a sound effect to be played at a specific world position with a given volume.

-   **UI Interactions:**
    -   `OnScreenHintEvent`: Triggers the display of a text hint on the screen, typically from a walkie-talkie message.
    -   `TruckUIEvent`: Defines actions originating from the Truck UI, such as `EndMission`, `ExitTruck`, or `CraftRepellent`.

## Dependencies

-   `uncore-board`
-   `bevy`
-   `bevy_platform`

## Architectural Analysis & Notes

### SOLID Principles
-   **Single Responsibility Principle (SRP):** This crate adheres very well to SRP. Its sole responsibility is to define event messages. Each event struct/enum typically represents a single, well-defined occurrence or request within the game.
-   **Open/Closed Principle (OCP):** Events naturally support OCP. New systems can be added to react to existing events without modifying the event definitions themselves, and new event types can be added without modifying existing systems (unless they need to react to the new event).
-   **Liskov Substitution Principle (LSP):** Events are data-only and typically don't have polymorphic behavior in a way that would violate LSP.
-   **Interface Segregation Principle (ISP):** By having many small, specific event types rather than a few large, generic ones, the crate promotes ISP. Systems only need to subscribe to the events relevant to their concerns.
-   **Dependency Inversion Principle (DIP):** Events facilitate DIP by providing an abstraction layer for communication. High-level systems don't depend on low-level system implementations; they both depend on the event abstraction.

### 2D/3D Coupling
-   **Conclusion:** **Low to Moderate Coupling.**
-   **Reasoning:** Most events are abstract representations of actions or state changes and are largely agnostic to 2D or 3D rendering.
    -   `GhostInteractionEvent` includes `destination: Option<Position>`, and `SoundEvent` includes `position: Option<Position>`. Since `Position` (from `uncore-board`) is highly coupled to 2D isometric rendering, this introduces an indirect coupling. If `Position` were refactored to be purely 3D, these events would automatically adapt.
    -   `LoadLevelEvent` and `LevelLoadedEvent` refer to "TMX map file" and `MapLayer` (from `uncore-board`), which are explicitly part of the Tiled 2D isometric map pipeline. This is a moderate, indirect coupling.
-   Overall, the event system itself is robust and abstract. The coupling comes from the data types embedded within certain events, which can be addressed by refactoring those underlying data types.

### Game Logic vs. Engine Logic
-   **Conclusion:** A strong mix, but leaning towards **Engine-level Concepts** with clear pathways to generalization.
-   **Reasoning:**
    -   **Engine-level:** Events like `LoadLevelEvent`, `SoundEvent`, `BoardDataToRebuild`, and even generic `GhostInteractionEvent` (if `GhostInteractionType` were more generic) are fundamental to any game engine. The concept of an `AmbientSoundMuteEvent` is also generally useful.
    -   **Game-specific:** `GhostInteractionType` (with variants like `DoorSlam`, `TripBreaker`), `NpcHelpEvent`, `OnScreenHintEvent`, and `TruckUIEvent` are more specific to *Unhaunter*'s gameplay and UI.
-   For a "ghost game engine," the more game-specific event types could be moved to a `unhaunter-game-events` crate, while the generic ones remain in a core `engine-events` crate. This would further enhance modularity.

### Other Notes
-   **Decoupling Communication:** The event system is a major strength for decoupling different parts of the codebase, which is directly aligned with the user's refactoring goals. Systems can publish events without knowing who consumes them, and systems can subscribe without knowing who publishes them.
-   **Clear Contract:** Each event acts as a clear contract for communication, making the overall flow of information easier to understand and debug.
-   **Dependencies on `uncore-board`:** Events often carry data from other core crates, explaining the dependency on `uncore-board` (for `Position`, `MapLayer`).

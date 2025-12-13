# Crate Analysis: `unnpc`

## Core Responsibilities

`unnpc` handles the interaction with Non-Player Characters (NPCs) in the game. Currently, its scope seems limited to
displaying help dialogs when the player interacts with an NPC.

## Public API

The crate exposes:

- **`plugin` Module**:

  - **`UnhaunterNPCPlugin`**: Registers the NPC help system.

- **`npchelp` Module**:
  - **`app_setup`**: Registers systems for the `GameState::NpcHelp` state.
  - **`NpcUIData` Resource**: Stores the text to be displayed in the dialog.
  - **UI Systems**: `setup_ui` and `cleanup` manage the dialog box rendering.
  - **Input Handling**: `keyboard` handles closing the dialog (Escape or E key).

## Dependencies

- `uncore-foundation`
- `uncore-board`
- `uncore-events`
- `uncore-types`
- `uncore-resources`
- `uncore-components`
- `unstd`
- `bevy`

## Architectural Analysis & Notes

### SOLID Principles

- **Single Responsibility Principle (SRP):** The crate is focused on "NPC Interaction UI". It doesn't handle NPC AI
  (which doesn't seem to exist yet, or is minimal) or movement.
- **Scope:** It's a very small crate, essentially just a UI modal for text.

### 2D/3D Coupling

- **Conclusion:** **Low Coupling.**
- **Reasoning:**
  - **UI-Centric:** It displays a 2D UI overlay (`NpcUI`).
  - **Interaction:** It responds to `NpcHelpEvent`, which is triggered by the interaction system (likely in `ungame` or
    `uncore-systems`). The trigger might be spatial, but the response (this crate) is just UI.

### Game Logic vs. Engine Logic

- **Conclusion:** **Game Logic**.
- **Reasoning:** The specific way NPCs talk to the player (modal dialogs) is a game design choice.

### Other Notes

- **Future Expansion:** If NPCs become more complex (moving, trading, giving quests), this crate would likely expand or
  be split. For now, it's just a "Help Dialog" system.

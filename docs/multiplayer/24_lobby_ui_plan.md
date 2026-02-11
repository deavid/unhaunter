# Plan 24: Lobby UI Polish

## Context

Plan 23 implemented the lobby functionality: protocol (`LobbyWelcome`, `LobbyState`, `StartMission`), connection rework,
`AppState::Lobby`, `unlobby-plugin` crate, and basic logic. It works — host can select map/difficulty with arrow keys,
clients see updates, Start Mission launches the game, Summary returns to lobby.

However, the UI was built from scratch with custom layout and keyboard-only input, ignoring the existing menu
infrastructure in `unmenu-core`. This makes it visually inconsistent with the rest of the game and lacking mouse
support. This plan replaces the lobby UI with one built on the existing pattern system.

## Current State

**What exists in `unlobby-plugin`:**

- `systems/setup.rs` — `app_setup` wires `OnEnter`/`OnExit`/`Update` systems for `AppState::Lobby`
- `systems/ui.rs` — Custom UI: hardcoded 500px left strip, raw `Text` entities, no `MenuItemInteractive`, no mouse
  support, frame-by-frame player list rebuild
- `systems/logic.rs` — `lobby_handle_input_system` (keyboard-only: arrows browse maps/difficulty, Enter starts),
  `lobby_broadcast_state_system` (host broadcasts `LobbyState` every 500ms)
- `plugin.rs` — `UnhaunterLobbyPlugin` delegates to `systems::setup::app_setup`

**What exists in `unmenu-core::templates` that we should use:**

- `create_background` — standard background image
- `create_logo` — top-left logo
- `create_breadcrumb_navigation` — left strip (300px) with title + subtitle, `MenuMouseTracker`
- `create_selectable_content_area` — right panel for selectable items, with `MenuRoot`, `MenuMouseTracker`
- `create_content_item` / `create_content_item_enabled` — list items with `Button`, `Interaction`,
  `MenuItemInteractive`, keyboard+mouse support
- `create_menu_item` — left strip menu items with `Button`, `Interaction`, `MenuItemInteractive`
- `create_help_text` — bottom help bar
- `create_player_status_bar` — player XP/bank bar

**Existing screens to model after:**

- `unified_mission_selection.rs` (in `uncampaign-plugin`) — breadcrumb left, scrollable map list + preview right
- `difficulty_selection.rs` (in `unmaphub-plugin`) — breadcrumb left, difficulty items as `create_content_item`

## Design

### Sub-State Approach

Add a `LobbyScreen` sub-state (like `MapHubState`):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States, Default)]
pub enum LobbyScreen {
    #[default]
    Main,           // Lobby overview: players, current selection, actions
    MapSelection,   // Host browsing maps (full map list UI)
    DifficultySelection, // Host browsing difficulties
}
```

This goes in `untypes-core/src/states.rs` alongside `MapHubState`.

### Screen Layouts

#### LobbyScreen::Main (Lobby Overview)

The "home" screen of the lobby. Both host and client see this.

**Left strip** (breadcrumb-style, 300px):

- Title: "Multiplayer Lobby"
- Three `create_menu_item` entries:
  - "Select Map" — host-only (disabled for clients via `create_content_item_enabled(..., is_host)`)
  - "Select Difficulty" — host-only (same)
  - "Start Mission" — host-only (same)

**Right content area** (the main panel):

- **Map preview thumbnail** — `ImageNode` loaded from `MissionData::preview_image_path`, 16:9 aspect ratio. Falls back
  to `img/placeholder_mission.png` if path is empty or no map selected.
- **Map info text** — display name, location, flavor text. Similar to `format_mission_details` in
  `unified_mission_selection.rs`.
- **Difficulty info** — difficulty name + `difficulty_description()` flavor text.
- **Player list** — colored squares + "Player N" labels. The local player's entry gets a visible marker
  (outlined/highlighted square, or a `► ` prefix, or both) so you know which one is you.

**Help text bar** (bottom): "[Enter]: Confirm | [ESC]: Back to Menu" (host gets the full set).

**Behavior:**

- Host clicks "Select Map" → `LobbyScreen::MapSelection`
- Host clicks "Select Difficulty" → `LobbyScreen::DifficultySelection`
- Host clicks "Start Mission" → broadcast `StartMission`, fire `LoadLevelEvent`, transition out
- Escape → `AppState::MainMenu`
- Client: left strip items are visible but disabled (grayed out). Client sees the same right panel content, updated in
  real-time via `LobbyState` broadcasts.

#### LobbyScreen::MapSelection (Host Only)

Full map browser. Modeled after `unified_mission_selection.rs` but simpler (no deposits, no level gating, no
campaign/custom split).

**Left strip**: `create_breadcrumb_navigation` with "Multiplayer Lobby" / "Select Map".

**Right content area**: Two-column layout (same as `unified_mission_selection`):

- Left column: scrollable map list using `create_content_item` entries. Each shows `display_name`.
  `ScrollableListContainer` + scrollbar.
- Right column: preview image + description text for the currently highlighted map.

**Behavior:**

- Navigate with keyboard (Up/Down) or mouse hover/click
- Enter or click → stores selected map filepath in `LobbyData.selected_map`, returns to `LobbyScreen::Main`
- Escape → returns to `LobbyScreen::Main` without changing selection

**Filtering:** Show non-tutorial, non-campaign maps (custom maps). Same filter as `MissionSelectMode::Custom` in the
unified mission selection.

**Clients:** Stay on `LobbyScreen::Main`. They see the map update live via `LobbyState`. No need to show clients the map
browser.

#### LobbyScreen::DifficultySelection (Host Only)

Difficulty picker. Modeled after `difficulty_selection.rs` in `unmaphub-plugin`.

**Left strip**: `create_breadcrumb_navigation` with "Multiplayer Lobby" / "Select Difficulty".

**Right content area**: Difficulty list as `create_content_item` entries. Non-tutorial difficulties only. When a
difficulty is highlighted, show its `difficulty_description()` text in a description panel (same pattern as the existing
difficulty selection screen).

**Behavior:**

- Navigate with keyboard or mouse
- Enter or click → stores selected difficulty in `LobbyData.selected_difficulty`, returns to `LobbyScreen::Main`
- Escape → returns to `LobbyScreen::Main` without changing selection

**Clients:** Stay on `LobbyScreen::Main`.

### Player List Display

Each player shown as a row:

```
[■] Player 1 (Host)
[►■] Player 2          ← this is you (local player)
[■] Player 3
```

The `■` is a small colored square (can be a text character or a small UI node with `BackgroundColor`). The color comes
from mapping `tint_color_index` to a fixed palette:

```rust
const PLAYER_COLORS: [Color; 8] = [
    // 0: warm white (host default)
    // 1: blue
    // 2: green
    // 3: red
    // 4: yellow
    // 5: purple
    // 6: cyan
    // 7: orange
];
```

This palette should live in a shared location (e.g., `unnet-core/src/resources.rs` next to `LobbyPlayer`, or
`unfoundation-core/src/colors.rs` if it fits there). The same palette will eventually be used by the rendering system to
tint player sprites in-game.

The local player is identified by comparing `LobbyPlayer.id` with `LocalPlayer.0`. Their row gets a visual marker — a
`►` prefix or a highlighted/outlined color square — making it immediately obvious which player you are.

### Player Identity Note

Players don't have names yet. The label format is:

- Host: `Player 1 (Host)`
- Others: `Player 2`, `Player 3`, etc.

The number comes from the player's position in the `LobbyData.players` list (which is ordered by join time, host first).
This is sufficient for now.

## Phases

### Phase A: Sub-State + Color Palette

**Files touched:** | File | Change | |------|--------| | `crates/untypes-core/src/states.rs` | Add `LobbyScreen` enum |
| `crates/unfoundation-core/src/colors.rs` (or `unnet-core/src/resources.rs`) | Add `PLAYER_COLORS` palette |

1. Add `LobbyScreen` with `Main`, `MapSelection`, `DifficultySelection` variants. `Default` = `Main`.
2. Add `PLAYER_COLORS: [Srgba; 8]` constant with 8 distinguishable colors suitable for colored squares on dark
   backgrounds.

**Verification:** `cargo clippy`.

### Phase B: Lobby Main Screen

**Files touched:** | File | Change | |------|--------| | `crates/unlobby-plugin/src/systems/ui.rs` | Rewrite: use
`unmenu-core::templates` | | `crates/unlobby-plugin/src/systems/setup.rs` | Add sub-state init, wire `OnEnter`/`OnExit`
per sub-state | | `crates/unlobby-plugin/src/systems/logic.rs` | Rework input handling to use `MenuItemClicked` events |
| `crates/unlobby-plugin/src/plugin.rs` | Init `LobbyScreen` state |

**Details:**

1. **`plugin.rs`**: Add `app.init_state::<LobbyScreen>()`.

2. **`setup.rs`**: Restructure scheduling:
   - `OnEnter(AppState::Lobby)`: set `LobbyScreen::Main`, spawn camera
   - `OnExit(AppState::Lobby)`: despawn everything, reset `LobbyScreen` to default
   - `OnEnter(LobbyScreen::Main)`: `setup_lobby_main_ui`
   - `OnExit(LobbyScreen::Main)`: `cleanup_lobby_main_ui`
   - `OnEnter(LobbyScreen::MapSelection)`: `setup_map_selection_ui`
   - `OnExit(LobbyScreen::MapSelection)`: `cleanup_map_selection_ui`
   - `OnEnter(LobbyScreen::DifficultySelection)`: `setup_difficulty_selection_ui`
   - `OnExit(LobbyScreen::DifficultySelection)`: `cleanup_difficulty_selection_ui`
   - `Update` (gated on `AppState::Lobby`): `lobby_broadcast_state_system`, `lobby_client_update_system`
   - `Update` (gated on `LobbyScreen::Main`): `lobby_main_handle_clicks`, `lobby_main_update_display`
   - `Update` (gated on `LobbyScreen::MapSelection`): `lobby_map_handle_clicks`, `lobby_map_update_preview`
   - `Update` (gated on `LobbyScreen::DifficultySelection`): `lobby_diff_handle_clicks`, `lobby_diff_update_description`

3. **`ui.rs`**: Complete rewrite.
   - `setup_lobby_main_ui`: Use `create_background`, `create_logo`, build left strip with 3 menu items (disabled for
     clients), build right content area with map preview + info + player list.
   - `cleanup_lobby_main_ui`: Despawn marker entities.
   - `lobby_main_update_display`: Update map preview image, map text, difficulty text, player list. Player list: rebuild
     only when `LobbyData` changes (check `lobby_data.is_changed()`), not every frame.
   - Add new module or section for map selection and difficulty selection sub-screen UIs.

4. **`logic.rs`**: Replace raw `KeyCode` handling with `MenuItemClicked` event reading.
   - `lobby_main_handle_clicks`: Read `MenuItemClicked`, match on menu item identifier → transition to sub-state or
     trigger Start Mission.
   - Keep `lobby_broadcast_state_system` mostly as-is (it's fine).
   - Keep Start Mission logic (broadcast `StartMission`, fire `LoadLevelEvent`, set `cli` fields).

**Verification:** `cargo clippy`. Manual test: lobby main screen looks like other menus, items clickable with mouse,
highlight on hover, Escape goes back to MainMenu.

### Phase C: Map Selection Sub-Screen

**Files touched:** | File | Change | |------|--------| | `crates/unlobby-plugin/src/systems/ui.rs` (or a new
`map_select.rs` module) | Map selection UI | | `crates/unlobby-plugin/src/systems/logic.rs` (or in the new module) | Map
selection input handling |

**Details:**

Modeled closely on `unified_mission_selection.rs::setup_ui`, but stripped down:

- No campaign vs custom mode split — show all non-campaign, non-tutorial maps
- No deposit checks, no level gating, no badges
- On selection: store `map.mission_data.map_filepath` into `LobbyData.selected_map`, transition to `LobbyScreen::Main`
- On escape: transition to `LobbyScreen::Main` (no selection change)
- Use `create_breadcrumb_navigation`, `create_selectable_content_area`, `create_content_item`, scrollbar

**Verification:** `cargo clippy`. Manual test: host clicks "Select Map" in lobby, sees map list with previews, clicks a
map, returns to lobby main with selection updated.

### Phase D: Difficulty Selection Sub-Screen

**Files touched:** | File | Change | |------|--------| | `crates/unlobby-plugin/src/systems/ui.rs` (or a new
`difficulty_select.rs` module) | Difficulty selection UI | | `crates/unlobby-plugin/src/systems/logic.rs` (or in the new
module) | Difficulty selection input handling |

**Details:**

Modeled on `difficulty_selection.rs` in `unmaphub-plugin`:

- List non-tutorial difficulties as `create_content_item` entries
- Show `difficulty_description()` for highlighted item
- On selection: store `difficulty.to_string()` into `LobbyData.selected_difficulty`, transition to `LobbyScreen::Main`
- On escape: transition to `LobbyScreen::Main`

**Verification:** `cargo clippy`. Manual test: host clicks "Select Difficulty", sees difficulty list with descriptions,
picks one, returns to lobby with updated difficulty.

### Phase E: Cleanup and Polish

**Details:**

1. **Remove dead code** from old `ui.rs` — any remnants of the custom layout that weren't cleaned up in Phase B.
2. **Verify font usage** — only use fonts already in use elsewhere (`font_londrina_light`, `font_titillium_regular`,
   `font_titillium_light`). No `font_londrina_regular` or `font_londrina_black` — these cause the Bevy font weight
   bleed-through bug.
3. **Verify camera cleanup** — ensure `LobbyCamera` is despawned on `OnExit(AppState::Lobby)` and doesn't leak into
   MainMenu.
4. **Player list rebuild optimization** — only rebuild when `LobbyData` is changed, using `lobby_data.is_changed()`.
5. **Escape handling from sub-screens** — use `MenuEscapeEvent` from `unmenu-core` rather than raw `KeyCode::Escape`
   checks, for consistency.

**Verification:** `cargo clippy`. Manual test: navigate lobby → map select → back → difficulty → back → start mission →
summary → lobby. Verify no visual glitches, no leaked entities, fonts look correct on MainMenu return.

## File Organization

The current `unlobby-plugin` has 3 source files under `systems/`. With the sub-screens, this will grow. Recommended
structure:

```
crates/unlobby-plugin/src/
  lib.rs                     # mod statements only
  plugin.rs                  # Plugin impl
  systems/
    mod.rs                   # mod statements only
    setup.rs                 # app_setup, system scheduling
    lobby_main.rs            # Main screen: UI setup, update, click handling
    map_select.rs            # Map selection sub-screen
    difficulty_select.rs     # Difficulty selection sub-screen
    broadcast.rs             # lobby_broadcast_state_system (moved from logic.rs)
```

The current `ui.rs` and `logic.rs` are replaced/split into the above. The split keeps each file focused on one screen,
matching the pattern in `unmaphub-plugin` (where `difficulty_selection.rs` is a self-contained module).

## Risks & Considerations

1. **Code duplication with `unified_mission_selection.rs` / `difficulty_selection.rs`:** Intentional. The lobby versions
   are simpler (no deposits, no campaign, no game launch). Trying to parameterize the existing screens would add
   complexity for little gain. Structural duplication through shared templates is acceptable.

2. **Font weight bug:** Avoid `font_londrina_regular` and `font_londrina_black`. Stick to `font_londrina_light` for menu
   text (matching all other screens). The font weight bleed-through is a known Bevy issue to be addressed separately.

3. **Sub-state reset:** When exiting `AppState::Lobby`, the `LobbyScreen` sub-state must be reset to `Main`. Otherwise
   re-entering the lobby could land on a stale sub-state.

4. **`LobbyScreen::MapSelection` / `DifficultySelection` on client:** Clients should never enter these sub-states. The
   menu items are disabled for clients, and the sub-state transitions are guarded by host-only checks.

5. **`LobbyState` broadcast during sub-screens:** The broadcast system runs on `AppState::Lobby` regardless of
   `LobbyScreen`. When the host is browsing maps, the broadcast still runs, and `LobbyData` updates as the host makes a
   selection. Clients see the update when the host confirms.

6. **Player color palette:** The palette is defined now for lobby display. Later it will also drive in-game sprite
   tinting. The palette just needs to be 8 distinguishable colors on a dark background.

## Estimated Scope

- **Phase A:** ~15 lines (sub-state enum + color palette)
- **Phase B:** ~300 lines (lobby main screen rewrite — the biggest piece)
- **Phase C:** ~200 lines (map selection, stripped-down version of unified_mission_selection)
- **Phase D:** ~150 lines (difficulty selection, stripped-down version of difficulty_selection)
- **Phase E:** ~20 lines (cleanup, font fixes)

Total: ~700 lines of changed/new code. Most of Phase B and C.

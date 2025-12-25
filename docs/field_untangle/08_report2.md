# Gear Refactor Audit Report: Phase 2

**Date:** December 25, 2025
**Status:** The refactor successfully transitions data structures to ECS, but **gameplay logic connection points were severed** during the migration. The bugs observed are not merely "glitches" but missing implementation bridges between the new systems.

## 1. Investigation of Reported Bugs

### Bug A: "Left hand issues / Difficulty assigns left to right"
**Verdict:** 🛑 **Confirmed Regression (Hardcoded Spawning)**

The difficulty system defines starting loadouts (e.g., Tutorial 1 gives you a Flashlight and Thermometer). The refactor **deleted the logic that reads this** and replaced it with hardcoded debugging lines.

*   **Evidence:** In `crates/unmapload/src/entity_spawning.rs`:
    ```rust
    // OLD (Deleted):
    // .insert(PlayerGear::from_playergearkind(p.difficulty.0.player_gear.clone()))

    // NEW (Current):
    let flashlight = p.gear_registry.spawn(commands, GearKind::Flashlight);
    let emf_meter = p.gear_registry.spawn(commands, GearKind::EMFMeter);

    let player_gear = PlayerGear {
        right_hand: Some(flashlight), // <--- HARDCODED
        inventory: vec![emf_meter],   // <--- HARDCODED
        ..default()                   // <--- Left hand defaults to None
    };
    ```
*   **The Fix:** Restore the logic that iterates over `p.difficulty.0.player_gear`. Since `PlayerGearKind` contains `GearKind` enums, iterate that struct and call `p.gear_registry.spawn()` for each item, then assign the resulting Entity ID to the correct slot in `PlayerGear`.

### Bug B: "Thermometer does not react to [R]"
**Verdict:** 🛑 **System Ordering Race Condition**

The Flashlight works because it manually checks for the `Triggered` component in its own specific system. The Thermometer relies on a generic `gear_trigger_handler`.

*   **The Mechanism:**
    1.  Input System adds `Triggered` component.
    2.  `gear_trigger_handler` flips the `Toggleable` bool.
    3.  `clear_trigger_handler` removes `Triggered` (in PostUpdate).
*   **The Failure:** Bevy systems run in parallel unless explicitly ordered. If `gear_trigger_handler` runs **before** the Input System adds the component, the Thermometer ignores the input. The `Triggered` component is then cleaned up in `PostUpdate` before the handler ever sees it in the next frame.
*   **Why Flashlight Works:** `update_flashlight` has specific logic: `if triggered.is_some() { ... }`. It likely happens to run *after* input handling in the schedule by chance, or handles state differently.
*   **The Fix:** Enforce system ordering.
    ```rust
    app.add_systems(Update, gear_trigger_handler.after(item_trigger_system));
    ```

### Bug C: "Truck UI odd behaviors"
**Verdict:** ⚠️ **Visual Sync Issue**

The Truck UI logic was ported to query entities (`q_gearkind`, `q_sprite`).
*   **Issue:** The UI updates are relying on `q_sprite`. If the `Thermometer` entity hasn't had its `update_thermometer` system run yet (to set the initial sprite ID), the UI might be reading `GearSpriteID::None` or a default value for one frame, or failing to update icons when swapping hands because the ECS hierarchy updates lag behind the logic.
*   **Entity Persistence:** When you move an item from Inventory to Van in the UI, the current code **despawns** the entity and spawns a new one. This destroys any state (e.g., recorded audio, battery level).
    *   *Observation:* `crates/untruck/src/loadoutui.rs`: `commands.entity(e).despawn();`
    *   *Correction:* In the "Van" (TruckGear), items should probably just be entities stored in the `TruckGear` resource (vector of entities), preserving their state. The current refactor treats the Van as a "Creative Mode" infinite spawner for some items, but a void for others.

---

## 2. Unreported Critical Regressions

### 1. Ghost Interaction is Broken
**Severity:** Critical
The Ghost Interaction System (GIS) relies on the component `InteractableByGhost` to throw or nudge items.
*   **Old System:** `Behavior` component added `InteractableByGhost`.
*   **New System:** `GearSpawnerRegistry` builds entities.
*   **The Gap:** `crates/ungearitems/src/registration.rs` does **not** add `InteractableByGhost` to any item.
*   **Consequence:** The Ghost cannot see, throw, or interact with any equipment. It will ignore them completely.

### 2. Physics/Collision Missing
**Severity:** High
When items are dropped, `grabdrop.rs` adds `FloorItemCollidable`. However, standard physics/collision components (like `Collision` from `uncore-board`) might be missing if they aren't added by the `GearSpawnerRegistry`.
*   *Check:* Does `FloorItemCollidable` handle actual physics, or just the logic for "can be picked up"? If the ghost tries to target it via GIS, it needs a valid `Position` (which it has), but if it relies on other board components, they are gone.

### 3. Flashlight Thermal Simulation Simplification
**Severity:** Minor/Design Choice
The old `Flashlight` struct simulated `inner_temp` and `heatsink_temp`.
*   **Analysis:** The new `Flashlight` component in `uncore-components` only has `power` and `color`. The `update_flashlight` system in `ungearitems` *does* implement heat logic, but it stores temp on the component.
*   **Finding:** Actually, `Flashlight` component definition in `crates/ungearitems/src/components/flashlight.rs` *does* still have `inner_temp`.
*   **Wait:** There is a duplicate definition!
    1. `crates/uncore-components/src/components/mod.rs` defines a generic `Flashlight` struct.
    2. `crates/ungearitems/src/components/flashlight.rs` defines a specific `Flashlight` struct.
*   **Conflict:** The registry adds the **generic** `uncore_components::Flashlight` AND the **specific** `ungearitems::...::Flashlight`.
*   **Risk:** The lighting system (`unlight/maplight.rs`) queries `(&Flashlight, &Toggleable)`. Which `Flashlight`? The imports show it is using `uncore_components::Flashlight`. But the logic in `update_flashlight` operates on `ungearitems::components::flashlight::Flashlight`.
*   **Result:** `update_flashlight` calculates heat and updates its own `Flashlight` component, but **never updates the `uncore_components::Flashlight` component** that the renderer uses. The renderer will see static power/color values.

---

## 3. Implementation Plan for AI Fixer

This plan outlines the steps to finalize the refactor and address the identified regressions.

### Phase 1: Fix Core Gameplay Loop
1.  **Restore Difficulty Loadouts:**
    *   Modify `crates/unmapload/src/entity_spawning.rs`.
    *   Iterate `p.difficulty.0.player_gear` (Left, Right, Inventory).
    *   Use `p.gear_registry.spawn()` for each kind and assign to `PlayerGear`.
2.  **Fix Component Duplication (Flashlight):**
    *   **Problem:** `Flashlight` exists in both `uncore-components` (used by render) and `ungearitems` (used by logic).
    *   **Fix:** `update_flashlight` system must write the calculated power/color into the `uncore_components::Flashlight` component so `unlight` can see it. Or better, merge them if possible, but writing the data sync is safer for now.
3.  **Fix Trigger Race Condition:**
    *   In `crates/ungear/src/systems.rs`, update `app_setup`:
    *   `app.add_systems(Update, gear_trigger_handler.after(item_trigger_system));`

### Phase 2: Restore Ghost Interaction
1.  **Update Registry:**
    *   In `crates/ungearitems/src/registration.rs`, add `InteractableByGhost` to all throw-able items (Flashlight, EMF, etc).
    *   Add `FloorItemCollidable` (if meant to be collidable immediately, though usually added on drop).

### Phase 3: Cleanup Truck UI
1.  **Persistence:** The `LoadoutButton::Van` logic spawns *new* entities. This is acceptable for a "Shop" style inventory, but if the intention is persistent gear, the `TruckGear` resource needs to hold actual Entities, and the UI should just move those Entity IDs around instead of despawning/respawning.
2.  **Left/Right Hand Logic:** Ensure `TruckGear` initialization (in `truckgear.rs`) iterates the difficulty settings correctly to populate the van.

## Summary
The code is clean, but the **wiring is loose**. The system logic (`update_flashlight`) is disconnected from the render logic (`unlight`), and the initialization logic (`spawn_player`) is disconnected from the configuration logic (`undifficulty`). Connect these wires, and the refactor will be a success.

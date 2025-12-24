# Field Untangling: Final Recommendation & Roadmap

## 1. Investigation Summary

We have analyzed the current "God Bottlenecks" in the `Gear` system and proposed a Bevy-native ECS architecture.

### The Current State (The Problem)
*   **`GearKind` (Enum):** A "Universal Dictionary" that forces the engine to know about every possible item.
*   **`GearUsable` (Trait):** A "God Trait" that mixes UI, Logic, and specific item features (Flashlight power, Liquid capacity) into one interface.
*   **`Gear` (Struct):** A wrapper that couples the Enum and the Trait, making it impossible to have "just a flashlight" without the overhead of the entire gear system.

### The Proposed State (The Solution)
*   **Entities as Gear:** A piece of gear is just an Entity with components.
*   **Components as Features:**
    *   Instead of `GearUsable::power()`, use `#[derive(Component)] struct Flashlight { power: f32 }`.
    *   Instead of `GearUsable::get_display_name()`, use `#[derive(Component)] struct ItemName(String)`.
*   **Systems as Logic:**
    *   `flashlight_system` queries for `&Flashlight` and updates light sources.
    *   `ui_system` queries for `&ItemName` and draws text.

## 2. Addressing Key Concerns

### 2.1 Exhaustiveness & Sets
*   **Concern:** How do we list "all gear" or "truck gear" without an Enum?
*   **Solution:** **Composition & Prefabs**.
    *   The "Truck" is a container entity. Anything inside it is "Truck Gear".
    *   "Starting Gear" is defined by a Registry of *functions* (Bundles), not a list of Enum variants.
    *   **Local Enums:** Mods can still use `enum ClassicGear` internally to organize their own assets, but they expose them to the engine as generic components.

### 2.2 Serialization
*   **Finding:** No `Serialize` traits were found on `Gear` or `GearUsable`.
*   **Implication:** We are not breaking existing save files because they likely don't exist (or don't save deep gear state).
*   **Future-Proofing:** ECS is *better* for serialization via `bevy_reflect` and `DynamicScene` if we ever need it.

## 3. The "Shared-Mod" Ecosystem

This refactor enables the "Shared-Mod" architecture the user desires:
1.  **`uncore-*` (Engine):** Defines `Flashlight`, `Thermometer`, `EvidenceSensor` components. Knows *nothing* about specific items.
2.  **`mod_classic_gear`:** Depends on `uncore`. Spawns entities with `Flashlight` + `Sprite`.
3.  **`mod_scifi_gear`:** Depends on `uncore`. Spawns entities with `Flashlight` + `HologramSprite`.
4.  **`mod_hardcore_mode`:** Depends on `mod_classic_gear`. Tweaks the `Flashlight` component values of the classic items.

## 4. Implementation Roadmap (Phase 1)

Since "Phase 2" (Difficulty Enum) is complete, we can proceed with Phase 1 when ready.

1.  **Component Definition:** Create granular components in `uncore-components` (or similar).
    *   `ItemName`, `ItemDescription`, `ItemSprite`.
    *   `Flashlight`, `Battery`, `EvidenceSensor`.
2.  **System Migration:**
    *   Port `GearUsable::update` logic into individual systems.
    *   Port `GearUsable::get_display_name` logic into UI systems.
3.  **Deprecation:**
    *   Mark `Gear`, `GearKind`, and `GearUsable` as deprecated.
    *   Refactor `untruck` and `unplayer` to use `Query<Entity, With<ItemName>>` instead of `Vec<Gear>`.

## 5. Conclusion

The investigation is complete. The proposed ECS architecture solves the "Type Hell" and "God Bottleneck" issues while maintaining the ability to have structured sets of gear via composition.

**Status:** READY FOR IMPLEMENTATION.

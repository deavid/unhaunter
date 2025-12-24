# Field Untangling: Detailed Implementation Plan

**Target Audience:** Junior to Mid-Level Rust Developer
**Goal:** Refactor `Gear` from a Struct/Enum/Trait architecture to a pure Bevy ECS Component architecture.
**Constraint:** Minimize breakage, ensure feature parity, and prepare for "Shared-Mod" capability.

---

## 1. Executive Summary

Currently, a piece of gear is a `struct Gear` containing a `GearKind` enum and a `Box<dyn GearUsable>`. This couples the engine to every specific item.
We will transform `Gear` into an **Entity** composed of granular components (`ItemName`, `Flashlight`, `Weight`, etc.).

**The Big Shift:**
*   **Old:** `player.gear.left_hand.kind == GearKind::Flashlight`
*   **New:** `world.get::<Flashlight>(player.gear.left_hand_entity).is_some()`

---

## 2. Prerequisites & Setup

### 2.1 Create `uncore-components`
**Critical Check:** It appears `crates/uncore-components` exists as a folder but might be missing `Cargo.toml` or `lib.rs`.
*   **Action:** Initialize the crate properly.
    *   Create `crates/uncore-components/Cargo.toml`.
    *   Create `crates/uncore-components/src/lib.rs`.
*   **Dependencies:** `bevy`, `uncore-types`.

### 2.2 Define the Granular Components
Create the following components in `uncore-components`. Deriving `Component`, `Debug`, `Clone`, `Reflect` is recommended.

*   **Identity:**
    *   `ItemName(pub String)`
    *   `ItemDescription(pub String)`
    *   `GearSprite(pub GearSpriteID)`
*   **Functionality:**
    *   `Flashlight { power: f32, color: Color, enabled: bool }`
    *   `Battery { level: f32, max: f32, drain_rate: f32 }`
    *   `EvidenceSensor { evidence_type: Evidence, range: f32 }`
    *   `LiquidContainer { content: Option<LiquidType>, quantity: f32, max: f32 }`
    *   `Toggleable { is_on: bool }`
    *   `Electronic { sensitivity: f32 }` (for EMI interference)
    *   `Handheld` (Marker for items that can be held)

---

## 3. Phase 1: The Bridge (The Registry)

We cannot delete `GearKind` yet because `Difficulty` and `SaveFiles` might rely on it. We need a way to translate `GearKind` -> `Entity`.

### 3.1 The `GearSpawner` Resource
Create a resource that knows how to spawn entities for each `GearKind`.

```rust
// In ungear/src/spawner.rs
#[derive(Resource, Default)]
pub struct GearSpawnerRegistry {
    // Maps a GearKind to a function that adds components to an entity
    builders: HashMap<GearKind, Box<dyn Fn(&mut EntityCommands) + Send + Sync>>,
}

impl GearSpawnerRegistry {
    pub fn register<F>(&mut self, kind: GearKind, builder: F) 
    where F: Fn(&mut EntityCommands) + Send + Sync + 'static { ... }
    
    pub fn spawn(&self, commands: &mut Commands, kind: GearKind) -> Entity {
        let mut entity_cmd = commands.spawn(GearMarker); // Marker component
        if let Some(builder) = self.builders.get(&kind) {
            (builder)(&mut entity_cmd);
        }
        entity_cmd.id()
    }
}
```

### 3.2 Registering Existing Gear
In `ungearitems`, create a plugin that registers all current items.
*   **Task:** For `GearKind::Flashlight`, register a builder that adds `ItemName("Flashlight")`, `Flashlight { ... }`, `Toggleable`, etc.
*   **Note:** You are essentially porting the `default()` values from the old structs to these new component bundles.

---

## 4. Phase 2: The Great Refactor (Breaking Changes)

This is the most difficult phase. We change how gear is stored.

### 4.1 Update `PlayerGear` Component
*   **File:** `crates/ungear/src/components/playergear.rs`
*   **Change:**
    ```rust
    pub struct PlayerGear {
        pub left_hand: Option<Entity>,  // Was Gear
        pub right_hand: Option<Entity>, // Was Gear
        pub inventory: Vec<Entity>,     // Was Vec<Gear>
    }
    ```
*   **Impact:** This will break `unplayer`, `untruck`, `unui`, and `uninteraction`.

### 4.2 Update `TruckGear` Resource
*   **File:** `crates/untruck/src/truckgear.rs`
*   **Change:** `pub inventory: Vec<Entity>`
*   **Migration:** Update `TruckGear::from_difficulty` to use the `GearSpawnerRegistry` (via `Commands`) instead of `Gear::from_gearkind`.
    *   *Blocker:* `from_difficulty` might not have access to `Commands` if it's just a helper function. It needs to become a System or take `&mut Commands`.

### 4.3 Fix Compilation Errors (The Slog)
You will encounter hundreds of errors. Group them by category:

*   **Accessing `gear.kind`:**
    *   *Fix:* You can't. You must query the entity.
    *   *Pattern:* `if let Ok(name) = name_q.get(entity) { ... }`
*   **Calling `gear.update()`:**
    *   *Fix:* Delete the call. Logic moves to systems (Phase 3).
*   **UI Rendering:**
    *   *Fix:* The UI currently iterates `Vec<Gear>`. Change it to iterate `Vec<Entity>` and use `Query<(&ItemName, &GearSprite)>` to get the data to draw.

---

## 5. Phase 3: Logic Migration (System-by-System)

Now that the data structure is `Entity`-based, we need to move the behavior.

### 5.1 The Flashlight System
*   **Old:** `GearUsable::update` handled light.
*   **New:**
    ```rust
    fn update_flashlights(
        mut q: Query<(&Flashlight, &Toggleable, &mut PointLight)>,
    ) {
        for (fl, toggle, mut light) in &mut q {
            light.intensity = if toggle.is_on { fl.power } else { 0.0 };
            light.color = fl.color;
        }
    }
    ```

### 5.2 The Interaction System (Triggers)
*   **Old:** `gear.set_trigger(gs)`
*   **New:** Use Bevy 0.17 **Observers**.
    *   Define event `struct UseItem(pub Entity);`
    *   Trigger it when player clicks.
    *   Observer: `commands.trigger_targets(ToggleEvent, item_entity);`
    *   Component Observer: `impl Toggleable { fn on_toggle(&mut self) { self.is_on = !self.is_on; } }`

### 5.3 The UI System
*   **Old:** `gear.get_status()`
*   **New:** A system that queries specific components (`Thermometer`, `EMFMeter`) and updates a generic `StatusText` component on the item, which the UI then reads.

---

## 6. Phase 4: Cleanup

Once all logic is ported:
1.  Delete `struct Gear`.
2.  Delete `trait GearUsable`.
3.  Delete `crates/ungearitems/src/from_gearkind.rs`.
4.  (Optional) Move `GearKind` into a "Legacy" or "ClassicMod" folder, as the engine no longer strictly needs it (though `Difficulty` might still use it as a key).

---

## 7. Risk Assessment

### 7.1 The "Null" Entity
*   **Risk:** `Option<Entity>` can be `None`. Old code assumed `Gear::default()` (NoneGear) was always present.
*   **Mitigation:** Ensure `PlayerGear` logic handles `None` gracefully. Or use a "Null Entity" (not recommended). `Option<Entity>` is cleaner.

### 7.2 Syncing Transforms
*   **Risk:** When an item is in hand, it needs to move with the player.
*   **Solution:** Use Bevy's hierarchy (`commands.entity(player).add_child(item)`). This handles transform propagation automatically.
    *   *Note:* The old system might have manually calculated positions. Parenting is much better.

### 7.3 Performance
*   **Risk:** Querying components for every inventory slot every frame.
*   **Reality:** ECS is designed for this. It will likely be faster than the virtual method calls of `Box<dyn GearUsable>`.

---

## 8. Final Advice for the Implementer

*   **Don't do it all at once.**
*   Start by creating the `Flashlight` component and attaching it to the old `Gear` struct (if possible) or just side-by-side.
*   **The "Big Break" (Phase 2) is unavoidable.** Set aside a day where the code will not compile.
*   **Tests:** Write a test that spawns a Flashlight entity and asserts it has the `Flashlight` component.

Good luck. This refactor is the key to a modular, data-driven future.

# Field Untangling: A Bevy-Native Vision

## 1. Deconstructing the "God" Architecture

The current architecture is built on "God" structures (`BoardData`, `HauntState`, `GearStuff`) and "Fat Traits" (`GearUsable`). This design is essentially an Object-Oriented pattern forced into an ECS framework, which leads to the "Vocabulary God" and "Dependency Loop" problems.

To solve this in a Bevy-native way (Bevy 0.17), we must move from **Methods on Objects** to **Systems on Components**.

---

## 2. The "Bevy Expert" Redesign

### 2.1 Granular Field Resources
Instead of a single `BoardData` struct containing every possible simulation field, we treat each field as a first-class Bevy `Resource`.

*   **The Engine (`uncore-board`):** Provides the `BoardDimensions` (size, origin) and the `CollisionField`.
*   **The Game (`unfog`, `unghost`):** Each plugin registers its own field resources.
    *   `TemperatureField(Array3<f32>)`
    *   `MiasmaField(MiasmaGrid)`
    *   `EMFField(Array3<f32>)`
*   **Benefit:** Systems only request the fields they actually simulate or sense. If a game mode doesn't use Miasma, the `MiasmaField` resource is never even initialized.

### 2.2 Gear as Entities, not Trait Objects
The `Box<dyn GearUsable>` pattern is the root of the `GearStuff` bottleneck. In a Bevy-native design, a "Thermometer" is not a struct with methods; it is an **Entity** with **Components**.

*   **Components:**
    *   `Thermometer { current_reading: f32 }`
    *   `Flashlight { on: bool, battery: f32 }`
    *   `GearInfo { name: String, description: String }`
*   **Systems:** Instead of a universal `update` method, we have specialized systems:
    ```rust
    fn thermometer_system(
        mut q: Query<(&Position, &mut Thermometer), With<HeldByPlayer>>,
        temp_field: Res<TemperatureField>,
    ) {
        for (pos, mut thermometer) in q.iter_mut() {
            thermometer.current_reading = temp_field.sample_at(pos);
        }
    }
    ```
*   **Benefit:** The `thermometer_system` *only* depends on `TemperatureField`. It doesn't know or care about `Miasma` or `HauntState`. This completely eliminates the need for a "Fat" `GearStuff` context.

### 2.3 Observer-Based Interaction (Bevy 0.17)
Instead of `set_trigger(&mut GearStuff)`, we use Bevy's new `Trigger` and `Observer` system.

*   **The Action:** When the player presses the "Use" key, the input system fires a `Trigger`:
    ```rust
    commands.trigger_targets(ActivateGear, held_entity);
    ```
*   **The Reaction:** Each gear type has its own `Observer`:
    ```rust
    fn on_flashlight_activate(
        trigger: Trigger<ActivateGear>,
        mut q: Query<&mut Flashlight>,
    ) {
        if let Ok(mut flashlight) = q.get_mut(trigger.entity()) {
            flashlight.on = !flashlight.on;
        }
    }
    ```

### 2.4 Honest Simulation (Emitters)
Currently, gear "cheats" by reading the ghost's internal state. A Bevy-native approach uses an **Emitter/Sensor** model.

*   **The Ghost:** Is an entity with `Emitter` components.
    *   `TemperatureEmitter { strength: -10.0 }`
*   **The Simulation:** A system in the `unghost` crate reads all `TemperatureEmitter`s and updates the `TemperatureField`.
*   **The Sensor:** The Thermometer entity reads the `TemperatureField`.
*   **Benefit:** The gear is now an "Honest Sensor." It doesn't know a ghost exists; it only knows the room is cold. This allows the engine to support any number of "Threats" (ghosts, broken pipes, open windows) that affect the same fields.

---

## 3. Solving the "Vocabulary God"

The `GearKind` and `GearSpriteID` enums in `uncore-foundation` are "Universal Dictionaries" that force recompiles.

*   **The Solution:** Move to **Data-Driven Definitions**.
    *   Instead of an enum, use a `Component` to store the sprite handle and index: `GearVisual { atlas: Handle<TextureAtlasLayout>, index: usize }`.
    *   The "Game" layer (where the assets live) is responsible for attaching these components when spawning the gear.
    *   The "Engine" foundation only needs to know that an entity *can* have a visual; it doesn't need to know what a "Thermometer" is.

---

## 4. Summary of the Shift

| Concept | Current (OO-Style) | Bevy-Native (ECS-Style) |
| :--- | :--- | :--- |
| **Gear** | `Box<dyn GearUsable>` | Entity with Components |
| **Context** | `GearStuff` (Fat Param) | Granular System Queries |
| **Fields** | `BoardData` (Fat Struct) | Individual `Resource`s |
| **Logic** | `gear.update()` | `fn gear_system()` |
| **Trigger** | `gear.set_trigger()` | `Trigger<ActivateGear>` |
| **Ghost** | `HauntState` (Global) | Entity with `Emitter`s |

## 5. Conclusion: The "Clean" Path
This design doesn't just "shuffle" the code; it **untangles** it. By moving to a pure ECS model, we leverage Bevy's scheduler to handle dependencies automatically. The "Engine" becomes a set of generic systems for spatial management and inventory linking, while the "Game" becomes a collection of independent plugins that provide specific fields, emitters, and sensors.

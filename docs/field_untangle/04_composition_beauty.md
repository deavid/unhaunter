# Field Untangling: Exhaustiveness, Sets, and the Beauty of Composition

## 1. The "Exhaustiveness" Myth: Match vs. Systems

The primary argument for the "Universal Enum" is compile-time exhaustiveness. While a `match` statement ensures you've handled every variant, it creates a **Rigid Hierarchy**.

### 1.1 The Enum Way (Rigid)
```rust
match gear.kind {
    GearKind::Thermometer => update_thermometer(gear),
    GearKind::EMFMeter => update_emf(gear),
    // Compiler error if you forget one.
}
```
*   **Problem:** This `match` must live in a crate that knows about *every* gear type. This is the "Vocabulary God."

### 1.2 The Bevy Way (Fluid)
In a pure ECS model, "Exhaustiveness" is handled by **System Composition**.
```rust
fn thermometer_system(q: Query<&Thermometer>) { /* ... */ }
fn emf_system(q: Query<&EMFMeter>) { /* ... */ }
```
*   **The Trade-off:** You lose the compiler error if you forget a system. However, you gain **Infinite Extensibility**. A new mod can add a "Holy Water" component and its own `holy_water_system` without touching a single line of engine code.
*   **Legibility:** Instead of one 500-line `match` statement, you have 20 small, 25-line systems. Each system is perfectly legible because it only cares about its own data.

---

## 2. Handling "Sets of Gear" (Difficulty & Truck)

The user asked: "How do we define the set of gear for a difficulty or the truck without an enum?"

### 2.1 The "Prefab" Pattern
In Bevy, a "Set of Gear" is a collection of **Bundles** or **Entity Spawners**.

**The Engine Foundation:**
```rust
#[derive(Resource, Default)]
pub struct StartingGearRegistry {
    pub sets: HashMap<Difficulty, Vec<fn(&mut Commands)>>,
}
```

**The "Shared-Mod" Plugin:**
```rust
fn plugin(app: &mut App) {
    app.world_mut()
        .resource_mut::<StartingGearRegistry>()
        .sets
        .entry(Difficulty::Tutorial1)
        .or_default()
        .push(|cmd| { cmd.spawn(ThermometerBundle::default()); });
}
```
*   **Simplicity:** The engine doesn't know what a Thermometer is. It just knows how to execute a list of "spawners."
*   **Legibility:** The mod's code clearly states: "I am adding a Thermometer to Tutorial 1."

### 2.2 The Truck Inventory
The Truck UI currently loops over a `Vec<Gear>`. In a Bevy-native design, the Truck is just a **Container Entity**.
*   **The Set:** The "Truck Gear" are simply entities with a `InTruck` component.
*   **The UI:** `Query<(&GearInfo, &GearVisual), With<InTruck>>`.
*   **Exhaustiveness:** The UI automatically shows *everything* that is in the truck, regardless of which mod provided it.

---

## 3. The "Shared-Mod" Beauty: Local Enums

We can keep the "Beauty of Enums" where it belongs: **Inside the Mod**.

If a mod provides 10 items in one spritesheet, it *should* use an enum for those 10 items internally.
```rust
// INSIDE THE MOD (Private)
enum ClassicGear { Thermometer, EMFMeter, ... }

impl ClassicGear {
    fn get_sprite_index(&self) -> usize { /* match here */ }
}
```
The mod uses this enum to implement its systems and map its visuals. But it **exports** its gear to the engine as generic components.

### 3.1 Mapping Evidence (The "Honest" Way)
Instead of a global `match gear_kind => evidence_type`, we use a component:
```rust
// In the Engine
#[derive(Component)]
pub struct EvidenceSensor(pub Evidence);

// In the Mod
commands.spawn((
    ThermometerBundle::default(),
    EvidenceSensor(Evidence::FreezingTemp),
));
```
Now, the Journal UI or the HUD doesn't need to know about `GearKind`. It just queries for `EvidenceSensor`.

---

## 4. Conclusion: Composition over Classification

By moving from "What is this object?" (Enum) to "What can this object do?" (Components), we achieve:
1.  **Perfect Decoupling:** The engine foundation is 100% agnostic.
2.  **Modularity:** Multiple "Shared-Mods" can contribute gear, fields, and difficulty settings simultaneously.
3.  **Legibility:** Code is organized into small, focused systems rather than giant branching logic.

The "Exhaustiveness" we lose at the compiler level is replaced by the **Robustness** of a system that cannot break when new types are added.

# Field Untangling: Preliminary Report

## 1. The Core Conflict: "Fat Traits" vs. "Decoupled Simulation"

The current architecture suffers from a fundamental tension between the `GearUsable` trait pattern and the goal of a decoupled, "Paranormal-Agnostic" engine.

### 1.1 The `GearStuff` Bottleneck
The `GearStuff` struct is a `SystemParam` that acts as a "Universal Context" for all gear. Because every gear's `update` method receives the same `GearStuff`, this struct must contain every resource that *any* gear might ever need.
*   **Dependency Magnet:** If a "Miasma Detector" needs `MiasmaGrid`, `GearStuff` must include `Res<MiasmaGrid>`. This forces the `ungear` crate (engine) to depend on the `unfog` crate (game logic), creating circular dependencies.
*   **Static Requirements:** Bevy's `SystemParam` requires all resources to be known at compile-time. We cannot dynamically add a "Miasma" resource to `GearStuff` only when the `unfog` plugin is active.

### 1.2 The "Leaky" Simulation
Currently, gear items like the Thermometer or EMF Meter are "cheating." Instead of sensing the environment (Fields), they are reading the ghost's internal state (`HauntState`) directly.
*   **Example:** The Thermometer checks `gs.haunt_state.ghost_dynamics.freezing_temp_clarity` instead of just reading the local temperature from a field.
*   **Consequence:** This bypasses the simulation layer, making it impossible to have a generic engine where different "Threats" (not just ghosts) can affect the same fields.

---

## 2. Investigation of Proposed Solutions

### 2.1 The "Spreadsheet Processor" Model (`BoardData`)
The feedback suggests turning `BoardData` into a "Spreadsheet Processor." This means moving away from hardcoded fields (`pub miasma: MiasmaGrid`) to a dynamic registry of primitive data.

*   **The "Primitive Language" Approach:** Instead of `MiasmaGrid` (a complex struct), `BoardData` would store named `Array3<f32>` or `Array3<Vec2>` buffers.
    *   `unfog` registers a scalar field named `"miasma_pressure"`.
    *   `ungearitems` reads from the scalar field named `"miasma_pressure"`.
    *   **Result:** Both crates depend on `uncore-board` (the "Spreadsheet"), but they don't need to know about each other.
*   **Avoiding "Type Hell":** By restricting the registry to a few primitive types (f32, Vec2, Vec3, bool), we avoid the need for complex downcasting (`dyn Any`). The gear simply asks for "the float field named X."

### 2.2 The "Bevy Resource per Field" Approach
The user suggested letting each plugin register its own Bevy resources.
*   **Pros:** Very idiomatic Bevy. Clean isolation.
*   **Cons:** Reintroduces the `GearStuff` problem. If `MiasmaGrid` is a separate resource, `GearStuff` still needs to know about it to pass it to the `GearUsable` trait.

### 2.3 The "Component-Based Gear" Alternative
To truly solve the dependency loop, we may need to move away from the `Box<dyn GearUsable>` pattern for gear logic.
*   **The Pattern:** Each gear is an Entity. A "Miasma Detector" has a `MiasmaDetector` component.
*   **The Logic:** The `unfog` crate provides a system: `update_miasma_detectors(Query<(&Position, &mut MiasmaDetector)>, Res<MiasmaGrid>)`.
*   **Result:** This system *only* depends on the resources it actually needs. The engine doesn't need to know about it. The "Engine" only manages the inventory (which entity is in which hand).

---

## 3. Preliminary Findings & Recommendations

1.  **The `GearUsable` trait is the primary blocker.** Its requirement for a unified `GearStuff` context makes decoupling impossible without resorting to `dyn Any` "Type Hell" or exclusive `World` access.
2.  **Field Generalization is viable via "Primitive Buffers".** `BoardData` should provide a registry of named `Array3<T>` where `T` is a primitive. This allows the simulation (Ghost/Miasma) and the perception (Gear) to communicate via a shared "data language" without shared "type vocabulary."
3.  **Simulation must be "Honest".** Gear must be refactored to read from Fields, not from `HauntState`. If a ghost is "Freezing," it should simply be an Emitter that modifies the `TemperatureField`. The gear should only see the resulting temperature.

## 4. Next Steps for Investigation
*   Analyze the performance impact of a `HashMap` lookup for fields in `BoardData` vs. static fields.
*   Explore if `GearStuff` can be made "extensible" using Bevy's `Entity` metadata or a "Blackboard" resource.
*   Determine which fields are truly "Core" (Collision, Light) and which are "Game-Specific" (Miasma, Temperature, Radiation).

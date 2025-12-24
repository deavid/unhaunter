# Field Untangling: The Beauty of Enums vs. Data-Driven Flexibility

## 1. The Beauty of Enums: Why We Love Them

The current use of `GearKind` and `GearSpriteID` enums provides several high-value benefits that are hard to replicate with a purely data-driven approach:

### 1.1 Compile-Time Safety & Exhaustiveness
*   **The `match` Guarantee:** When adding a new gear type, the compiler forces you to update every system that uses it (e.g., `from_gearkind`, UI rendering, evidence mapping). You cannot "forget" a piece of gear.
*   **No Typos:** `GearKind::Thermometer` is checked at compile-time. `"thermometer"` (as a string) is not.

### 1.2 Legibility & Intent
*   **Self-Documenting Code:** `match gear.kind { GearKind::Flashlight => ... }` is immediately understandable. It describes *what* the object is, not just *how* it behaves.
*   **IDE Support:** Autocomplete and "Go to Definition" work perfectly.

### 1.3 Simplicity of Serialization
*   **Clean JSON/TMX:** Enums serialize to readable strings like `"Thermometer"`. Mapping these to Tiled properties or save files is trivial and robust.

---

## 2. The "Shared-Mod" Concept: A Middle Ground

Instead of a "Universal Dictionary" in the engine foundation, we can think of gear as being provided in **Vocabulary Blocks** (Plugins).

### 2.1 The "Gear Set" Plugin
Imagine a crate `unhaunter-gear-classic`. It provides:
*   `pub enum ClassicGearKind { Thermometer, EMFMeter, ... }`
*   `pub enum ClassicGearSprite { ... }`
*   The actual `GearUsable` implementations.

### 2.2 How the Engine Stays Clean
The engine (`ungear`) doesn't need to know about `ClassicGearKind`. It only needs to know about **Entities**.
*   **The Engine's Vocabulary:** `struct Gear`, `struct Inventory`, `struct Hand`.
*   **The Game's Vocabulary:** `ClassicGearKind`.

### 2.3 The Legibility Trade-off
If we move to a purely data-driven approach (no enums), we lose the `match` statement. We replace it with:
```rust
// Data-Driven (Legibility Loss)
if entity.has::<Thermometer>() { ... }
else if entity.has::<EMFMeter>() { ... }
```
This is "legible" but lacks the **exhaustiveness** of an enum. You can't easily ask the compiler "did I handle all types of gear in this UI panel?"

---

## 3. The "Block-Based" Simplicity

If we treat a "Set of Gear" as a unit, we can keep the enums **local to that set**.

### 3.1 Multiple Atlases & Sets
If "Mod A" provides 10 gear items in one image, it defines an enum for those 10 indices.
If "Mod B" provides 5 gear items in another image, it defines its own enum.

### 3.2 The "Universal" Problem
The only place that *truly* needs a universal list is the **Truck UI** (to show all available gear).
*   **The Bevy Way:** The Truck UI doesn't need an enum. It just needs a `Query<&GearInfo>`.
*   **The Legibility Win:** We can still use enums for **internal logic** of a gear set, while using components for **cross-plugin communication**.

---

## 4. Preliminary Conclusion: "Local Enums, Global Components"

We don't have to choose between "Type Hell" and "Vocabulary Gods."

1.  **Keep Enums for Sets:** Let each gear-set plugin define its own `GearKind` for its own internal logic and serialization.
2.  **Use Components for the Engine:** The engine foundation should use generic components (e.g., `GearVisual { atlas, index }`) so it can render *any* gear from *any* mod without knowing what it is.
3.  **Decouple the Foundation:** Move `GearKind` out of `uncore-foundation`. The foundation should only provide the "Grammar" (how to hold things), not the "Words" (what the things are).

**Next Investigation:** How to allow `undifficulty` (which needs to know about starting gear) to work with multiple independent gear-set plugins without creating a dependency bottleneck.

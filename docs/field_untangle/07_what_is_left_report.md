# Field Untangling: Status Report & "What is Left"

**Date:** December 24, 2025
**Status:** Phase 3 (Cleanup) - **COMPLETED**

## 1. Current State Assessment

The codebase has successfully undergone the major architectural shift from Object-Oriented (`Gear` struct + `GearUsable` trait) to Data-Oriented (ECS Entities + Components). The legacy scaffolding has been removed.

### 1.1 Achievements
*   **`PlayerGear` & `TruckGear` Refactored:** These now store `Entity` (or `Vec<Entity>`) instead of `Gear` structs.
*   **Registry Implemented:** `GearSpawnerRegistry` is active and used to spawn gear entities.
*   **Components Defined:** `uncore-components` is populated with the new vocabulary (`Flashlight`, `EvidenceSensor`, `Toggleable`, etc.).
*   **Systems Ported:**
    *   **Interaction:** `unplayer` now handles grabbing/dropping/cycling using entities.
    *   **UI:** `untruck` now queries components to render the loadout and journal.
    *   **Triggers:** `unwalkie` now detects gear usage by querying components.

### 1.2 The "Clean" Reality
The transition is complete.
*   **Legacy Types Deleted:** `struct Gear` and `trait GearUsable` have been deleted.
*   **Legacy Implementations Removed:** All `impl GearUsable` blocks have been stripped from `ungearitems`.
*   **Files Cleaned:** `ungearitems` components now only contain ECS Component definitions and their associated Systems.
*   **Obsolete Files Removed:** `crates/ungear/src/gear_usable.rs` and `crates/ungearitems/src/from_gearkind.rs` are gone.

---

## 2. The "Lost" Logic (Feature Parity Check)

During the transition, some complex simulation logic embedded in `GearUsable::update` was simplified.

*   **Flashlight Thermodynamics:** The old `Flashlight` struct simulated `inner_temp` and `heatsink_temp`. The new `Flashlight` component only stores `power` and `color`.
    *   *Impact:* Flashlights no longer heat up or throttle.
    *   *Action:* Decide if this feature is needed. If so, implement a `HeatSystem` and `Heatsink` component.
*   **Battery Drain:** The old system handled battery drain in `update`.
    *   *Status:* `Battery` component exists, but we need to ensure a `battery_drain_system` is registered and active.

---

## 3. What is Left (The Verification Roadmap)

The refactor and cleanup are done. The focus now shifts to verification and polish.

### 3.1 System Verification
1.  **Battery System:** Verify `battery_system` exists and drains power.
2.  **Sensor System:** Verify `thermometer_system`, `emf_system`, etc., are reading from the environment (or the ghost, for now) and updating their state.

### 3.2 Feature Restoration (Optional)
1.  **Flashlight Heat:** Re-implement if deemed necessary.

## 4. Conclusion

The "Field Untangling" is complete. The engine is now fully decoupled from the legacy `Gear` object model. The codebase compiles and runs using the new ECS architecture.

**Next Steps:** Verify the gameplay systems (battery, sensors) are functioning as expected in the new architecture.

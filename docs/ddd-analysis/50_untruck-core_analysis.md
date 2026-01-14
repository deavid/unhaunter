# DDD Analysis: `untruck-core`

## 1. Bounded Context

**Truck & Base Operations Domain**. This crate owns the data structures and events that represent the player's safe zone
(the truck) and the metadata related to investigation progress (the journal).

## 2. Responsibility

- **Inventory Management**: Defines `TruckGear` to track which items are currently stored in the truck.
- **Journal State**: Provides events and types for managing the investigation journal (e.g.,
  `ForceDiscardEvidenceEvent`).
- **Truck UI Components**: Defines marker components for truck-specific UI elements.

## 3. Dependencies and Appropriateness

- **Dependencies**: `bevy`, `unfoundation-core`, `unevents-core`.
- **Appropriateness**: High. It depends on the evidence types defined in the foundation and communication infrastructure
  in events.

## 4. Encapsulation (Data/Logic Split)

- **Data**: Structs and events.
- **Logic**: Zero logic.

## 5. Semantic & Infrastructure Leakage

- **Semantic Leakage**: Low. It uses `Evidence` from the domain to implement the journal logic.
- **Infrastructure Leakage**: Low. Only depends on Bevy for ECS markers.

---

## Technical Debt & Strategic Notes

- **Unified Inventory**: Currently, there is a split between player inventory and truck inventory. This crate owns the
  latter.
- **Journal-Domain Split**: The journal is currently tightly coupled with the "Truck", but it is also a portable UI
  element. Future refactors might move the journal to a more general "Investigation" domain.

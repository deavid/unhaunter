# DDD Analysis: `unui-core`

## 1. Bounded Context

**UI Taxonomy Domain**. This crate defines the marker components and metadata needed to identify, organize, and query UI
elements within the global Bevy ECS.

## 2. Responsibility

- **UI Identification**: Defines marker structs like `GameUI`, `EvidenceUI`, `HeldObjectUI`, and `SummaryUI`.
- **UI State Metadata**: Stores simple data associated with UI elements, such as the `DamageBackground` intensity.
- **Component Filtering**: Provides the types used by UI-updating systems to find their target elements.

## 3. Dependencies and Appropriateness

- **Dependencies**: `bevy`.
- **Appropriateness**: High. By centralizing these markers, different domain plugins (e.g., `unwalkie-plugin` and
  `untruck-plugin`) can share a common set of UI handles.

## 4. Encapsulation (Data/Logic Split)

- **Data**: Pure marker structs and small data components.
- **Logic**: Zero logic.

## 5. Semantic & Infrastructure Leakage

- **Semantic Leakage**: Moderate. It contains semantic labels for various game features (Evidence, Walkie-Talkie,
  Damage).
- **Infrastructure Leakage**: Low. Only depends on Bevy's ECS attributes.

---

## Technical Debt & Strategic Notes

- **Decentralization Potential**: As the UI grows, this crate might become a bottleneck. Currently, it acts as a "UI
  Dictionary".
- **Dynamic UI**: If the game moves to a more dynamic or data-driven UI system (like a scripting language or
  templating), this crate would likely serve as the registry of "Known Anchors".

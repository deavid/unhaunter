# DDD Analysis: `unroot-plugin`

## 1. Bounded Context

**Application Lifecycle & Bootstrap**. This crate acts as the "composition root" of the application, responsible for
initializing global state, loading early assets, and orchestrating the final assembly of the Bevy app.

## 2. Responsibility

- **State Initialization**: Initializes core application states (`AppState`, `GameState`).
- **Asset Loading**: Manages the centralized (albeit hardcoded) loading of images, fonts, and manual assets into the
  `GameAssets` resource.
- **Global Resource Setup**: Initializes cross-cutting resources like `Maps`, `CurrentEvidenceReadings`, `PlayerInput`,
  and `PerlinNoise`.
- **Event Registration**: Registers global event types like `OnScreenHintEvent`.

## 3. Dependencies and Appropriateness

- **Dependencies**: `unassets-core`, `untypes-core`, `unevents-core`, `unmenu-core`, `unghost-core`, `unnoise-core`,
  `unplayer-core`, `bevy`, `bevy_framepace`.
- **Appropriateness**: High as a root orchestration crate. It necessarily depends on many domains to set up their
  initial state.

## 4. Encapsulation (Data/Logic Split)

- **Data**: Does not define new domain data; it mostly instantiates containers defined in `-core` crates.
- **Logic**: Contains the imperative logic for asset loading and app setup.

## 5. Semantic & Infrastructure Leakage

- **Semantic Leakage**: Very High. The `load_assets` function contains paths and logic for manual pages, character
  sprites, and gear icons. It effectively "knows" about every visual element in the game.
- **Infrastructure Leakage**: High. It interacts directly with the `AssetServer` and sets up hardware-specific
  configurations (via `bevy_framepace`).

---

## Technical Debt & Strategic Notes

- **Asset Decentralization**: As noted in `16_summary_1.md`, the current hardcoded list in `unroot-plugin` is a major
  bottleneck and source of semantic leakage. Assets should ideally be registered by the plugins that use them (e.g.,
  `ungear-plugin` should load gear sprites).
- **Resource Ownership**: Several resources (like `CurrentEvidenceReadings` or `PlayerInput`) are initialized here but
  are logically owned by other domains. Moving these to their respective plugin's `build` function would improve
  modularity and reduce the coupling in `unroot-plugin`.
- **Composition Root**: In a "Ghost Engine" future, this crate would likely be the only thing that changes between
  different "games" or "mods", as it defines which assets and initial states are loaded.

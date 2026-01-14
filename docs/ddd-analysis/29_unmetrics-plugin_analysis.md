# Crate Analysis: `unmetrics-plugin`

## Bounded Context

**Diagnostics & Telemetry Integration**

This crate integrates the project's custom metrics infrastructure into the Bevy app lifecycle.

## Responsibility

- Implement the `UnmetricsPlugin`.
- Register the `metric_drain_system` (from `unmetrics-core`) into the `Update` schedule.
- Ensure that metrics are drained and populated into the `DiagnosticsStore` every frame.

## Dependencies Audit

- `unmetrics-core`: For the `metric_drain_system` and transport channel.
- `bevy`: For the `Plugin` trait and system scheduling.

## Encapsulation Assessment

- **Privacy**: High. Exposes only the `Plugin`.
- **Role**: A thin "Glue" layer that connects the metrics infrastructure to the engine.

## Semantic & Infrastructure Leakage

- **Cleanliness**: Very high. It doesn't introduce any new dependencies or leak any game logic.

## Future Recommendations

- **Consolidation**: Since the plugin is so thin and `unmetrics-core` already has a Bevy dependency, these two are good
  candidates for staying as-is or being merged if the "no-logic-in-core" rule is relaxed for base infrastructure.

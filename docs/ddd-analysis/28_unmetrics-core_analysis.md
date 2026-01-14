# Crate Analysis: `unmetrics-core`

## Bounded Context

**Diagnostics & Telemetry**

This crate provides the infrastructure for measuring performance and recording gameplay metrics across the entire
workspace.

## Responsibility

- Provide a thread-safe global channel for reporting metrics from any context (not just ECS).
- Implement the `metric_drain_system` to bridge raw reports into Bevy's `DiagnosticsStore`.
- Offer a convenience API (`SendMetric` trait) for recording task durations.

## Dependencies Audit

- `bevy`: For the diagnostic system integration.
- `bevy_platform`: For cross-platform high-resolution timing.

## Encapsulation Assessment

- **Privacy**: The internal message channel is well-encapsulated behind a public `METRICS_TRANSPORT` static and the
  `SendMetric` trait.
- **Architectural Violation**: Contains a Bevy system (`metric_drain_system`), which technically makes it more than just
  a "data/types" crate, though appropriate for shared infrastructure.

## Semantic & Infrastructure Leakage

- **Global State**: Uses a `static` lazy transport. While this is an "architectural smell" for some, it is a pragmatic
  choice to allow non-ECS code to report performance metrics.
- **Engine Coupling**: Deeply tied to Bevy's `Diagnostics` infrastructure.

## Future Recommendations

- **Abstract the Transport**: If the engine ever needs to send metrics to an external service (like Prometheus or
  OpenTelemetry), the `metric_drain_system` should be moved to a plugin, or the transport should be made pluggable.
- **Isolate Logic**: Move the `metric_drain_system` to `unmetrics-plugin` to keep `unmetrics-core` purely as a
  lightweight reporting library.

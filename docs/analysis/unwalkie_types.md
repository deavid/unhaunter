# Crate Analysis: `unwalkie_types`

## Core Responsibilities

The `unwalkie_types` crate defines shared data types for the Walkie Talkie system. It handles:

- **Categorization**: Defining `WalkieTag` to categorize voice lines (e.g., hints, warnings, reminders).
- **Metadata**: Defining structures for voice line data (implied).

## Architectural Analysis

- **Type**: Data Crate
- **Role**: Shared type definitions for the walkie-talkie system and its tools.
- **Dependencies**:
  - `serde`: For serialization.

## 2D/3D Coupling

- **None**: This crate is purely data-driven and contains no rendering or spatial logic.

## Migration Risks

- **None**: This crate can be used as-is in the 3D version.

## Key Files

- `src/lib.rs`: Defines `WalkieTag`.

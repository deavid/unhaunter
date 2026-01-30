# Refactor Plan: Map Registry & Untruck Extraction

**Date:** 2026-01-30
**Target:** `unmapload-plugin`, `untruck-plugin`
**Goal:** Remove dead code (`factory.rs`) and decouple Truck/Van logic from the core Map Loader.

## Context
The DDD Audit identified `unmapload-plugin` as having a "God Factory". Upon investigation, the file `factory.rs` was found to be dead code (leftover from a previous refactor). However, `hydration_generic.rs` still contains logic for `is_van_entry`, which belongs in the Truck domain.

This plan outlines the steps to:
1.  Clean up the dead code.
2.  Move the Van Entry hydration logic to `untruck-plugin`.

## Step-by-Step Instructions for Copilot

### Phase 1: Cleanup
1.  **DELETE** the file `crates/unmapload-plugin/src/factory.rs`.
    *   *Verification:* Ensure no other file references it (checked: `lib.rs`, `module.rs` do not).

### Phase 2: Untruck Extraction
1.  **MODIFY** `crates/untruck-plugin/Cargo.toml`:
    *   Add `unbehavior = { path = "../unbehavior" }` to `[dependencies]`.
2.  **CREATE** `crates/untruck-plugin/src/hydration.rs` with the following content:
    ```rust
    use bevy::prelude::*;
    use unbehavior::behavior::{Behavior, Interactive};
    use unbehavior::components;
    use untypes_core::hydration::HydrationStage;

    fn hydration_van_entry_system(
        mut q: Query<(Entity, &Behavior), With<HydrationStage<3>>>,
        mut commands: Commands,
    ) {
        use bevy::picking::Pickable;
        for (entity, behavior) in q.iter_mut() {
            if behavior.p.is_van_entry {
                commands.entity(entity)
                    .insert(Pickable::default())
                    .insert(Interactive::new(
                        "sounds/door-open.ogg",
                        "sounds/door-close.ogg",
                    ))
                    .insert(components::FloorItemCollidable);
            }
        }
    }

    pub(crate) fn app_setup(app: &mut App) {
        app.add_systems(Update, hydration_van_entry_system);
    }
    ```
3.  **MODIFY** `crates/untruck-plugin/src/lib.rs`:
    *   Add `pub(crate) mod hydration;` to the module list.
4.  **MODIFY** `crates/untruck-plugin/src/plugin.rs`:
    *   In `fn build(...)`, add `super::hydration::app_setup(app);`.

### Phase 3: Unmapload Cleanup
1.  **MODIFY** `crates/unmapload-plugin/src/hydration_generic.rs`:
    *   Remove the `else if behavior.p.is_van_entry { ... }` block entirely.
    *   Ensure the `if/else` chain remains valid (i.e., if it was in the middle, fix the `else if` linkage).

### Phase 4: Validation
1.  Run `cargo check -p untruck-plugin` to ensure dependencies are correct.
2.  Run `cargo check -p unmapload-plugin` to ensure no regressions.

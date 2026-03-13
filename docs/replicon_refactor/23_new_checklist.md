# 23 — Skeleton/Skin Final Completion Checklist

This document tracks the remaining technical debt and architectural gaps identified after auditing the implementation of
[21_gear_skeleton_skin_design.md](21_gear_skeleton_skin_design.md).

## 1. Fix "Mixed" Replicated Components (Skeleton/Skin Split)

The following components are currently replicated but still contain local accumulators (Skin) that the server should not
track and that would be stomped by replication.

### Sage Bundle

- [ ] Create `SageBundleSkin` in `crates/ungearitems-core/src/components/sage.rs`.
- [ ] Move `burn_timer: Timer` and `smoke_produced: usize` from `SageBundleData` to `SageBundleSkin`.
- [ ] Update `update_sage_skeleton` and `update_sage_skin` in `unplayer-plugin` to reflect the split.
- [ ] Ensure `SageBundleSkin` is **not** replicated.

### Quartz Stone

- [ ] Create `QuartzStoneSkin` in `crates/ungearitems-core/src/components/quartz.rs`.
- [ ] Move `cracked_time: f32` and `energy_absorbed: f32` from `QuartzStoneData` to `QuartzStoneSkin`.
- [ ] Update `update_quartz_skeleton` and `update_quartz_skin` to reflect the split.
- [ ] Ensure `QuartzStoneSkin` is **not** replicated.

---

## 2. Fix Export Data Loss & `ExportGearStateMessage`

The current `status_level: u8` hack in `ExportGearStateMessage` is insufficient for complex gear and results in data
loss during client→server sync.

- [ ] **Repellent Flask Fix:**
  - [ ] `liquid_content` is currently **not synced**. It must be added to the export.
  - [ ] `qty` is currently capped at 255 (u8 cast). It must be synced as `i32` or `u16` to support `MAX_QTY = 400`.
- [ ] **Structural Refactor:**
  - [ ] Replace `status_level: u8` with an enum `GearSkeletonState` that can safely hold `FlashlightStatus`,
        `RepellentData`, etc., without lossy casting.
  - [ ] Update `send_export_gear_state` and `handle_export_gear_state` in
        [crates/unreplicon-plugin/src/systems/players.rs](../crates/unreplicon-plugin/src/systems/players.rs) to use the
        new structured data.

---

## 3. Server-Side Memory Cleanup (Skin removal from Authority)

The server currently attaches "Skin" structs (which contain large vectors or complex buffers) to its own entities, even
though it never runs the systems that use them.

- [ ] **Audit `gear_registry.spawn`:** In `crates/ungearitems-plugin/src/registration.rs`, the server-side spawning path
      should **not** attach the following components:
  - `Thermometer`
  - `EMFMeter`
  - `GeigerCounter`
  - `Recorder`
  - `SpiritBox`
  - `FlashlightSkin` / `UVTorchSkin` / `RedTorchSkin`
- [ ] **Verify Hydration:** Ensure that `hydrate_gear_system` on the client successfully attaches these components so
      that players still see their sensor readings.
- [ ] **Remove from Dedicated Server:** Confirm that a dedicated server's memory footprint no longer includes these
      UI/Visual state buffers.

---

## 4. Feature Parity & Polishing

- [ ] **Repellent Audio:** Add `SoundEmitter` calls to `update_repellentflask_skeleton` for spray activation (consistent
      with Salt/Sage).
- [ ] **Repellent UI:** Update the `StatusText` in `update_repellentflask_skin` to show a countdown/gauge for "Emptying
      flask..." rather than a static string.

---

## Summary of Audit Findings

| Gear Item           | Skeleton Correct? |     Skin Split?     |  Network Sync?   | Server-Clean? |
| :------------------ | :---------------: | :-----------------: | :--------------: | :-----------: |
| Flashlight          |        ✅         |         ✅          |        ✅        |      ✅       |
| UV/Red Torch        |        ✅         |         ✅          |        ✅        |      ✅       |
| Repellent           |        ✅         |         ✅          |        ✅        |      ✅       |
| Sage                |        ✅         |         ✅          |        ✅        |      ✅       |
| Quartz              |        ✅         |         ✅          |        ✅        |      ✅       |
| Sensors (EMF, etc.) |        ✅         | ⚠️ (No Skin Struct) | ✅ (Toggle only) |      ✅       |

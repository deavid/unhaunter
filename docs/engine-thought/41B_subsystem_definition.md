# Subsystem: A Rigorous Abstract Definition

## The Problem

In previous design phases, we focused on "Components" (in the ECS sense). This was too granular. It led to listing individual structs like `ThermalMass` or `Switch` rather than seeing the bigger picture.

We need a term for the **"Big Chunks"** of the engine. The architectural pillars. The things that might become their own crates.

We call these **Subsystems**.

---

## The Definition

> **A Subsystem is a cohesive collection of data, logic, and rules that fulfills a specific architectural capability.**

It is not just a struct. It is a **domain**. It typically includes:
- **Data Types** (Components, Resources, Enums)
- **Logic** (Systems, Functions)
- **API** (Public traits, Events)

---

## The Five Tests

For something to qualify as a Subsystem, it must pass **ALL** of these:

| Test                 | Question                                                                  | Failure Example                                                                        |
| -------------------- | ------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| **1. Cohesive**      | Does it solve ONE clear problem domain?                                   | "Gameplay" fails — it's too broad. "Lighting" passes.                                  |
| **2. Significant**   | Is it complex enough to warrant its own crate or major module?            | "Position" fails — it's just a data type. "Spatial" passes.                            |
| **3. Encapsulated**  | Can it hide its internal complexity behind a clean API?                   | "Ghost AI" fails if it's scattered across 10 files. It must be bundled.                |
| **4. Decoupled**     | Could you theoretically rip it out and replace it with a different version?| "The Player" fails if it's hardcoded into every other system.                          |
| **5. Named**         | Can you name it without using "and"?                                      | "LightAndSound" fails. They are two subsystems.                                        |

---

## Applying the Tests

| Candidate              | Cohesive? | Significant? | Encapsulated? | Decoupled? | Verdict                        |
| ---------------------- | --------- | ------------ | ------------- | ---------- | ------------------------------ |
| **Thermal**            | ✅ Heat   | ✅ Complex   | ✅ Temp API   | ✅ Yes     | ✅ **Subsystem**               |
| **Inventory**          | ✅ Items  | ✅ Complex   | ✅ Slots API  | ✅ Yes     | ✅ **Subsystem**               |
| **Position**           | ✅ Where  | ❌ Too small | —             | —          | ❌ Data Type (part of Spatial) |
| **Ghost**              | ❌ Entity | —            | —             | —          | ❌ Entity (uses Subsystems)    |
| **Walking**            | ❌ Action | —            | —             | —          | ❌ Behavior (part of Actor)    |
| **Lighting**           | ✅ Light  | ✅ Complex   | ✅ Lux API    | ✅ Yes     | ✅ **Subsystem**               |

---

## Subsystem vs. Component vs. Entity

```
┌─────────────────────────────────────────────────────────────────┐
│                       SUBSYSTEM                                 │
│                    (e.g., Thermal)                              │
│                                                                 │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐         │
│  │  Component   │   │   System     │   │   Resource   │         │
│  │ (ThermalMass)│   │ (Propagation)│   │ (GlobalTemp) │         │
│  └──────────────┘   └──────────────┘   └──────────────┘         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
         ▲
         │ (Attached to)
         ▼
┌─────────────────────────────────────────────────────────────────┐
│                        ENTITY                                   │
│                      (The Ghost)                                │
│                                                                 │
│  [Spatial Component]  [Thermal Component]  [Render Component]   │
└─────────────────────────────────────────────────────────────────┘
```

---

## The Litmus Test (Quick Version)

When someone proposes a Subsystem, ask:

> **"Could this be its own crate?"**

If the answer is "No, it's just a struct," it's a Component.
If the answer is "No, it's just a function," it's a System.
If the answer is "Yes, `unthermal` or `uninventory`," it's a Subsystem.

### Examples

- **`unspatial`**: Handles grids, transforms, directions. ✅
- **`unlight`**: Handles propagation, visibility, lux. ✅
- **`unhealth`**: Handles damage, death, healing. ✅
- **`unghost`**: Handles AI, state machines, hunting logic. ✅

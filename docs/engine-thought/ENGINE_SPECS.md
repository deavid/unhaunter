# Unhaunter Engine Specifications: The Technical Contract

> **Definition:** The Unhaunter Engine is a discrete spatial simulation platform. It manages a 3D voxel grid ("The
> Board"), propagates values through that grid ("The Fields"), and manages entities that react to those values ("The
> Agents").

## 1. Core Philosophy

### 1.1 The Grid is Sovereign

The engine treats the discrete `BoardPosition` (x, y, z) as the single source of truth.

- **Visuals are Interpolation:** `Transform` (floating point position) is merely a visual interpolation of the grid
  state. Logic never queries `Transform`.
- **No Sub-Grid Logic:** An entity is strictly inside a cell or it is not. There is no partial occupancy.

### 1.2 Data Over Scripts

The engine simulates systems, it does not script events.

- **Emergence:** "Horror" occurs when an Agent reacts to a Field value change. It is not triggered by a scripted
  timeline.
- **Universal Simulation:** Rules apply globally. If a Field propagates heat, it propagates it everywhere, not just
  where the player is looking.

### 1.3 The Three Primitives

The engine provides exactly three primitive types:

1.  **Space (The Board):** The container and connectivity graph.
2.  **Force (The Fields):** The invisible data layers permeating the container.
3.  **Agents (The Actors):** The state-bearing entities navigating the container.

---

## 2. The Spatial System (The Board)

### 2.1 Guarantees

- **Discrete Voxel Topology:** The world is strictly a collection of `Cell` structs indexed by `IVec3`.
- **Live Connectivity Graph:** The engine maintains a cached graph of accessible neighbors. This graph updates
  immediately when `Interactive` objects change state (e.g., a door closing updates the graph to sever the link between
  cells).
- **Standard Interactivity:** A trait-based system (`Interactive`) allows objects (Doors, Containers, Switches) to
  define state toggles. The engine guarantees that toggling state triggers:
  1.  Visual update (Sprite/Mesh change).
  2.  Connectivity Graph update.
  3.  Field Occlusion update (walls blocking light/sound).

### 2.2 Limitations

- **No Arbitrary Physics:** There is no generalized physics engine. Collision is boolean (blocked/unblocked). There are
  no slopes, physics-based projectiles, or ragdoll interactions with geometry.
- **Verticality is Layered:** The Z-axis functions as discrete "Floors." True 3D spatial reasoning (e.g., looking up
  through a hole) is limited to specific Field propagation rules, not general geometry.

---

## 3. The Simulation System (The Fields)

### 3.1 Guarantees

- **Field Registry:** A dynamic system to register `Array3<T>` data structures mapped to the spatial grid dimensions.
- **Built-in Propagation Solvers:**
  - **Fluid Solver:** Simulates pressure, velocity, and diffusion (e.g., for Temperature, Miasma, Gas).
  - **Signal Solver:** Simulates inverse-square decay and occlusion (e.g., for Light, Sound, EMF, Radiation).
- **Emitter/Receiver Pattern:**
  - `Emitter`: A component that injects values into a specific Field at its Grid coordinates per tick.
  - `Sensor`: A component that queries values from a specific Field at its Grid coordinates.

### 3.2 Limitations

- **Discrete Time Steps:** Propagation occurs in ticks. Fast-moving entities may outrun propagation updates.
- **Grid Resolution:** Field granularity cannot exceed grid granularity. You cannot have a "cold corner" of a single
  tile; the whole tile is cold.

---

## 4. The Agent System (The Actors)

### 4.1 Definition

An **Agent** is any entity possessing:

1.  `GridTransform` (Position/Orientation)
2.  `Navigation` (Pathfinding state)
3.  `AgentBehavior` (State machine)

### 4.2 Guarantees

- **Field-Aware Pathfinding:** The A* implementation accepts Field values as cost modifiers. Agents can be configured to
  pathfind *around* high Light values or *towards\* high Sound values.
- **Modular Behavior:** Agents are composed of independent logic modules (e.g., `Roam`, `Flee`, `Investigate`).
- **Threat Module (Optional):** A specialized module providing target acquisition (Line-of-Sight, Distance) and
  agitation metrics. _Note: Not all Agents are Threats._

### 4.3 Limitations

- **Interaction Proxy:** Agents cannot physically manipulate the world beyond the `Interactive` trait. They cannot drag
  objects, stack boxes, or physically restrain the player.
- **No Physics-Based Animation:** Agents slide between tiles. Walk cycles and reactions are purely visual/cosmetic.

---

## 5. The Perception System (The Gear)

### 5.1 Guarantees

- **Sensor API:** A standardized trait for tools to sample Field data.
- **Feedback Drivers:** Infrastructure to map Sensor outputs to hardware/visual feedbacks:
  - Audio (Volume/Pitch)
  - Visual (Graph/Needle/Color)
  - Haptic (Rumble intensity)
- **Remote Linking:** A data pipeline to replicate Sensor readings to a secondary UI context (e.g., "The Truck"
  monitoring screen) without duplicating the simulation logic.

### 5.2 Limitations

- **Illiteracy:** Tools can _only_ read Fields or Tags. They cannot deduce intent. A tool cannot detect "The Ghost"; it
  can only detect "EMF Level 5."
- **No Complex Manipulation:** Tools are sensors or toggle-switches. There is no complex mechanical interaction (e.g.,
  lockpicking mini-games are UI overlays, not simulation interactions).

---

## Summary of Constraints

The Unhaunter Engine trades **freedom** for **fidelity**.

| Freedom Lost           | Fidelity Gained                                                                            |
| :--------------------- | :----------------------------------------------------------------------------------------- |
| **No Free Movement**   | **Precise Spatial Reasoning:** We know exactly where everything is at all times.           |
| **No Physics Engine**  | **Deep Field Simulation:** We spend our CPU budget on invisible fluids, not rigid bodies.  |
| **No Scripted Events** | **Emergent Horror:** Scares happen because the systems interacted, making them replayable. |

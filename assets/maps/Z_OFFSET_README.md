# Placing Objects on Tables and Surfaces

This document explains how to place objects on tables, shelves, or other surfaces in Unhaunter maps.

## The Problem

By default, all items on a floor level are placed with a z-coordinate of 0 relative to the floor. This means items appear directly on the floor, even when they should be on top of tables or other furniture.

## The Solution: Layer Z-Offset

We've added a new property that can be applied to any layer in Tiled:

- **Property Name**: `z_offset`
- **Type**: Float
- **Purpose**: Specifies a vertical offset for all tiles in the layer

## How to Use

1. In Tiled, create separate layers for objects that should be at different heights:
   - A base layer for floor items (default z_offset = 0)
   - A "TableTop" layer for items on tables (e.g., z_offset = 0.2)
   - A "ShelfItems" layer for items on shelves (e.g., z_offset = 0.4)

2. Add the `z_offset` property to these layers:
   - Right-click the layer in Tiled
   - Select "Properties"
   - Add a new property with name "z_offset" and type "float"
   - Set values based on the desired height (e.g., 0.2 for table tops)

3. Place tiles/objects in the appropriate layers based on where they should appear

## Implementation Details

- The z_offset is added to the base floor level z-coordinate
- This only affects visual placement; collision detection still works normally
- Items with z-offsets still belong to the room they're placed in

## Tips

- For consistent heights, use the same z_offset values across all maps
- Recommended values:
  - Tables/Desks: 0.2-0.3
  - Shelves: 0.4-0.5
  - Higher shelves: 0.6-0.8
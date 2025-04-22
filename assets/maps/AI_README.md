# AI README: Understanding Unhaunter Map Data (TMX/TSX)

**Purpose:** This document explains how Unhaunter utilizes Tiled map editor files (`.tmx` and `.tsx`) to define game levels, focusing on the information encoded within the Tileset (`.tsx`) files that AI assistants need to understand for code analysis and generation.

**Core Concept:** The `.tsx` files act as a crucial bridge between visual tiles in the spritesheets (`assets/img/*.png`) and the game's logic implemented in Bevy. Information stored in the TSX determines the behavior and properties of objects placed in `.tmx` map files.

## 1. Tiled `type` -> Bevy `Class` Mapping

The most important piece of information for any tile defined in a TSX file is its **`type` attribute**.

```xml
<tile id="0" type="Floor"> ... </tile>
<tile id="41" type="Door"> ... </tile>
<tile id="48" type="Decor"> ... </tile>
```

*   **Mapping:** The value of the `type` attribute directly corresponds to the `uncore::behavior::Class` enum variant in the codebase.
*   **Significance:** This `Class` determines the fundamental *category* of the tile and dictates which core Bevy components (`Ground`, `Collision`, `Opaque`, `Interactive`, `Light`, `Door`, `Stairs`, `NpcHelpDialog`, etc., found in `uncore::behavior::component`) are initially added to the entity when it's spawned during level loading (`ungame/src/level.rs`). It's the primary driver of how the game treats the tile.

## 2. Key Tile Properties (`<properties>`)

Within a `<tile>` definition, specific `<property>` tags encode further details that populate the `uncore::behavior::Behavior` component, primarily within its `cfg: SpriteConfig` and `p: Properties` fields.

*   **`sprite:variant` (String)**
    *   **Maps to:** `Behavior.cfg.variant`
    *   **Purpose:** Differentiates tiles *within the same Class*. For `Floor`, it defines the texture (e.g., "StonePath", "Grass"). For `Wall` or `Door`, it might define material ("FlatBlue", "OakWood", "Metal"). For `Decor` or `Item`, it often identifies the specific object ("GreenChair", "RedBook", "Vase1"). This is used for visual distinction and potentially specific logic variations. If absent, a default based on tileset/ID might be used.
*   **`sprite:orientation` (String -> Enum)**
    *   **Maps to:** `uncore::behavior::Orientation` (`XAxis`, `YAxis`, `Both`, `None`)
    *   **Purpose:** Defines the facing direction for sprites like walls, doors, and some switches. Affects visual appearance (which sprite variant might be chosen implicitly by `SpriteDB` lookup) and gameplay logic (e.g., collision shape, light occlusion).
*   **`sprite:state` (String -> Enum)**
    *   **Maps to:** `uncore::behavior::TileState` (`On`/`Off`, `Open`/`Closed`, `Full`/`Partial`/`Minimum`, `None`)
    *   **Purpose:** Represents the *initial* state of an interactive or multi-state object (like lamps, switches, doors, potentially complex walls).
    *   **Convention:** See Section 5.
*   **`object:*` Properties (Boolean/Float/String)**
    *   **Maps to:** Fields within `Behavior.p.object` (`pickable`, `movable`, `hidingspot`, `weight`, `name`).
    *   **Purpose:** Define physical interactions for `Decor`, `Item`, and `Furniture`. Determines if the player can pick up, push/pull, or hide behind the object. `weight` might influence movement speed when carried. `name` provides a display name.
*   **Other Properties:** Tiles might have other specific properties used by custom logic (though none were obvious in the examples). The `RoomDef` class uses `sprite:variant` to store the room's name.

## 3. Tileset Properties

*   **`Anchor::bottom_px` (Int, Defined at `<tileset>` level)**
    *   **Purpose:** Used during asset loading (`untmxmap/src/bevy.rs`) to calculate a `y_anchor` value (`MapTileSet.y_anchor`). This anchor is then used when creating the sprite's mesh (`QuadCC` in `SpriteDB` population) to ensure correct vertical positioning/sorting in the isometric view. It defines how many pixels from the *bottom* of the sprite image should be considered the logical "floor" position (0,0) of the tile.

## 4. Taxonomy / Categories of Tiles (Based on `Class`)

For easier understanding, the tile `Class` types can be grouped:

1.  **Base Structure:** Define the physical layout.
    *   `Floor`: Walkable ground surfaces.
    *   `Wall`: Standard obstacles blocking movement and sight.
    *   `LowWall`: Obstacles blocking movement but not necessarily sight (e.g., fences, counters).
    *   `Window`: Allow sight, may or may not block movement depending on specific implementation, affects light transmission.
    *   `Door`: Interactive barriers, state changes collision/opacity.
    *   `Doorway`: Non-colliding visual connectors between walls.
    *   `StairsUp`/`StairsDown`: Floor transition points.
2.  **Interactive Objects:** Player can directly interact with these to change their state.
    *   `Switch`, `RoomSwitch`: Toggle state (On/Off), often affect lights.
    *   `Breaker`: Main power control switch.
    *   `WallLamp`, `FloorLamp`, `TableLamp`: Light sources, state controls emission. Often interactive.
3.  **Furniture & Appliances:** Larger world objects providing context, collision, potential hiding spots, or item placement surfaces.
    *   `Furniture`: (e.g., Couch, Bed, Table, Wardrobe, Bookshelf, Sink). Usually static, may be hiding spots.
    *   `Appliance`: (e.g., TV, Fridge, Stove, WashingMachine). Usually static context props.
4.  **Items & Decor:** Smaller objects, often interactable via pickup/move.
    *   `Decor`: (e.g., Chair, Speaker, Chest, TrashCan). Can be pickable/movable based on `object:*` properties.
    *   `Item`: (e.g., Book, Lamp, Candle, Pot, Vase). Usually pickable/movable.
    *   `WallDecor`: (e.g., Mirror, Clock, Picture, Bookshelf). Static wall attachments.
5.  **Gameplay Markers (Editor Only):** Define key locations, invisible in-game.
    *   `PlayerSpawn`, `GhostSpawn`, `VanEntry`: Define starting/transition points. Map to `Behavior.p.util`.
    *   `RoomDef`: Define room areas for `RoomDB`. Variant is room name.
    *   `InvisibleWall`, `CornerWall`: Define collision/occlusion without visuals.
    *   `FakeGhost`, `FakeBreach`: Likely for tutorials/scripting.
6.  **Specialized Light Sources (Editor Only):**
    *   `CeilingLight`, `StreetLight`: Define light source positions without a specific interactive object sprite. State likely controlled by room/time.
7.  **NPCs:**
    *   `NPC`: Placeholder for non-player characters. Variant likely links to dialogue/ID.

## 5. Important Conventions for Map Creation

*   **Default State:** When placing interactive tiles (like `Door`, `Switch`, `*Lamp`) in a `.tmx` map file, **always use the tile representing the default, inactive state.**
    *   Doors should be placed using their **`Closed`** state tile.
    *   Switches and Lamps should be placed using their **`Off`** state tile.
    *   The game's logic (`InteractiveStuff`, room state changes) will handle swapping the entity's components/material to the `Open` or `On` state variant during gameplay. Placing them initially open/on in Tiled will likely lead to incorrect behavior.
*   **`type` is King:** The `type` attribute is the primary determinant of how the game logic treats a tile. Ensure it accurately reflects the intended function.
*   **`sprite:variant` for Identity:** Use variants consistently for objects that should be treated as the same logical item (e.g., all "OakTable" variants belong to the Oak Table type).

## 6. How to Use This Information (For AI)

*   When asked about a specific tile ID from a spritesheet, refer to the corresponding TSX file to find its `<tile>` definition.
*   Identify the `type` attribute to understand the tile's base `Class` and associated core behaviors/components.
*   Check the `<properties>` for `sprite:variant`, `sprite:orientation`, `sprite:state` to understand its specific visual and initial state configuration.
*   Check `object:*` properties for physical interaction details.
*   Remember the default state convention when interpreting map files or generating code related to interactive objects.
*   Use the visual spritesheet images to correlate the TSX data with the intended appearance.


Okay, David. Here's an Addendum section for the `AI_README_MAPS.md` file, providing a tile-by-tile description for the `unhaunter_spritesheetA_3x3x3.tsx` tileset. This integrates the TSX data with the visual appearance from the PNG.


## Addendum: Tile Details for `unhaunter_spritesheetA_3x3x3.tsx`

This section provides a description for each tile ID in the `A3x3x3` tileset, based on its `type` and properties defined in the TSX file and its visual appearance in `spritesheetA_3x3x3.png`.

**Tile Dimensions:** 30x30 (spacing 2, margin 1)
**Columns:** 12
**Tile Count:** 264

*(Note: Tile IDs are 0-based, counting left-to-right, top-to-bottom)*

*   **ID 0-35:** Floors of different types - variant describes the type. Each variant has 4 sprites to be used randomly to give a better look.
*   **ID 36 (Type: Wall, Variant: FlatBlue, Orient: YAxis, State: Partial):** A partial-height blue wall segment oriented vertically (along Y-axis in isometric view). Blocks movement and sight.
*   **ID 37 (Type: Wall, Variant: FlatBlue, Orient: XAxis, State: Partial):** A partial-height blue wall segment oriented horizontally (along X-axis in isometric view). Blocks movement and sight.
*   **ID 38 (Type: Wall, Variant: FlatBlue, Orient: Both, State: Partial):** An inner corner piece for partial-height blue walls. Blocks movement and sight.
*   **ID 39 (Type: Doorway, Variant: Base):** A gap or opening in a wall structure, visually represented by floor/wall edges. Does not block movement or sight.
*   **ID 40 (Type: LowWall, Variant: Fence, Orient: XAxis, State: Partial):** A section of wooden fence oriented horizontally. Blocks movement but likely allows sight over it (`see_through=true`).
*   **ID 41 (Type: Door, Variant: Wood, Orient: YAxis, State: Closed):** A closed wooden door oriented vertically. Blocks movement and sight in this state. Interactive.
*   **ID 42 (Type: Door, Variant: Wood, Orient: XAxis, State: Closed):** A closed wooden door oriented horizontally. Blocks movement and sight in this state. Interactive.
*   **ID 43 (Type: Door, Variant: Wood, Orient: YAxis, State: Open):** An open wooden door oriented vertically. Allows movement and sight. Interactive.
*   **ID 44 (Type: Door, Variant: Wood, Orient: XAxis, State: Open):** An open wooden door oriented horizontally. Allows movement and sight. Interactive.
*   **ID 45 (Type: LowWall, Variant: FlatBlue, Orient: YAxis, State: Partial):** A low blue wall oriented vertically. Blocks movement, allows sight.
*   **ID 46 (Type: LowWall, Variant: FlatBlue, Orient: XAxis, State: Partial):** A low blue wall oriented horizontally. Blocks movement, allows sight.
*   **ID 47 (Undefined in TSX):** Visually shows a wireframe cube. Likely an editor helper/placeholder.
*   **ID 48 (Type: Decor, Variant: GreenChairFront):** A simple green chair viewed from the front/side. `object:*` props define interaction (pickable, movable, weight 2.5). Collidable.
*   **ID 49 (Type: Decor, Variant: GreenChairBack):** The same green chair viewed from the back/side. `object:*` props define interaction. Collidable.
*   **ID 50 (Type: Decor, Variant: Speaker):** A small speaker. `object:*` props define interaction (pickable, movable, weight 1.5). Collidable.
*   **ID 51 (Type: LowWall, Variant: Base, Orient: XAxis, State: Full):** A basic, low grey divider wall oriented horizontally. Blocks movement, allows sight.
*   **ID 52 (Undefined in TSX):** Visually empty.
*   **ID 53 (Type: Door, Variant: Metal, Orient: YAxis, State: Closed):** A closed metal door oriented vertically. Blocks movement and sight. Interactive.
*   **ID 54 (Type: Door, Variant: Metal, Orient: XAxis, State: Closed):** A closed metal door oriented horizontally. Blocks movement and sight. Interactive.
*   **ID 55 (Type: Door, Variant: Metal, Orient: YAxis, State: Open):** An open metal door oriented vertically. Allows movement and sight. Interactive.
*   **ID 56 (Type: Door, Variant: Metal, Orient: XAxis, State: Open):** An open metal door oriented horizontally. Allows movement and sight. Interactive.
*   **ID 57 (Undefined in TSX):** Visually empty.
*   **ID 58 (Type: CornerWall, Variant: FlatBlue):** Invisible corner wall piece for editor use. Defines collision/occlusion.
*   **ID 59 (Type: InvisibleWall, Variant: Base):** Invisible straight wall piece for editor use. Defines collision/occlusion.
*   **ID 60 (Type: Decor, Variant: CedarChest):** A wooden chest. `object:*` props define interaction (pickable, movable, hiding spot, weight 4.5). Collidable.
*   **ID 61 (Type: Decor, Variant: TrashCan):** A metal trash can. `object:*` props define interaction (pickable, movable, weight 4.5). Collidable.
*   **ID 62 (Type: Item, Variant: RedBook):** A small red book. `object:*` props define interaction (pickable, movable, weight 0.5).
*   **ID 63 (Type: Item, Variant: BlueBook):** A small blue book. `object:*` props define interaction (pickable, movable, weight 0.5).
*   **ID 64 (Type: Item, Variant: OilLamp):** An oil lamp base. `object:*` props define interaction.
*   **ID 65 (Type: Item, Variant: OilLampCandle):** An oil lamp with a candle inside (likely 'On' state visual). `object:*` props define interaction.
*   **ID 66 (Type: Item, Variant: CandleOff):** A single candle, unlit. `object:*` props define interaction.
*   **ID 67 (Type: Item, Variant: CandleOn1):** Lit candle, frame 1 of animation/state. `object:*` props define interaction.
*   **ID 68 (Type: Item, Variant: CandleOn2):** Lit candle, frame 2 of animation/state. `object:*` props define interaction.
*   **ID 69 (Type: Item, Variant: CandleConsumed):** A mostly burned-down candle stub. `object:*` props define interaction.
*   **ID 70-71 (Undefined in TSX):** Visually empty.
*   **ID 72 (Type: Item, Variant: TinyTrashCan):** A very small trash can. `object:*` props define interaction (pickable, movable, weight 0.5). Collidable.
*   **ID 73 (Type: Furniture, Variant: BathSink):** A bathroom sink. Collidable.
*   **ID 74 (Type: Furniture, Variant: Shower):** A shower base/stall. Collidable.
*   **ID 75 (Type: Furniture, Variant: Toilet):** A toilet. Collidable.
*   **ID 76 (Undefined in TSX):** Visually empty.
*   **ID 77 (Type: Wall, Variant: OakWood, Orient: YAxis, State: Partial):** Partial-height oak wood wall, vertical orientation. Blocks movement/sight.
*   **ID 78 (Type: Wall, Variant: OakWood, Orient: XAxis, State: Partial):** Partial-height oak wood wall, horizontal orientation. Blocks movement/sight.
*   **ID 79 (Type: Wall, Variant: OakWood, Orient: Both, State: Partial):** Inner corner for partial-height oak wood walls. Blocks movement/sight.
*   **ID 80 (Type: CornerWall, Variant: OakWood):** Invisible corner wall piece (oak version) for editor use.
*   **ID 81 (Type: LowWall, Variant: OakWood, Orient: YAxis, State: Partial):** Low oak wood wall, vertical orientation. Blocks movement, allows sight.
*   **ID 82 (Type: LowWall, Variant: OakWood, Orient: XAxis, State: Partial):** Low oak wood wall, horizontal orientation. Blocks movement, allows sight.
*   **ID 83 (Undefined in TSX):** Visually empty.
*   **ID 84 (Type: Item, Variant: FlowerPotRed):** A plant in a red pot. `object:*` props define interaction. Collidable.
*   **ID 85 (Type: Item, Variant: FlowerPotGreen):** A different plant in a green pot. `object:*` props define interaction. Collidable.
*   **ID 86 (Type: Item, Variant: Vase1):** A style of vase. `object:*` props define interaction. Collidable.
*   **ID 87 (Type: Item, Variant: Vase2):** Another style of vase. `object:*` props define interaction. Collidable.
*   **ID 88 (Type: Item, Variant: Vase3):** A third style of vase. `object:*` props define interaction. Collidable.
*   **ID 89-95 (Undefined in TSX):** Visually empty.
*   **ID 96 (Type: PlayerSpawn):** Editor marker ('P') for player start position. Visually disabled in-game.
*   **ID 97 (Type: VanEntry):** Editor marker ('E->') for van entry/exit point. Visually disabled in-game.
*   **ID 98 (Type: GhostSpawn):** Editor marker (Blue Ghost) for ghost initial spawn/breach location. Visually disabled in-game.
*   **ID 99 (Type: FakeGhost):** Editor marker ('fG') for placing a fake ghost (tutorial/scripting). Visually disabled in-game.
*   **ID 100 (Type: FakeBreach):** Editor marker ('fB') for placing a fake breach (tutorial/scripting). Visually disabled in-game.
*   **ID 101-107 (Undefined in TSX):** Visually empty.
*   **ID 108-179 (Type: RoomDef, Variant: [RoomName]):** Colored diamond shapes used solely in the Tiled editor to define room areas. The variant property holds the room name used by `RoomDB`. Visually disabled in-game. (Covers Foyer, Living Room, ... En Suite).
*   **ID 180-191 (Undefined in TSX):** Visually empty.
*   **ID 192-203 (Type: NPC, Variant: [A-L]):** Placeholder character sprites with letters (A-L) used in the Tiled editor to place NPCs. The variant links to the specific NPC's data/dialogue.
*   **ID 204-263 (Undefined):** Remaining tiles are not defined in the TSX and appear empty visually.


## Addendum: Tile Details for `unhaunter_spritesheetA_6x6x10.tsx`

This section provides a description for each tile ID in the `A6x6x10` tileset, based on its `type` and properties defined in the TSX file and its visual appearance in `spritesheetA_6x6x10.png`.

**Tile Dimensions:** 48x64
**Columns:** 8
**Tile Count:** 64 (Note: Some IDs within this range might not be defined in the TSX or visually present)

*(Note: Tile IDs are 0-based, counting left-to-right, top-to-bottom)*

*   **ID 0 (Type: Switch, Variant: Base, Orient: YAxis, State: Off):** A standard light switch plate, oriented vertically, with the toggle down (Off). Interactive.
*   **ID 1 (Type: Switch, Variant: Base, Orient: YAxis, State: On):** Same switch plate, toggle up (On). Interactive.
*   **ID 2 (Type: WallLamp, Variant: Base, State: Off):** A wall-mounted lamp fixture, visually unlit. Implicitly interactive via room state. Emits light when On.
*   **ID 3 (Type: WallLamp, Variant: Base, State: On):** Same wall lamp fixture, visually lit (glowing slightly). Emits light when On.
*   **ID 4 (Type: FloorLamp, Variant: Base, State: Off):** A standing floor lamp, visually unlit. Interactive. Emits light when On. Collidable base.
*   **ID 5 (Type: FloorLamp, Variant: Base, State: On):** Same floor lamp, visually lit (bulb area glowing). Interactive. Emits light when On. Collidable base.
*   **ID 6 (Type: TableLamp, Variant: Base, State: Off):** A smaller table lamp, visually unlit. Interactive. Emits light when On. Collidable base.
*   **ID 7 (Type: TableLamp, Variant: Base, State: On):** Same table lamp, visually lit (shade glowing). Interactive. Emits light when On. Collidable base.
*   **ID 8 (Type: WallDecor, Variant: Mirror):** A rectangular wall mirror. Static decoration.
*   **ID 9 (Type: WallDecor, Variant: Clock):** A round wall clock. Static decoration.
*   **ID 10 (Type: CeilingLight, Variant: Base, State: On):** Visually represented as a light cone effect. This tile itself is likely hidden in-game, acting only as a light source definition. State: On (Off state is ID 62).
*   **ID 11 (Type: RoomSwitch, Variant: Base, Orient: YAxis, State: Off):** A potentially larger or distinct switch plate, oriented vertically, toggle down (Off). Interactive, likely controls all lights in a room.
*   **ID 12 (Type: RoomSwitch, Variant: Base, Orient: YAxis, State: On):** Same room switch plate, toggle up (On). Interactive.
*   **ID 13 (Type: Breaker, Variant: Base, Orient: YAxis, State: Off):** A main electrical breaker switch, handle down (Off). Interactive.
*   **ID 14 (Type: Breaker, Variant: Base, Orient: YAxis, State: On):** Same breaker switch, handle up (On). Interactive.
*   **ID 15 (Type: StreetLight, Variant: Base, State: On):** The lamp head of a streetlight, always On. Likely placed high or outside, acts as a light source. Visually disabled in-game?
*   **ID 16 (Type: Furniture, Variant: GreenCouch):** A green couch/sofa. Static furniture, provides collision.
*   **ID 17 (Type: Furniture, Variant: RedSofa):** A reddish/maroon sofa. Static furniture, provides collision.
*   **ID 18 (Type: Appliance, Variant: TV):** A flatscreen television on a low stand. Static appliance, provides collision.
*   **ID 19 (Type: WallDecor, Variant: Bookshelf):** A wall-mounted bookshelf filled with books. Static decoration.
*   **ID 20 (Type: Furniture, Variant: Bed):** A standard bed. Static furniture, provides collision, designated as a `hidingspot`.
*   **ID 21 (Type: Furniture, Variant: Bathtub):** A standalone bathtub. Static furniture, provides collision, designated as a `hidingspot`.
*   **ID 22 (Type: Appliance, Variant: WashingMachine):** A front-loading washing machine. Static appliance, provides collision.
*   **ID 23 (Type: Appliance, Variant: Fridge):** A tall refrigerator. Static appliance, provides collision.
*   **ID 24 (Type: Furniture, Variant: GreenTable):** A rectangular table with a green top. Static furniture, provides collision, designated as a `hidingspot`.
*   **ID 25 (Type: Furniture, Variant: OakTable):** Similar table with an oak wood texture. Static furniture, provides collision, designated as a `hidingspot`.
*   **ID 26 (Type: Furniture, Variant: WoodTable):** Similar table with a different wood texture. Static furniture, provides collision, designated as a `hidingspot`.
*   **ID 27 (Type: Furniture, Variant: Desk):** A desk, likely for an office or study. Static furniture, provides collision, designated as a `hidingspot`.
*   **ID 28 (Type: Appliance, Variant: Stove):** A kitchen stove with an oven. Static appliance, provides collision.
*   **ID 29 (Type: Furniture, Variant: Sink):** A kitchen sink, likely part of a counter unit. Static furniture, provides collision.
*   **ID 30 (Type: StairsDown, Variant: Blue, Orient: XAxis):** Stairs oriented horizontally (X-axis), visually descending. Allows floor transition down.
*   **ID 31 (Type: StairsUp, Variant: Blue, Orient: XAxis):** Stairs oriented horizontally (X-axis), visually ascending. Allows floor transition up.
*   **ID 32 (Type: Furniture, Variant: EmptyBookshelf):** A tall, empty wooden bookshelf or cabinet. Static furniture, provides collision.
*   **ID 33 (Type: Furniture, Variant: Bookshelf):** The same bookshelf, filled with books. Static furniture, provides collision.
*   **ID 34-39 (Undefined in TSX):** Visually empty in the spritesheet.
*   **ID 40 (Type: Furniture, Variant: Drawer):** A chest of drawers. Static furniture, provides collision.
*   **ID 41 (Type: Furniture, Variant: Wardrobe):** A tall wardrobe or closet. Static furniture, provides collision, designated as a `hidingspot`.
*   **ID 42-47 (Undefined in TSX):** Visually empty in the spritesheet.
*   **ID 48 (Type: Window, Variant: Window):** A standard window frame. Affects light transmission.
*   **ID 49 (Type: Window, Variant: RedCurtainWindow):** Same window frame with red curtains. Affects light transmission.
*   **ID 50 (Type: Window, Variant: GreenCurtainWindow):** Same window frame with green curtains. Affects light transmission.
*   **ID 51-55 (Undefined in TSX):** Visually empty in the spritesheet.
*   **ID 56 (Type: WallDecor, Variant: BlankPicture):** An empty picture frame. Static decoration.
*   **ID 57 (Type: WallDecor, Variant: GreenPicture):** Same frame with a green-toned picture. Static decoration.
*   **ID 58 (Type: WallDecor, Variant: RedPicture):** Same frame with a red-toned picture. Static decoration.
*   **ID 59-61 (Undefined in TSX):** Visually empty in the spritesheet.
*   **ID 62 (Type: CeilingLight, Variant: Base, State: Off):** Same ceiling light fixture as ID 10, but visually off.
*   **ID 63 (Undefined in TSX):** Visually a wireframe cube, likely an editor helper/placeholder.


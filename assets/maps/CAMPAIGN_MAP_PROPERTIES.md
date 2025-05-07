# Unhaunter Campaign Map Properties (TMX)

This document outlines the specific custom properties required within the Tiled `.tmx` map files that are designated as part of the Unhaunter Campaign Mode. These properties allow the game engine to identify, order, configure, and display campaign missions correctly.

All properties listed here should be defined within the main `<map>` tag's `<properties>` section.

## Required Campaign Properties

1.  **`is_campaign_mission`**
    *   **Type:** `bool`
    *   **Purpose:** A mandatory flag that identifies this map file as a mission intended for the Campaign Mode. If `true`, the map will be parsed and included in the campaign mission list. If `false` or absent, it will be ignored by the campaign loading system.
    *   **Example XML:** `<property name="is_campaign_mission" type="bool" value="true"/>`

2.  **`campaign_order`**
    *   **Type:** `string`
    *   **Purpose:** A string used to sort campaign missions in the selection UI. It dictates the intended progression.
    *   **Format:** Typically `NNL` (two digits for major order, one letter for minor order, e.g., "01A", "01B", "02A", "10C"). Standard string sorting will be applied.
    *   **Example XML:** `<property name="campaign_order" type="string" value="01A"/>`

3.  **`campaign_difficulty`**
    *   **Type:** `string`
    *   **Purpose:** Specifies the fixed difficulty level for this campaign mission. The value must exactly match one of the defined `Difficulty` enum variant names in the codebase (e.g., "TutorialChapter1", "StandardChallenge").
    *   **Example XML:** `<property name="campaign_difficulty" type="string" value="TutorialChapter1"/>`

## Recommended Properties (Leveraged by Campaign & Custom Mission Modes)

These properties are typically already present for map identification and flavor but are crucial for how missions are presented in the UI.

1.  **`display_name`**
    *   **Type:** `string`
    *   **Purpose:** The human-readable name of the map/mission as it will appear in UI lists (e.g., Campaign Mission list, Custom Mission map list).
    *   **Example XML:** `<property name="display_name" value="Stage 1A: The Whispering Shed"/>`

2.  **`flavor_text`**
    *   **Type:** `string` (can be multiline in Tiled)
    *   **Purpose:** A descriptive text or briefing for the mission. Displayed when the mission is highlighted in the Campaign selection UI.
    *   **Example XML:** `<property name="flavor_text" value="A chilling presence has been reported... Your first official case."/>`

3.  **`map_preview_image`**
    *   **Type:** `string`
    *   **Purpose:** A filesystem path (relative to the `assets/` directory) to a preview image or screenshot representing the map. This image is used in the UI for both Campaign mission selection and Custom Mission map selection.
    *   **Recommended Location:** `maps/previews/your_map_name.png`
    *   **Example XML:** `<property name="map_preview_image" type="string" value="maps/previews/stage1a.png"/>`

## Existing Standard Properties (Informational)

These are standard Tiled properties or existing custom properties that provide general map context but are not directly parsed by the campaign system for its core logic (though they are good to maintain).

*   **`author`** (string): Name of the map creator.
*   **`class`** (string): Should be "UnhaunterMap1" for Unhaunter maps.
*   **`location_name`** (string): In-universe name of the location.
*   **`location_address`** (string): In-universe address of the location.

## Example Implementation in TMX:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<map version="1.10" tiledversion="1.10.2" class="UnhaunterMap1" orientation="isometric" renderorder="right-down" width="20" height="20" tilewidth="32" tileheight="16" infinite="0" nextlayerid="2" nextobjectid="1">
 <properties>
  <property name="author" value="deavidsedice"/>
  <property name="display_name" value="Stage 1A: The Whispering Shed"/>
  <property name="flavor_text" value="A chilling presence has been reported in this seemingly innocuous shed. Locals say tools move on their own. Your first official case."/>
  <property name="is_campaign_mission" type="bool" value="true"/>
  <property name="campaign_order" type="string" value="01A"/>
  <property name="campaign_difficulty" type="string" value="TutorialChapter1"/>
  <property name="map_preview_image" type="string" value="img/map_previews/stage1a_shed.png"/>
  <property name="location_address" value="27 Briar Patch Rd"/>
  <property name="location_name" value="The Whispering Shed"/>
 </properties>
 <!-- ... rest of map data (tilesets, layers, etc.) ... -->
</map>
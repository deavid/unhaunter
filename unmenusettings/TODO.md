**`unmenusettings` Crate: Requirements and Design**

**I. Core Purpose:**

*   Provide a UI system within the main menu (AppState::MainMenu) for the player
    to view and modify persistent game settings.
*   The settings menu will only be accessible from the Main Menu. It must not be
    accessible during gameplay.

**II. UI Structure and Navigation:**

1. **Main Menu Integration:**
    *   The main menu (in `unmenu` crate) has a "Settings" option that, when selected, 
        transitions the game to the `AppState::SettingsMenu` state, activating the `unmenusettings` UI.

2. **Hierarchical Structure:**
    *   **Level 1: Category List:** A vertically arranged list of setting categories:
        *   Gameplay
        *   Video
        *   Audio
        *   Controls
        *   Profile

    *   **Level 2: Settings List:** When a category is selected, a new vertically 
    arranged list of individual settings within that category appears, replacing 
    the category list.

    *   **Level 3: Setting Detail View:** When a setting is selected, a detail view 
    appears, replacing the settings list. This view will contain:
        *   A header/title indicating the name of the setting being edited.
        *   UI elements appropriate for editing the setting's value (see "Setting-Specific UI Elements" below).

3. **Navigation:**
    *   **Up/Down Arrow Keys:** Navigate between items in the currently active list (categories, settings, or setting options).
    *   **Enter/Select Key:**
        *   In the category list, selects a category and opens the settings list.
        *   In the settings list, selects a setting and opens the detail view.
        *   In the detail view, confirms the current change.
    *   **Escape/Back Key:**
        *   In the detail view, discards any temporary changes to the setting 
            and returns to the settings list.
        *   In the settings list, discards any temporary changes to the settings
            in that category, reloads the settings for that category from disk, 
            and returns to the category list.
        *   In the category list, just returns to the main menu.

**III. Setting-Specific UI Elements:**

*   **Enum:** Displayed as a vertical list of options (using the enum variants' 
    names using strum::Display). The currently selected option is highlighted.
    Up/Down keys cycle through the options.
*   **Text (String):** This will not be implemented for now. 

*   **Apply Changes Button:** Saves the current temporary settings to disk and
    navigates to the Level 1 Category List. (only present in Level 2 category setting lists).
*   **Revert to Default Button:** Loads default values for the current category,
    saves them to disk, and navigates to the Level 1 Category List.
    (only present in Level 2 category setting lists).
*   **Cancel Button:** Discards any unsaved changes, reloads settings from disk,
    and navigates to the Level 1 Category List.
    (only present in Level 2 category setting lists).
*   **Back Button:** Navigates to the parent level or to the main menu if on level 1.
    (present in Level 1 and Level 3)

**IV. Settings Data Management:**

1. **`unsettings` Crate Integration:**
    *   `unmenusettings` will depend on the `unsettings` crate to access the setting struct definitions (`GameplaySettings`, `VideoSettings`, `AudioSettings`, `ProfileSettings`).
    *   `unsettings` will provide default values for all settings.

2. **Temporary Storage:**
    *   `unmenusettings` will NOT maintain an in-memory copy of the settings.
        Instead it will directly modify the Res<Persistent<XYZSetting>>.
    *   Changes are *not* written to disk immediately when a setting is modified in the detail view.
    *   Changes are written to disk only when the "Apply Changes" button is pressed.

3. **Persistence:**
    *   `bevy-persistent` will be used to manage persistent storage.
    *   Each settings category will be stored in a separate `.ron` file within a `config` directory (native) or using a `/local/config/` prefix (WASM).
    *   The `Persistent<T>` resource will be used for each category's settings struct.
    *   This is already done in the `unsettings` crate and doesn't need further modifications.

**V. Visual Style:**

*   The settings UI should follow the general aesthetic of the main menu
    (colors, fonts, etc.). We will use the `colors` and `TextFont` defined in 
    `uncore` to achieve this.
*   **Selected Items:** The currently selected category, setting, or option
    should be clearly highlighted (e.g., using `colors::MENU_ITEM_COLOR_ON`, likely orange).
*   **Button Styles:** Buttons ("Apply Changes", "Revert to Default", "Cancel/Back")
    are just regular menu items with text only. There are no buttons.

**VI. Error Handling:**

*   If settings cannot be loaded from disk (e.g., file not found, invalid format), 
    default values (provided by the `unsettings` crate) will be used.
*   Errors during loading/saving should be logged to the console using `warn!` or `error!`.
*   No error messages need to be displayed to the user in the UI.

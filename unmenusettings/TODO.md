Settings ->
    *   Gameplay
        * Master Volume
          * 0%
          * 10%
          * 20%
          * 30%
          * ...
        * Music Volume
        * volume_ambient
        * volume_voice_chat
        * sound_output
        * audio_positioning
        * feedback_delay
        * feedback_eq
    *   Video
    *   Audio
    *   Controls


A menu for settings contains: (SettingsMenu)

- A list of options (sub-menus) (MenuItem)
- selected_item_idx: The actual position selected


A MenuItem is:
- pub idx: usize - The relative position so that keyboard arrows work (up and down arrow select menu)
- pub on_activate: MenuEvent - The event to trigger on Key Return/Enter
- Text to display for the Menu


Menus have 3 levels:

- The list of the type/category/file settings to edit (Audio, Video, etc)
- The selection setting inside the file (Audio / Music Volume)
- The value selection for the particular setting (10% -> Music Volume)



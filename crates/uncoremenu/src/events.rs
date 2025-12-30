use bevy::prelude::*;
use uncore_resources::AppState;

/// Event sent when keyboard navigation (up/down arrows) changes the selected menu item.
/// This event is distinct from hover-based selection to enable specific behaviors like
/// auto-scrolling in scrollable menus. The usize parameter represents the index of
/// the newly selected item.
#[derive(Message, Debug, Clone, Copy)]
pub struct KeyboardNavigate(pub usize);

/// Event sent when a menu item is clicked
#[derive(Message, Debug, Clone, Copy)]
pub struct MenuItemClicked {
    pub state: AppState,
    pub pos: usize,
}

/// Event sent when keyboard navigation changes the selected item
#[derive(Message, Debug, Clone, Copy)]
pub struct MenuItemSelected(pub usize);

/// Event sent when ESC is pressed in a menu
#[derive(Message, Debug, Clone, Copy)]
pub struct MenuEscapeEvent;

use bevy::prelude::*;

/// Event sent when keyboard navigation (up/down arrows) changes the selected menu item.
/// This event is distinct from hover-based selection to enable specific behaviors like
/// auto-scrolling in scrollable menus. The usize parameter represents the index of
/// the newly selected item.
#[derive(Event, Debug, Clone, Copy)]
pub struct KeyboardNavigate(pub usize);

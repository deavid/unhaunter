use bevy::prelude::*;

/// A component that marks an entity as a menu item that can be interacted with
#[derive(Component, Debug)]
pub struct MenuItemInteractive {
    /// Identifier for this menu item, used for tracking selection
    pub identifier: usize,
    /// Whether this item is currently selected
    pub selected: bool,
}

/// A component that marks an entity as containing menu items
#[derive(Component, Debug)]
pub struct MenuRoot {
    /// The currently selected menu item identifier
    pub selected_item: usize,
}

/// Component that tracks if mouse has moved since menu was displayed
/// Used to prevent hover selection on initial render
#[derive(Component, Debug, Default)]
pub struct MenuMouseTracker {
    /// Whether the mouse has moved since the menu was displayed
    pub mouse_moved: bool,
}

/// A component that marks an entity as the background for a menu
#[derive(Component, Debug)]
pub struct MenuBackground;

/// A component that marks an entity as the content area for a menu
#[derive(Component, Debug)]
pub struct MenuContentArea;

/// A component that marks an entity as the left strip for a menu
#[derive(Component, Debug)]
pub struct MenuStrip;

/// A component that marks an entity as the help text for a menu
#[derive(Component, Debug)]
pub struct MenuHelpText;

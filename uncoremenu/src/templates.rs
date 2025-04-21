use crate::components::*;
use bevy::prelude::*;
use uncore::colors;
use uncore::platform::plt::{FONT_SCALE, UI_SCALE, VERSION};
use uncore::types::root::game_assets::GameAssets;

/// Creates a standard menu background with the background image
pub fn create_background(parent: &mut ChildBuilder, handles: &GameAssets) {
    parent
        .spawn(ImageNode {
            image: handles.images.menu_background.clone(),
            ..default()
        })
        .insert(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .insert(ZIndex(-10)) // Ensure it's behind other elements
        .insert(MenuBackground);
}

/// Creates a standard menu logo
pub fn create_logo(parent: &mut ChildBuilder, handles: &GameAssets) {
    parent
        .spawn(ImageNode {
            image: handles.images.title.clone(),
            ..default()
        })
        .insert(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(16.0 * UI_SCALE),
            top: Val::Px(16.0 * UI_SCALE),
            width: Val::Px(350.0 * UI_SCALE),
            height: Val::Auto,
            aspect_ratio: Some(130.0 / 17.0), // Maintain logo aspect ratio
            ..default()
        })
        .insert(ZIndex(0));
}

/// Creates a standard menu left strip
pub fn create_menu_strip<'a, T: Component + Copy>(
    parent: &'a mut ChildBuilder,
    _handles: &GameAssets,  // Explicit unused parameter
    _items: &[(T, String)], // Explicit unused parameter
    selected_item_idx: usize,
) -> EntityCommands<'a> {
    let strip_color = Color::Srgba(Srgba {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
        alpha: 0.6, // Semi-transparent black for the strip
    });

    let mut entity_cmd = parent.spawn(Node {
        position_type: PositionType::Absolute,
        left: Val::Px(32.0 * UI_SCALE),
        top: Val::Px(0.0), // Align to top
        width: Val::Px(300.0 * UI_SCALE),
        height: Val::Percent(100.0), // Full height
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center, // Center items vertically
        align_items: AlignItems::FlexStart,      // Align items to the left
        padding: UiRect::horizontal(Val::Px(20.0 * UI_SCALE)), // Padding inside strip
        row_gap: Val::Px(16.0 * UI_SCALE),
        ..default()
    });

    entity_cmd
        .insert(BackgroundColor(strip_color))
        .insert(ZIndex(-5)) // Behind logo, above background
        .insert(MenuRoot {
            selected_item: selected_item_idx,
        })
        .insert(MenuStrip);

    entity_cmd
}

/// Creates a standard menu item within a strip
pub fn create_menu_item<'a>(
    strip: &'a mut ChildBuilder,
    text: impl Into<String>,
    idx: usize,
    is_selected: bool,
    handles: &GameAssets,
) -> EntityCommands<'a> {
    let text: String = text.into();
    warn!("Creating menu item {} with idx {}", text, idx);

    // Define colors for menu items
    let selected_color = colors::MENU_ITEM_COLOR_ON;
    let unselected_color = colors::MENU_ITEM_COLOR_OFF;

    let mut entity_cmd = strip.spawn(Node {
        padding: UiRect::all(Val::Px(10.0 * UI_SCALE)), // Add padding for better click target
        margin: UiRect::vertical(Val::Px(5.0 * UI_SCALE)), // Add vertical margin for spacing
        ..default()
    });

    entity_cmd
        .insert(Button) // Add Button component for interactivity
        .insert(Interaction::None)
        .insert(MenuItemInteractive {
            identifier: idx,
            selected: is_selected,
        })
        .with_children(|parent| {
            parent
                .spawn(Text::new(text.clone()))
                .insert(TextFont {
                    font: handles.fonts.londrina.w300_light.clone(),
                    font_size: 38.0 * FONT_SCALE,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                })
                .insert(TextColor(if is_selected {
                    selected_color
                } else {
                    unselected_color
                }));
        });

    entity_cmd
}

/// Creates a standard help text at the bottom of the screen
pub fn create_help_text(parent: &mut ChildBuilder, handles: &GameAssets, text: Option<String>) {
    let default_help_text = format!(
        "Unhaunter {}    |    [Up]/[Down]: Change    |    [Enter]: Select    |    [ESC]: Go Back",
        VERSION
    );

    parent
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0 * UI_SCALE), // Small margin from bottom
            left: Val::Percent(0.0),          // Align to left edge of parent
            width: Val::Percent(100.0),       // Full width
            justify_content: JustifyContent::Center, // Center the text container
            ..default()
        })
        .insert(MenuHelpText)
        .with_children(|bottom_bar| {
            bottom_bar
                .spawn(Text::new(text.unwrap_or(default_help_text)))
                .insert(TextFont {
                    font: handles.fonts.titillium.w300_light.clone(),
                    font_size: 14.0 * FONT_SCALE,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                })
                .insert(TextColor(colors::MENU_ITEM_COLOR_OFF))
                .insert(TextLayout {
                    // Center align text within its container
                    justify: JustifyText::Center,
                    ..default()
                });
        });
}

/// Creates a complete standard menu layout
pub fn create_standard_menu_layout<T: Component + Copy>(
    commands: &mut Commands,
    handles: &GameAssets,
    items: &[(T, String)],
    selected_item_idx: usize,
    help_text: Option<String>,
    menu_marker: impl Component,
) -> Entity {
    warn!("Creating standard menu layout with {} items", items.len());
    // Root node to hold everything
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute, // Make root absolute to position children absolutely
            ..default()
        })
        .insert(menu_marker) // Add the marker component for easy cleanup
        .with_children(|parent| {
            // Background
            create_background(parent, handles);

            // Logo
            create_logo(parent, handles);

            // Left strip with menu items
            let mut strip_entity = create_menu_strip(parent, handles, items, selected_item_idx);

            // Add mouse tracker to prevent unwanted initial hover selection
            strip_entity.insert(MenuMouseTracker::default());

            // Add menu items to the strip
            strip_entity.with_children(|strip| {
                for (idx, (menu_id, text)) in items.iter().enumerate() {
                    let mut entity_cmd = create_menu_item(
                        strip,
                        text.clone(),
                        idx,
                        idx == selected_item_idx,
                        handles,
                    );
                    entity_cmd.insert(menu_id.to_owned());
                }
            });

            // Help text
            create_help_text(parent, handles, help_text);
        })
        .id()
}

/// Creates a content area with a grid layout, suitable for map/difficulty selection
pub fn create_grid_content_area<'a>(
    parent: &'a mut ChildBuilder,
    _handles: &GameAssets, // Explicit unused parameter
    columns: usize,
) -> EntityCommands<'a> {
    let mut entity_cmd = parent.spawn(Node {
        position_type: PositionType::Absolute,
        left: Val::Px(350.0 * UI_SCALE), // After menu strip
        top: Val::Px(100.0 * UI_SCALE),  // Below logo
        width: Val::Auto,
        height: Val::Auto,
        max_width: Val::Percent(60.0),
        display: Display::Grid,
        grid_template_columns: vec![GridTrack::flex(1.0); columns],
        column_gap: Val::Px(20.0 * UI_SCALE),
        row_gap: Val::Px(20.0 * UI_SCALE),
        ..default()
    });

    entity_cmd.insert(MenuContentArea);

    entity_cmd
}

/// Creates a content area with a description panel, suitable for settings
pub fn create_description_content_area<'a>(
    parent: &'a mut ChildBuilder,
    handles: &GameAssets,
    description: impl Into<String>,
) -> EntityCommands<'a> {
    let mut entity_cmd = parent.spawn(Node {
        position_type: PositionType::Absolute,
        left: Val::Px(350.0 * UI_SCALE), // After menu strip
        top: Val::Px(100.0 * UI_SCALE),  // Below logo
        width: Val::Auto,
        height: Val::Auto,
        max_width: Val::Percent(60.0),
        flex_direction: FlexDirection::Column,
        ..default()
    });

    entity_cmd.insert(MenuContentArea).with_children(|content| {
        // Description text
        content
            .spawn(Text::new(description))
            .insert(TextFont {
                font: handles.fonts.titillium.w300_light.clone(),
                font_size: 19.0 * FONT_SCALE,
                font_smoothing: bevy::text::FontSmoothing::AntiAliased,
            })
            .insert(TextColor(colors::MENU_ITEM_COLOR_OFF))
            .insert(Node {
                margin: UiRect::all(Val::Px(20.0 * UI_SCALE)),
                ..default()
            });
    });

    entity_cmd
}

/// Creates a scrollable content area with text, suitable for manual pages
pub fn create_scrollable_content_area<'a>(
    parent: &'a mut ChildBuilder,
    handles: &GameAssets,
    content: impl Into<String>,
) -> EntityCommands<'a> {
    let mut entity_cmd = parent.spawn(Node {
        position_type: PositionType::Absolute,
        left: Val::Px(350.0 * UI_SCALE), // After menu strip
        top: Val::Px(100.0 * UI_SCALE),  // Below logo
        width: Val::Auto,
        height: Val::Percent(80.0),
        max_width: Val::Percent(60.0),
        overflow: Overflow::clip(),
        ..default()
    });

    entity_cmd
        .insert(MenuContentArea)
        .with_children(|content_area| {
            // Scrollable container
            content_area
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Auto,
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::scroll_y(),
                    ..default()
                })
                .with_children(|scroll| {
                    // Content text
                    scroll
                        .spawn(Text::new(content))
                        .insert(TextFont {
                            font: handles.fonts.chakra.w300_light.clone(),
                            font_size: 16.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(colors::MENU_ITEM_COLOR_OFF))
                        .insert(Node {
                            margin: UiRect::all(Val::Px(20.0 * UI_SCALE)),
                            ..default()
                        });
                });
        });

    entity_cmd
}

/// Creates a navigation button bar for manual/tutorial pages
pub fn create_navigation_bar(parent: &mut ChildBuilder, handles: &GameAssets) -> Entity {
    parent
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(350.0 * UI_SCALE),
            bottom: Val::Px(50.0 * UI_SCALE),
            width: Val::Auto,
            height: Val::Px(40.0 * UI_SCALE),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(20.0 * UI_SCALE),
            ..default()
        })
        .with_children(|nav| {
            // Previous button
            nav.spawn(Button)
                .insert(Node {
                    width: Val::Px(120.0 * UI_SCALE),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .insert(BackgroundColor(Color::BLACK.with_alpha(0.2)))
                .with_children(|btn| {
                    btn.spawn(Text::new("Previous"))
                        .insert(TextFont {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 24.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(Color::WHITE));
                });

            // Next button
            nav.spawn(Button)
                .insert(Node {
                    width: Val::Px(120.0 * UI_SCALE),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .insert(BackgroundColor(Color::BLACK.with_alpha(0.2)))
                .with_children(|btn| {
                    btn.spawn(Text::new("Next"))
                        .insert(TextFont {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 24.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(Color::WHITE));
                });
        })
        .id()
}

/// Creates a breadcrumb-style navigation in the left strip, showing the current section path
pub fn create_breadcrumb_navigation<'a>(
    parent: &'a mut ChildBuilder,
    handles: &GameAssets,
    main_text: impl Into<String>,
    sub_text: impl Into<String>,
) -> EntityCommands<'a> {
    let strip_color = Color::Srgba(Srgba {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
        alpha: 0.6, // Semi-transparent black for the strip
    });

    let mut entity_cmd = parent.spawn(Node {
        position_type: PositionType::Absolute,
        left: Val::Px(32.0 * UI_SCALE),
        top: Val::Px(0.0), // Align to top
        width: Val::Px(300.0 * UI_SCALE),
        height: Val::Percent(100.0), // Full height
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::FlexStart, // Align near the top
        align_items: AlignItems::FlexStart,         // Align items to the left
        padding: UiRect::new(
            Val::Px(20.0 * UI_SCALE),  // left
            Val::Px(20.0 * UI_SCALE),  // right
            Val::Px(120.0 * UI_SCALE), // top
            Val::Px(0.0),              // bottom
        ),
        row_gap: Val::Px(8.0 * UI_SCALE),
        ..default()
    });

    entity_cmd
        .insert(BackgroundColor(strip_color))
        .insert(ZIndex(-5)) // Behind logo, above background
        .insert(MenuStrip)
        .insert(MenuMouseTracker::default())
        .with_children(|strip| {
            // Main breadcrumb item (e.g., "New Game")
            strip
                .spawn(Node {
                    padding: UiRect::all(Val::Px(10.0 * UI_SCALE)),
                    ..default()
                })
                .with_children(|node| {
                    node.spawn(Text::new(main_text))
                        .insert(TextFont {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 38.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(Color::WHITE));
                });

            // Sub-breadcrumb item (e.g., "Select Map")
            strip
                .spawn(Node {
                    padding: UiRect::new(
                        Val::Px(30.0 * UI_SCALE), // left - indentation
                        Val::Px(10.0 * UI_SCALE), // right
                        Val::Px(10.0 * UI_SCALE), // top
                        Val::Px(10.0 * UI_SCALE), // bottom
                    ),
                    ..default()
                })
                .with_children(|node| {
                    node.spawn(Text::new(sub_text))
                        .insert(TextFont {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 30.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(colors::MENU_ITEM_COLOR_ON)); // Orange color for current item
                });
        });

    entity_cmd
}

/// Creates a content area with a semi-transparent background and selectable items
pub fn create_selectable_content_area<'a>(
    parent: &'a mut ChildBuilder,
    _handles: &GameAssets, // Explicit unused parameter
    initial_selection: usize,
) -> EntityCommands<'a> {
    let content_bg_color = Color::Srgba(Srgba {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
        alpha: 0.5, // Semi-transparent black background
    });

    let mut entity_cmd = parent.spawn(Node {
        position_type: PositionType::Absolute,
        left: Val::Px(350.0 * UI_SCALE), // After menu strip
        top: Val::Px(100.0 * UI_SCALE),  // Below logo
        width: Val::Percent(60.0),
        height: Val::Percent(70.0),
        flex_direction: FlexDirection::Row, // Split into two columns
        padding: UiRect::all(Val::Px(15.0 * UI_SCALE)),
        ..default()
    });

    entity_cmd
        .insert(BackgroundColor(content_bg_color))
        .insert(MenuContentArea)
        .insert(MenuRoot {
            selected_item: initial_selection,
        })
        .insert(MenuMouseTracker::default());

    entity_cmd
}

/// Creates a selectable item for the content area
pub fn create_content_item<'a>(
    parent: &'a mut ChildBuilder,
    text: impl Into<String>,
    idx: usize,
    _is_selected: bool, // Always start unselected, systems will update this
    handles: &GameAssets,
) -> EntityCommands<'a> {
    let text: String = text.into();
    // Always start with not selected to avoid UI jumping
    let is_selected = false;

    // Define colors and background for items
    let selected_color = colors::MENU_ITEM_COLOR_ON;
    let unselected_color = colors::MENU_ITEM_COLOR_OFF;
    let selected_bg = Color::srgba(0.3, 0.3, 0.3, 0.1);

    let mut entity_cmd = parent.spawn(Node {
        width: Val::Percent(100.0),
        padding: UiRect::all(Val::Px(8.0 * UI_SCALE)),
        margin: UiRect::vertical(Val::Px(2.0 * UI_SCALE)),
        ..default()
    });

    entity_cmd
        .insert(Button)
        .insert(BackgroundColor(if is_selected {
            selected_bg
        } else {
            Color::NONE
        }))
        .insert(Interaction::None)
        .insert(MenuItemInteractive {
            identifier: idx,
            selected: is_selected,
        })
        // Allow mouse wheel scrolling to work through this element
        .insert(PickingBehavior {
            should_block_lower: false,
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(Text::new(text))
                .insert(TextFont {
                    font: handles.fonts.titillium.w600_semibold.clone(),
                    font_size: 23.0 * FONT_SCALE,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                })
                // Allow mouse wheel scrolling to work through text as well
                .insert(PickingBehavior {
                    should_block_lower: false,
                    ..default()
                })
                .insert(TextColor(if is_selected {
                    selected_color
                } else {
                    unselected_color
                }));
        });

    entity_cmd
}

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::focus::HoverMap;
use bevy::prelude::*;
use bevy::ui::ComputedNode;
use bevy::ui::ScrollPosition;
use bevy::utils::Instant;
use uncore::events::map_selected::MapSelectedEvent;
use uncore::resources::maps::Maps;
use uncore::states::AppState;
use uncore::states::MapHubState;
use uncore::types::root::game_assets::GameAssets;
use uncoremenu::components::{MenuItemInteractive, MenuMouseTracker};
use uncoremenu::events::KeyboardNavigate;
use uncoremenu::systems::{MenuItemClicked, MenuItemSelected};
use uncoremenu::templates;

/// UI component marker for the map selection screen
#[derive(Component, Debug)]
pub struct MapSelectionUI;

/// Component that links a menu item to its corresponding map index
#[derive(Component, Debug)]
pub struct MapSelectionItem {
    pub map_idx: usize,
}

/// Marker component for the scrollable container that holds the map list
#[derive(Component, Debug)]
pub struct ScrollableListContainer;

/// Component that identifies the scrollbar container
#[derive(Component, Debug)]
pub struct ScrollbarContainer;

/// Component that identifies the scrollbar track
#[derive(Component, Debug)]
pub struct ScrollbarTrack;

/// Component that identifies the scrollbar thumb
#[derive(Component, Debug)]
pub struct ScrollbarThumb;

/// Component that identifies the scrollbar up arrow
#[derive(Component, Debug)]
pub struct ScrollbarUpArrow;

/// Component that identifies the scrollbar down arrow
#[derive(Component, Debug)]
pub struct ScrollbarDownArrow;

/// Resource that tracks the state of map selection, including timing for event debouncing
#[derive(Resource, Debug)]
pub struct MapSelectionState {
    pub selected_map_idx: usize,
    pub state_entered_at: Instant,
}

impl Default for MapSelectionState {
    fn default() -> Self {
        Self {
            selected_map_idx: 0,
            state_entered_at: Instant::now(),
        }
    }
}

/// Sets up all systems and resources needed for the map selection screen
pub fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(MapHubState::MapSelection), setup_systems)
        .add_systems(OnExit(MapHubState::MapSelection), cleanup_systems)
        .add_systems(
            Update,
            (
                handle_menu_events,
                handle_esc_key,
                update_scroll_position,
                update_scrollbar,
                handle_scrollbar_interactions,
            )
                .run_if(in_state(MapHubState::MapSelection)),
        )
        .add_systems(
            PostUpdate,
            ensure_selected_item_visible.run_if(in_state(MapHubState::MapSelection)),
        )
        .init_resource::<MapSelectionState>()
        .add_event::<MapSelectedEvent>()
        .add_event::<KeyboardNavigate>();
}

/// Creates the initial UI and state for the map selection screen
pub fn setup_systems(mut commands: Commands, maps: Res<Maps>, handles: Res<GameAssets>) {
    setup_ui(&mut commands, &handles, &maps);
    commands.insert_resource(MapSelectionState {
        selected_map_idx: 0,
        state_entered_at: Instant::now(),
    });
}

/// Cleans up the map selection UI when exiting the state
pub fn cleanup_systems(mut commands: Commands, qtui: Query<Entity, With<MapSelectionUI>>) {
    for e in qtui.iter() {
        commands.entity(e).despawn_recursive();
    }
}

/// Handles menu selection and click events, managing transitions between states
pub fn handle_menu_events(
    mut menu_selection: EventReader<MenuItemSelected>,
    mut menu_clicks: EventReader<MenuItemClicked>,
    mut map_selection_state: ResMut<MapSelectionState>,
    maps: Res<Maps>,
    mut next_state: ResMut<NextState<MapHubState>>,
    mut root_next_state: ResMut<NextState<AppState>>,
    mut ev_map_selected: EventWriter<MapSelectedEvent>,
) {
    const GRACE_PERIOD_SECS: f32 = 0.1;
    let time_in_state = map_selection_state.state_entered_at.elapsed().as_secs_f32();

    if time_in_state < GRACE_PERIOD_SECS {
        menu_clicks.clear();
        menu_selection.clear();
        return;
    }

    for ev in menu_selection.read() {
        map_selection_state.selected_map_idx = ev.0;
    }

    for ev in menu_clicks.read() {
        let selected_idx = ev.0;
        if selected_idx < maps.maps.len() {
            ev_map_selected.send(MapSelectedEvent {
                map_idx: selected_idx,
            });
            next_state.set(MapHubState::DifficultySelection);
        } else {
            root_next_state.set(AppState::MainMenu);
            next_state.set(MapHubState::None);
        }
    }
}

/// Handles ESC key press to return to the main menu
pub fn handle_esc_key(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut root_next_state: ResMut<NextState<AppState>>,
    mut next_state: ResMut<NextState<MapHubState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        root_next_state.set(AppState::MainMenu);
        next_state.set(MapHubState::None);
    }
}

/// Ensures that the keyboard-selected item remains visible in the scrollable list by adjusting scroll position
pub fn ensure_selected_item_visible(
    mut keyboard_nav_events: EventReader<KeyboardNavigate>,
    mut container_query: Query<
        (Entity, &Node, &ComputedNode, &mut ScrollPosition),
        With<ScrollableListContainer>,
    >,
    items_query: Query<(&Node, &ComputedNode, &GlobalTransform, &MenuItemInteractive)>,
) {
    if keyboard_nav_events.is_empty() {
        return;
    }

    const SCROLL_MARGIN_PX: f32 = 80.0;

    if let Ok((_entity, _node, container_computed, mut scroll_position)) =
        container_query.get_single_mut()
    {
        let container_height = container_computed.size().y;
        let current_scroll_y = scroll_position.offset_y;

        let mut sorted_items_data: Vec<_> = items_query
            .iter()
            .map(|(_, computed, _, item)| (item.identifier, computed.size().y))
            .collect();
        sorted_items_data.sort_by_key(|(id, _)| *id);

        for ev in keyboard_nav_events.read() {
            let selected_idx = ev.0;
            let mut current_y_offset = 0.0;
            let mut selected_item_top_y = 0.0;
            let mut selected_item_height = 0.0;
            let mut found = false;

            for (id, height) in sorted_items_data.iter() {
                if *id == selected_idx {
                    selected_item_top_y = current_y_offset;
                    selected_item_height = *height;
                    found = true;
                    break;
                }
                current_y_offset += height;
            }

            if !found {
                continue;
            }

            let item_bottom_in_content = selected_item_top_y + selected_item_height;
            let visible_top_with_margin = current_scroll_y + SCROLL_MARGIN_PX;
            let visible_bottom_with_margin = current_scroll_y + container_height - SCROLL_MARGIN_PX;

            let mut new_scroll_y = current_scroll_y;

            if selected_item_top_y < visible_top_with_margin {
                new_scroll_y = selected_item_top_y - SCROLL_MARGIN_PX;
            } else if item_bottom_in_content > visible_bottom_with_margin {
                new_scroll_y = item_bottom_in_content - container_height + SCROLL_MARGIN_PX;
            }

            new_scroll_y = new_scroll_y.max(0.0);

            if (new_scroll_y - current_scroll_y).abs() > 0.1 {
                scroll_position.offset_y = new_scroll_y;
            }

            break;
        }
    }
}

/// Updates the scroll position based on mouse wheel input when hovering over the list
pub fn update_scroll_position(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut scrolled_node_query: Query<&mut ScrollPosition, With<ScrollableListContainer>>,
) {
    const LINE_HEIGHT: f32 = 21.0;

    for mouse_wheel_event in mouse_wheel_events.read() {
        let scroll_amount = match mouse_wheel_event.unit {
            MouseScrollUnit::Line => mouse_wheel_event.y * LINE_HEIGHT,
            MouseScrollUnit::Pixel => mouse_wheel_event.y,
        };

        if scroll_amount == 0.0 {
            continue;
        }

        for (_pointer, pointer_map) in hover_map.iter() {
            for (entity, _hit) in pointer_map.iter() {
                if let Ok(mut scroll_position) = scrolled_node_query.get_mut(*entity) {
                    scroll_position.offset_y -= scroll_amount;
                    scroll_position.offset_y = scroll_position.offset_y.max(0.0);
                }
            }
        }
    }
}

/// Updates the scrollbar thumb position and visual state based on the current scroll position
pub fn update_scrollbar(
    // Query container node, scroll position, and its children
    scroll_container_query: Query<
        (&ScrollPosition, &ComputedNode, &Children),
        With<ScrollableListContainer>,
    >,
    // Query the computed nodes of the items within the container
    item_node_query: Query<&ComputedNode>,
    track_query: Query<&ComputedNode, With<ScrollbarTrack>>,
    mut thumb_query: Query<(&mut Node, &Children), With<ScrollbarThumb>>,
    mut thumb_image_query: Query<&mut ImageNode>,
    arrow_up_query: Query<
        (&Children, &Interaction),
        (With<ScrollbarUpArrow>, Without<ScrollbarDownArrow>),
    >,
    arrow_down_query: Query<
        (&Children, &Interaction),
        (With<ScrollbarDownArrow>, Without<ScrollbarUpArrow>),
    >,
) {
    // Get container info and children list
    if let Ok((scroll_position, container_node, children)) = scroll_container_query.get_single() {
        let scroll_y = scroll_position.offset_y;
        let container_height = container_node.size().y;

        // Calculate actual content height by summing children heights
        let mut content_height = 0.0;
        for child_entity in children.iter() {
            // Get the ComputedNode for each child item
            if let Ok(child_node) = item_node_query.get(*child_entity) {
                // Add the child's height to the total content height
                content_height += child_node.size().y + 2.0;
                // Note: This assumes no vertical margin/padding between items.
                // If items have margins, this calculation might need adjustment.
            }
        }

        // Determine if scrolling is possible
        let has_scrollable_content = content_height > container_height;

        // Update up/down arrow colors based on scroll position
        if let Ok((children, interaction)) = arrow_up_query.get_single() {
            if let Some(child) = children.first() {
                if let Ok(mut image) = thumb_image_query.get_mut(*child) {
                    // Set transparency/color for up arrow
                    let at_top = scroll_y <= 0.1;
                    let is_disabled = at_top || !has_scrollable_content;

                    // If disabled, use a very transparent gray color regardless of interaction
                    image.color = if is_disabled {
                        Color::srgba(0.5, 0.5, 0.5, 0.15) // Much more transparent and gray when disabled
                    } else {
                        // Normal coloring based on interaction state
                        match *interaction {
                            Interaction::Pressed => uncore::colors::MENU_ITEM_COLOR_ON,
                            Interaction::Hovered => {
                                uncore::colors::MENU_ITEM_COLOR_ON.with_alpha(0.8)
                            }
                            Interaction::None => Color::WHITE,
                        }
                    };
                }
            }
        }

        if let Ok((children, interaction)) = arrow_down_query.get_single() {
            if let Some(child) = children.first() {
                if let Ok(mut image) = thumb_image_query.get_mut(*child) {
                    // Set transparency/color for down arrow
                    // Use the calculated content_height to determine if at the bottom
                    let at_bottom = scroll_y >= (content_height - container_height - 0.1).max(0.0);
                    let is_disabled = at_bottom || !has_scrollable_content;

                    // If disabled, use a very transparent gray color regardless of interaction
                    image.color = if is_disabled {
                        Color::srgba(0.5, 0.5, 0.5, 0.15) // Much more transparent and gray when disabled
                    } else {
                        // Normal coloring based on interaction state
                        match *interaction {
                            Interaction::Pressed => uncore::colors::MENU_ITEM_COLOR_ON,
                            Interaction::Hovered => {
                                uncore::colors::MENU_ITEM_COLOR_ON.with_alpha(0.8)
                            }
                            Interaction::None => Color::WHITE,
                        }
                    };
                }
            }
        }

        // Update thumb position and visibility
        if let Ok((mut thumb_node, children)) = thumb_query.get_single_mut() {
            // If we have a track, calculate the thumb position
            if let Ok(track_node) = track_query.get_single() {
                let track_height = track_node.size().y;

                if has_scrollable_content {
                    // Fixed thumb height - don't dynamically resize the thumb
                    // Instead, adjust the visible track area that the thumb can move in

                    // Get the actual thumb height from the node
                    let thumb_height = 80.0;

                    // Calculate scroll percentage using the actual content_height
                    let max_scroll = content_height - container_height; // Use actual content height
                    let scroll_percent = if max_scroll > 0.0 {
                        (scroll_y / max_scroll).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };

                    // Calculate thumb position - keep the thumb within the visible track area
                    let available_track = track_height - thumb_height;
                    let thumb_position = scroll_percent * available_track;

                    // Update thumb position
                    thumb_node.top = Val::Px(thumb_position);

                    // Make thumb visible
                    if let Some(child) = children.first() {
                        if let Ok(mut image) = thumb_image_query.get_mut(*child) {
                            image.color = Color::WHITE.with_alpha(0.2);
                        }
                    }
                } else {
                    // If there's no scrollable content, hide the thumb
                    if let Some(child) = children.first() {
                        if let Ok(mut image) = thumb_image_query.get_mut(*child) {
                            image.color = Color::srgba(1.0, 1.0, 1.0, 0.0);
                        }
                    }
                }
            }
        }
    }
}

/// Handles click interactions with the scrollbar buttons to scroll the content
pub fn handle_scrollbar_interactions(
    mut scroll_container_query: Query<(Entity, &mut ScrollPosition), With<ScrollableListContainer>>,
    up_arrow_query: Query<&Interaction, (With<ScrollbarUpArrow>, Changed<Interaction>)>,
    down_arrow_query: Query<&Interaction, (With<ScrollbarDownArrow>, Changed<Interaction>)>,
    thumb_query: Query<
        (&Interaction, &GlobalTransform),
        (With<ScrollbarThumb>, Changed<Interaction>),
    >,
    mut drag_state: Local<Option<(Entity, Vec2)>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
) {
    // Handle the up arrow button
    if let Ok(interaction) = up_arrow_query.get_single() {
        if *interaction == Interaction::Pressed {
            if let Ok((_, mut scroll_position)) = scroll_container_query.get_single_mut() {
                // Scroll up by 60px when clicking the up arrow
                scroll_position.offset_y = (scroll_position.offset_y - 60.0).max(0.0);
            }
        }
    }

    // Handle the down arrow button
    if let Ok(interaction) = down_arrow_query.get_single() {
        if *interaction == Interaction::Pressed {
            if let Ok((_, mut scroll_position)) = scroll_container_query.get_single_mut() {
                // Scroll down by 60px when clicking the down arrow
                scroll_position.offset_y += 60.0;
            }
        }
    }

    // Handle thumb drag start
    if let Ok((interaction, _)) = thumb_query.get_single() {
        if *interaction == Interaction::Pressed && mouse_button_input.pressed(MouseButton::Left) {
            if let Ok((entity, _)) = scroll_container_query.get_single() {
                if drag_state.is_none() {
                    // Start dragging - store the current cursor position and scroll container entity
                    if let Some(event) = cursor_moved_events.read().last() {
                        *drag_state = Some((entity, event.position));
                    }
                }
            }
        }
    }

    // Handle ongoing thumb drag
    if let Some((entity, start_pos)) = *drag_state {
        if !mouse_button_input.pressed(MouseButton::Left) {
            // Mouse released, stop dragging
            *drag_state = None;
        } else if let Some(event) = cursor_moved_events.read().last() {
            // Mouse moved while dragging
            let delta_y = event.position.y - start_pos.y;

            if let Ok((_, mut scroll_position)) = scroll_container_query.get_mut(entity) {
                // Make scroll speed relative to the estimated content size
                // This is a rough approximation - adjust the multiplier as needed
                scroll_position.offset_y = (scroll_position.offset_y + delta_y * 1.5).max(0.0);

                // Update the stored start position for next frame's calculation
                *drag_state = Some((entity, event.position));
            }
        }
    }
}

/// Creates the map selection UI layout including background, logo, breadcrumb navigation, and scrollable list
pub fn setup_ui(commands: &mut Commands, handles: &GameAssets, maps: &Maps) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .insert(MapSelectionUI)
        .with_children(|parent| {
            templates::create_background(parent, handles);
            templates::create_logo(parent, handles);
            templates::create_breadcrumb_navigation(parent, handles, "New Game", "Select Map");

            let mut content_area = templates::create_selectable_content_area(parent, handles, 0);
            content_area.insert(MenuMouseTracker::default());

            content_area.with_children(|content| {
                // Create a container for both the content and scrollbar
                content
                    .spawn(Node {
                        width: Val::Percent(50.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Stretch,
                        ..default()
                    })
                    .with_children(|scrollable_container| {
                        // Scrollable list container
                        scrollable_container
                            .spawn(Node {
                                width: Val::Percent(90.0),
                                height: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::FlexStart,
                                justify_content: JustifyContent::Start,
                                overflow: Overflow::scroll_y(),
                                ..default()
                            })
                            .insert(ScrollableListContainer)
                            .insert(ScrollPosition::default())
                            .with_children(|map_list| {
                                for (idx, map) in maps.maps.iter().enumerate() {
                                    templates::create_content_item(map_list, map.name.clone(), idx, idx == 0, handles)
                                        .insert(MapSelectionItem { map_idx: idx });
                                }
                                map_list.spawn(Node {
                                    min_height: Val::Px(16.0),
                                    flex_basis: Val::Px(16.0),
                                    flex_shrink: 0.0,
                                    ..Default::default()
                                }).insert(PickingBehavior {
                                    should_block_lower: false,
                                    ..default()
                                });

                                templates::create_content_item(map_list, "Go Back", maps.maps.len(), false, handles);
                                map_list.spawn(Node {
                                    width: Val::Percent(100.0),
                                    min_height: Val::Px(64.0),
                                    flex_basis: Val::Px(64.0),
                                    flex_shrink: 0.0,
                                    ..Default::default()
                                }).insert(PickingBehavior {
                                    should_block_lower: false,
                                    ..default()
                                });
                            });

                        // Scrollbar container
                        scrollable_container
                            .spawn(Node {
                                width: Val::Px(48.0),
                                height: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Start,
                                justify_content: JustifyContent::Start,
                                padding: UiRect::vertical(Val::Px(4.0)),
                                ..default()
                            })
                            .insert(ScrollbarContainer)
                            .with_children(|scrollbar| {
                                // Up arrow
                                scrollbar
                                    .spawn(Button)
                                    .insert(ScrollbarUpArrow)
                                    .insert(Node {
                                        width: Val::Px(32.0),
                                        height: Val::Px(32.0),
                                        margin: UiRect::vertical(Val::Px(2.0)),
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    })
                                    .insert(Interaction::default())
                                    .with_children(|button| {
                                        button.spawn(ImageNode {
                                            image: handles.images.scroll_arrow_up.clone(),
                                            color: Color::WHITE,
                                            ..default()
                                        });
                                    });

                                // Track and thumb container
                                scrollbar
                                    .spawn(Node {
                                        width: Val::Px(32.0),
                                        height: Val::Percent(100.0),
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Center,
                                        flex_grow: 1.0,
                                        margin: UiRect::vertical(Val::Px(2.0)),
                                        ..default()
                                    })
                                    .with_children(|track_container| {
                                        // Track
                                        track_container
                                            .spawn(ImageNode {
                                                image: handles.images.scroll_track.clone(),
                                                color: Color::srgba(0.5, 0.5, 0.5, 0.4),
                                                ..default()
                                            })
                                            .insert(ScrollbarTrack)
                                            .insert(Node {
                                                width: Val::Px(32.0),
                                                height: Val::Percent(100.0),
                                                justify_content: JustifyContent::Center,
                                                align_content: AlignContent::Center,
                                                align_self: AlignSelf::Center,
                                                justify_self: JustifySelf::Center,
                                                ..default()
                                            });

                                        // Thumb
                                        track_container
                                            .spawn(Button)
                                            .insert(ScrollbarThumb)
                                            .insert(Node {
                                                width: Val::Px(32.0),
                                                height: Val::Px(48.0),
                                                position_type: PositionType::Absolute,
                                                top: Val::Percent(0.0),
                                                justify_content: JustifyContent::Center,
                                                ..default()
                                            })
                                            .insert(Interaction::default())
                                            .with_children(|thumb| {
                                                thumb.spawn(ImageNode {
                                                    image: handles.images.scroll_thumb.clone(),
                                                    color: Color::srgba(0.6, 0.6, 0.6, 0.4),
                                                    ..default()
                                                });
                                            });
                                    });

                                // Down arrow
                                scrollbar
                                    .spawn(Button)
                                    .insert(ScrollbarDownArrow)
                                    .insert(Node {
                                        width: Val::Px(32.0),
                                        height: Val::Px(32.0),
                                        margin: UiRect::vertical(Val::Px(2.0)),
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    })
                                    .insert(Interaction::default())
                                    .with_children(|button| {
                                        button.spawn(ImageNode {
                                            image: handles.images.scroll_arrow_down.clone(),
                                            color: Color::WHITE,
                                            ..default()
                                        });
                                    });
                            });
                    });

                content.spawn(Node {
                    width: Val::Percent(50.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                });
            });

            templates::create_help_text(
                parent,
                handles,
                Some("Select a map to play    |    [Up]/[Down]: Change    |    [Enter]: Select    |    [ESC]: Go Back".into()),
            );
        });
}

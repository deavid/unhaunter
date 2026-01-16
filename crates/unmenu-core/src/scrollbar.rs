use bevy::prelude::*;
use unui_core::assets::UiAssets;

// Component Definitions

/// Marker component for the scrollable container that holds the list content
#[derive(Component, Debug)]
pub struct ScrollableListContainer;

/// Component that identifies the scrollbar container
#[derive(Component, Debug)]
pub struct ScrollbarContainer;

/// Component that identifies the scrollbar track
#[derive(Component, Debug)]
pub struct ScrollbarTrack;

/// Component that identifies the scrollbar thumb (the draggable part)
#[derive(Component, Debug)]
pub struct ScrollbarThumb;

/// Component that identifies the scrollbar up arrow button
#[derive(Component, Debug)]
pub struct ScrollbarUpArrow;

/// Component that identifies the scrollbar down arrow button
#[derive(Component, Debug)]
pub struct ScrollbarDownArrow;

/// Builds the UI nodes for the scrollbar component.
///
/// This function should be called within a `with_children` closure
/// where the scrollbar is intended to be placed.
///
/// This is not a system but a helper function.
pub fn build_scrollbar_ui(scrollbar: &mut ChildSpawnerCommands, ui_assets: &UiAssets) {
    scrollbar
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
                        image: ui_assets.scroll_arrow_up.clone(),
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
                            image: ui_assets.scroll_track.clone(),
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
                        })
                        .insert(Pickable {
                            should_block_lower: false,
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
                                image: ui_assets.scroll_thumb.clone(),
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
                        image: ui_assets.scroll_arrow_down.clone(),
                        color: Color::WHITE,
                        ..default()
                    });
                });
        });
}

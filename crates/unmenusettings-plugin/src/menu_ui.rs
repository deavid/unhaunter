use crate::components::*;
use crate::menus::MenuSettingsLevel1;
use bevy::prelude::*;
use unmenu_core::assets::MenuAssets;
use unmenu_core::components::{MenuMouseTracker, MenuRoot};
use unmenu_core::templates;
use unorchestrator_core::UIContextState;

fn setup_ui_cam(mut commands: Commands) {
    commands.spawn(Camera2d).insert(SCamera);
}

fn setup_ui_main_cat_system(
    mut commands: Commands,
    menu_assets: Res<MenuAssets>,
    qtui: Query<Entity, With<SettingsMenu>>,
) {
    let menu_items = MenuSettingsLevel1::iter_events();
    setup_ui_main_cat(&mut commands, &menu_assets, &qtui, "Settings", &menu_items);
}

/// Helper function to set up the main categories UI for settings menu (not a system)
pub(crate) fn setup_ui_main_cat(
    commands: &mut Commands,
    menu_assets: &Res<MenuAssets>,
    qtui: &Query<Entity, With<SettingsMenu>>,
    title: impl Into<String>,
    menu_items: &[(String, MenuEvent)],
) {
    for e in qtui.iter() {
        commands.entity(e).despawn();
    }

    // Create new UI with unmenu_core templates
    let root_entity = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .insert(SettingsMenu {
            selected_item_idx: 0,
        })
        .with_children(|parent| {
            // Background
            templates::create_background(parent, menu_assets);

            // Logo
            templates::create_logo(parent, menu_assets);

            // Create breadcrumb navigation with title
            templates::create_breadcrumb_navigation(
                parent,
                menu_assets,
                title,
                "" // No subtitle for this level
            );

            // Create content area for settings items
            let mut content_area_entity = templates::create_selectable_content_area(
                parent,
                menu_assets,
                0 // Initial selection
            );

            // Add mouse tracker to prevent unwanted initial hover selection
            content_area_entity.insert(MenuMouseTracker::default());

            let content_area = content_area_entity.insert(MenuRoot {
                selected_item: 0,
            });

            // Add a column container inside the content area for vertical layout
            content_area.with_children(|content| {
                content
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        justify_content: JustifyContent::FlexStart,
                        overflow: Overflow::scroll_y(),
                        ..default()
                    })
                    .with_children(|menu_list| {
                        let mut idx = 0;

                        // Add each menu item
                        for (item_text, event) in menu_items.iter() {
                            if !event.is_none() {
                                templates::create_content_item(
                                    menu_list,
                                    item_text,
                                    idx,
                                    idx == 0, // First item selected by default
                                    menu_assets
                                )
                                .insert(MenuItem::new(idx, *event));
                                idx += 1;
                            } else {
                                // Add disabled item with gray color
                                templates::create_content_item_disabled(
                                    menu_list,
                                    item_text,
                                    menu_assets
                                );
                            }
                        }

                        // Add "Go Back" option
                        templates::create_content_item(
                            menu_list,
                            "Go Back",
                            idx,
                            false,
                            menu_assets
                        )
                        .insert(MenuItem::new(idx, MenuEvent::Back(MenuEvBack)));
                    });
            });

            // Help text
            templates::create_help_text(
                parent,
                menu_assets,
                Some("[Up]/[Down] arrows to navigate. Press [Enter] to select or [Escape] to go back".to_string())
            );
        })
        .id();

    debug!("Settings UI initialized with entity: {:?}", root_entity);
}

fn cleanup(
    mut commands: Commands,
    qtui: Query<Entity, With<SettingsMenu>>,
    qc: Query<Entity, With<SCamera>>,
    qtimer: Query<Entity, With<SettingsStateTimer>>,
) {
    // Despawn old camera if exists
    for cam in qc.iter() {
        commands.entity(cam).despawn();
    }

    // Clean up menu entities
    for e in qtui.iter() {
        commands.entity(e).despawn();
    }

    // Clean up timer
    for e in qtimer.iter() {
        commands.entity(e).despawn();
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        OnEnter(UIContextState::SettingsMenu),
        (
            setup_ui_cam,
            setup_ui_main_cat_system,
            |mut commands: Commands| {
                commands.spawn(SettingsStateTimer {
                    state_entered_at: bevy_platform::time::Instant::now(),
                });
            },
        )
            .chain(),
    )
    .add_systems(OnExit(UIContextState::SettingsMenu), cleanup);
}

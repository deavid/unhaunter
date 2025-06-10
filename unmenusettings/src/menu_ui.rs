use crate::components::*;
use crate::menus::MenuSettingsLevel1;
use bevy::prelude::*;
use uncore::states::AppState;
use uncore::types::root::game_assets::GameAssets;
use uncoremenu::components::{MenuMouseTracker, MenuRoot};
use uncoremenu::templates;

fn setup_ui_cam(mut commands: Commands) {
    commands.spawn(Camera2d).insert(SCamera);
}

fn setup_ui_main_cat_system(
    mut commands: Commands,
    handles: Res<GameAssets>,
    qtui: Query<Entity, With<SettingsMenu>>,
) {
    let menu_items = MenuSettingsLevel1::iter_events();
    setup_ui_main_cat(&mut commands, &handles, &qtui, "Settings", &menu_items);
}

/// Helper function to set up the main categories UI for settings menu (not a system)
pub(crate) fn setup_ui_main_cat(
    commands: &mut Commands,
    handles: &Res<GameAssets>,
    qtui: &Query<Entity, With<SettingsMenu>>,
    title: impl Into<String>,
    menu_items: &[(String, MenuEvent)],
) {
    for e in qtui.iter() {
        commands.entity(e).despawn();
    }

    // Create new UI with uncoremenu templates
    let root_entity = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .insert(SettingsMenu {
            menu_type: MenuType::MainCategories,
            selected_item_idx: 0,
        })
        .with_children(|parent| {
            // Background
            templates::create_background(parent, handles);

            // Logo
            templates::create_logo(parent, handles);

            // Create breadcrumb navigation with title
            templates::create_breadcrumb_navigation(
                parent,
                handles,
                title,
                "" // No subtitle for this level
            );

            // Create content area for settings items
            let mut content_area_entity = templates::create_selectable_content_area(
                parent,
                handles,
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
                                    handles
                                )
                                .insert(MenuItem::new(idx, *event));
                                idx += 1;
                            } else {
                                // Add disabled item with gray color
                                templates::create_content_item_disabled(
                                    menu_list,
                                    item_text,
                                    handles
                                );
                            }
                        }

                        // Add "Go Back" option
                        templates::create_content_item(
                            menu_list,
                            "Go Back",
                            idx,
                            false,
                            handles
                        )
                        .insert(MenuItem::new(idx, MenuEvent::Back(MenuEvBack)));
                    });
            });

            // Help text
            templates::create_help_text(
                parent,
                handles,
                Some("[Up]/[Down] arrows to navigate. Press [Enter] to select or [Escape] to go back".to_string())
            );
        })
        .id();

    info!("Settings UI initialized with entity: {:?}", root_entity);
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
        OnEnter(AppState::SettingsMenu),
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
    .add_systems(OnExit(AppState::SettingsMenu), cleanup);
}

use crate::components::*;
use crate::menus::MenuSettingsLevel1;
use bevy::prelude::*;
use uncore::colors;
use uncore::platform::plt::{FONT_SCALE, UI_SCALE};
use uncore::types::root::game_assets::GameAssets;

pub fn setup_ui_cam(mut commands: Commands) {
    commands.spawn(Camera2d).insert(SCamera);
}

pub fn setup_ui_main_cat_system(
    mut commands: Commands,
    handles: Res<GameAssets>,
    qtui: Query<Entity, With<SettingsMenu>>,
) {
    let menu_items = MenuSettingsLevel1::iter_events();
    setup_ui_main_cat(&mut commands, &handles, &qtui, "Settings", &menu_items);
}

pub fn setup_ui_main_cat(
    commands: &mut Commands,
    handles: &Res<GameAssets>,
    qtui: &Query<Entity, With<SettingsMenu>>,
    title: impl Into<String>,
    menu_items: &[(String, MenuEvent)],
) {
    // Clean up old UI:
    for e in qtui.iter() {
        commands.entity(e).despawn_recursive();
    }

    // Create new UI
    let mut menu_idx = 0;

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::Percent(10.0)),
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .insert(SettingsMenu {
            menu_type: MenuType::MainCategories,
            selected_item_idx: 0,
        })
        .with_children(|parent| {
            // Header
            parent
                .spawn(Text::new(title))
                .insert(TextFont {
                    font: handles.fonts.londrina.w300_light.clone(),
                    font_size: 38.0 * FONT_SCALE,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                })
                .insert(TextColor(Color::WHITE));

            parent.spawn(Node {
                flex_grow: 0.01,
                min_height: Val::Px(18.0 * UI_SCALE),
                ..default()
            });

            let create_menu_item =
                |parent: &mut ChildBuilder<'_>, title: &str, idx: &mut usize, menu_event: MenuEvent| {
                    let mut menu_item = parent.spawn((
                        Text::new(title),
                        TextFont {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 38.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        },
                        Node {
                            min_height: Val::Px(40.0 * UI_SCALE),
                            align_self: AlignSelf::Start,
                            justify_self: JustifySelf::Start,
                            padding: UiRect::left(Val::Px(15.0)),
                            ..default()
                        },
                    ));
                    if menu_event.is_none() {
                        menu_item.insert(
                            TextColor(colors::MENU_ITEM_COLOR_OFF.with_alpha(0.1)),
                        );
                    } else {
                        menu_item.insert(
                            (
                                MenuItem::new(*idx, menu_event),
                                TextColor(colors::MENU_ITEM_COLOR_OFF),
                            )
                        );
                        *idx += 1;
                    }
                };

            // Menu Items
            parent
                .spawn(Node {
                    width: Val::Percent(80.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::FlexStart,
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    ..default()
                })
                .with_children(|parent| {
                    for (item, event) in menu_items.iter() {
                        create_menu_item(parent, item, &mut menu_idx, *event);
                    }
                    parent.spawn(Node {
                        min_height: Val::Px(40.0 * UI_SCALE),
                        ..default()
                    });
                    create_menu_item(
                        parent,
                        "Go Back",
                        &mut menu_idx,
                        MenuEvent::Back(MenuEvBack),
                    );
                });
            parent
                .spawn(Text::new("[Up]/[Down] arrows to navigate. Press [Enter] to select or [Escape] to go back"))
                .insert(TextFont {
                    font: handles.fonts.chakra.w300_light.clone(),
                    font_size: 18.0 * FONT_SCALE,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                })
                .insert(TextColor(colors::TRUCKUI_TEXT_COLOR))
                .insert(Node {
                    align_self: AlignSelf::End,
                    justify_self: JustifySelf::End,
                    margin: UiRect::all(Val::Px(15.0 * UI_SCALE)),
                    ..default()
                });
        });

    info!("Settings UI initialized");
}

pub fn cleanup(
    mut commands: Commands,
    qtui: Query<Entity, With<SettingsMenu>>,
    qc: Query<Entity, With<SCamera>>,
) {
    // Despawn old camera if exists
    for cam in qc.iter() {
        commands.entity(cam).despawn_recursive();
    }

    for e in qtui.iter() {
        commands.entity(e).despawn_recursive();
    }
}

use crate::components::*;
use crate::menus::MenuSettingsLevel1;
use bevy::prelude::*;
use strum::IntoEnumIterator;
use uncore::colors;
use uncore::platform::plt::{FONT_SCALE, UI_SCALE};
use uncore::types::root::game_assets::GameAssets;

pub fn setup_ui(mut commands: Commands, handles: Res<GameAssets>) {
    let mut menu_idx = 0;
    commands.spawn(Camera2d).insert(SCamera);

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
            last_selected: default(),
            settings_entity: None,
        })
        .with_children(|parent| {
            // Header
            parent
                .spawn(Text::new("Settings"))
                .insert(TextFont {
                    font: handles.fonts.londrina.w300_light.clone(),
                    font_size: 38.0 * FONT_SCALE,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                })
                .insert(TextColor(Color::WHITE));

            parent.spawn(Node {
                flex_grow: 0.01,
                ..default()
            });

            let create_menu_item =
                |parent: &mut ChildBuilder<'_>, title, idx: &mut usize, menu_event: MenuEvent| {
                    parent.spawn((
                        Text::new(title),
                        TextFont {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 38.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        },
                        TextColor(colors::MENU_ITEM_COLOR_OFF),
                        MenuItem::new(*idx, menu_event),
                        Node {
                            min_height: Val::Px(40.0 * UI_SCALE),
                            align_self: AlignSelf::Start,
                            justify_self: JustifySelf::Start,
                            padding: UiRect::left(Val::Px(15.0)),
                            ..default()
                        },
                    ));
                    *idx += 1;
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
                    for item in MenuSettingsLevel1::iter() {
                        create_menu_item(parent, item.to_string(), &mut menu_idx, MenuEvent::None);
                    }
                    parent.spawn(Node {
                        min_height: Val::Px(40.0 * UI_SCALE),
                        ..default()
                    });
                    create_menu_item(
                        parent,
                        "Go Back".into(),
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

    info!("Settings UI initialized - Main menu only");
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

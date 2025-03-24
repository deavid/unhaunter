use super::gear_ui::{setup_ui_gear_inv_left, setup_ui_gear_inv_right};
use bevy::{color::palettes::css, prelude::*};
use bevy_persistent::Persistent;
use uncore::behavior::Behavior;
use uncore::colors;
use uncore::components::game_ui::{
    DamageBackground, ElementObjectUI, EvidenceUI, GameUI, RightSideGearUI, WalkieText,
};
use uncore::components::player_sprite::PlayerSprite;
use uncore::platform::plt::{FONT_SCALE, UI_SCALE};
use uncore::states::{AppState, GameState};
use uncore::types::root::game_assets::GameAssets;
use ungear::components::playergear::PlayerGear;
use unsettings::game::GameplaySettings;

pub fn cleanup(mut commands: Commands, qg: Query<Entity, With<GameUI>>) {
    // Despawn game UI if not used
    for gui in qg.iter() {
        commands.entity(gui).despawn_recursive();
    }
}

pub fn pause(mut qg: Query<&mut Visibility, With<GameUI>>) {
    for mut vis in qg.iter_mut() {
        *vis = Visibility::Hidden;
    }
}

pub fn resume(mut qg: Query<&mut Visibility, With<GameUI>>) {
    for mut vis in qg.iter_mut() {
        *vis = Visibility::Visible;
    }
}

pub fn setup_ui(
    mut commands: Commands,
    handles: Res<GameAssets>,
    game_settings: Res<Persistent<GameplaySettings>>,
) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .insert(ZIndex(-5))
        .insert(BackgroundColor(css::BLACK.with_alpha(0.0).into()))
        .insert(GameUI)
        .insert(DamageBackground::new(4.0));
    commands
        .spawn(ImageNode {
            image: handles.images.vignette.clone(),
            color: Color::NONE,
            ..default()
        })
        .insert(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .insert(ZIndex(-4))
        .insert(GameUI)
        .insert(DamageBackground::new(0.7));

    // Spawn game UI
    type Cb<'a, 'b> = &'b mut ChildBuilder<'a>;
    let key_legend = |p: Cb| {
        // For now a reminder of the keys:
        let ch_control = game_settings.character_controls.to_string();
        let controls = vec![
            format!("[{ch_control}]: Movement"),
            "[Shift]: Sprint".to_string(),
            "[Ctrl]: Left Hand".to_string(),
            "[E]: Interact".to_string(),
            "[F]: Grab/Move".to_string(),
            "[G]: Drop".to_string(),
            "[Q]: Next".to_string(),
            "[T]: Swap Hands".to_string(),
            "[C]: Change Evidence".to_string(),
        ];
        for ctrl in controls {
            p.spawn(Node {
                padding: UiRect::all(Val::Px(6.0 * UI_SCALE)),
                margin: UiRect::right(Val::Px(6.0 * UI_SCALE)),
                ..default()
            })
            .insert(BackgroundColor(css::BLACK.with_alpha(0.3).into()))
            .with_children(|p| {
                p.spawn(Text::new(ctrl))
                    .insert(TextFont {
                        font: handles.fonts.chakra.w300_light.clone(),
                        font_size: 16.0 * FONT_SCALE,
                        font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                    })
                    .insert(TextLayout::new_with_justify(JustifyText::Center))
                    .insert(TextColor(colors::INVENTORY_STATS_COLOR))
                    .insert(Node {
                        align_self: AlignSelf::Center,
                        justify_self: JustifySelf::Center,
                        justify_content: JustifyContent::Center,
                        margin: UiRect::bottom(Val::Px(-6.0 * UI_SCALE)),
                        padding: UiRect::all(Val::Px(4.0 * UI_SCALE)),
                        ..default()
                    });
            });
        }
    };
    let evidence = |p: Cb| setup_ui_evidence(p, &handles);
    let inv_left = |p: Cb| setup_ui_gear_inv_left(p, &handles);
    let inv_right = |p: Cb| setup_ui_gear_inv_right(p, &handles);
    let bottom_panel = |p: Cb| {
        // Left side
        // Split for the bottom side in three regions Leftmost side - Inventory left
        p.spawn(Node {
            min_width: Val::Px(100.0 * UI_SCALE),
            max_width: Val::Percent(33.3),
            // Horizontal alignment - start from the left.
            align_content: AlignContent::Start,
            flex_direction: FlexDirection::Row,
            border: UiRect::all(Val::Px(1.0 * UI_SCALE)),
            padding: UiRect::all(Val::Px(1.0)),
            flex_grow: 0.01,
            flex_shrink: 0.0,
            ..Default::default()
        })
        .insert(colors::DEBUG_BCOLOR)
        .insert(BackgroundColor(colors::PANEL_BGCOLOR))
        .with_children(inv_left);

        // Mid side
        p.spawn(Node {
            border: UiRect::all(Val::Px(1.0 * UI_SCALE)),
            padding: UiRect::all(Val::Px(8.0 * UI_SCALE)),
            flex_grow: 1.0,
            ..Default::default()
        })
        .insert(colors::DEBUG_BCOLOR)
        .insert(BackgroundColor(colors::PANEL_BGCOLOR))
        .with_children(evidence);

        // Right side
        p.spawn(Node {
            flex_direction: FlexDirection::Column,
            max_width: Val::Percent(33.3),
            align_items: AlignItems::Start,
            align_content: AlignContent::Center,
            border: UiRect::all(Val::Px(1.0)),
            padding: UiRect::all(Val::Px(1.0)),
            flex_grow: 0.01,
            ..Default::default()
        })
        .insert(colors::DEBUG_BCOLOR)
        .insert(BackgroundColor(colors::PANEL_BGCOLOR))
        .with_children(|p| {
            p.spawn(Node {
                align_items: AlignItems::Start,
                align_content: AlignContent::Center,
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(1.0)),
                flex_grow: 1.0,
                ..Default::default()
            })
            .insert(RightSideGearUI)
            .with_children(inv_right);
            // TODO: For now disabling the held object UI because it will clash with the looking left gear function.
            // p.spawn(Node {
            //     display: Display::None,
            //     border: UiRect::all(Val::Px(1.0)),
            //     padding: UiRect::all(Val::Px(1.0)),
            //     flex_direction: FlexDirection::Column,
            //     flex_grow: 1.0,
            //     ..Default::default()
            // })
            // .insert(colors::DEBUG_BCOLOR)
            // .insert(BackgroundColor(colors::PANEL_BGCOLOR))
            // .insert(HeldObjectUI)
            // .with_children(|parent| setup_ui_held_object(parent, &handles));
        });
    };
    let game_ui = |p: Cb| {
        // Top row (Game title)
        p.spawn(Node {
            height: Val::Percent(5.0),
            min_height: Val::Px(16.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            ..default()
        })
        .insert(colors::DEBUG_BCOLOR)
        .with_children(|parent| {
            // logo
            parent
                .spawn(ImageNode {
                    image: handles.images.title.clone(),
                    ..default()
                })
                .insert(Node {
                    aspect_ratio: Some(130.0 / 17.0),
                    width: Val::Percent(20.0),
                    height: Val::Auto,
                    max_width: Val::Percent(20.0),
                    max_height: Val::Percent(100.0),
                    flex_shrink: 1.0,
                    flex_grow: 0.0,
                    ..default()
                });
            parent
                .spawn(Text::new(""))
                .insert(TextFont {
                    font: handles.fonts.chakra.w400i_regular.clone(),
                    font_size: 18.0 * FONT_SCALE,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                })
                .insert(TextLayout::new_with_justify(JustifyText::Center))
                .insert(TextColor(colors::WALKIE_TALKIE_COLOR))
                .insert(Node {
                    align_self: AlignSelf::Center,
                    justify_self: JustifySelf::Center,
                    justify_content: JustifyContent::Center,
                    flex_grow: 1.0,
                    margin: UiRect::bottom(Val::Px(-6.0 * UI_SCALE)),
                    padding: UiRect::all(Val::Px(4.0 * UI_SCALE)),
                    ..default()
                })
                .insert(WalkieText);
        });

        // Main game viewport - middle
        p.spawn(Node {
            min_height: Val::Px(2.0),
            border: UiRect::all(Val::Px(1.0)),
            padding: UiRect::all(Val::Px(1.0)),
            flex_grow: 1.0,
            ..Default::default()
        })
        .insert(colors::DEBUG_BCOLOR);

        p.spawn(Node {
            align_content: AlignContent::Start,
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            border: UiRect::all(Val::Px(1.0 * UI_SCALE)),
            padding: UiRect::all(Val::Px(6.0 * UI_SCALE)), // .with_bottom(Val::Px(15.0 * UI_SCALE)),
            flex_grow: 0.0,
            ..Default::default()
        })
        .insert(BackgroundColor(colors::PANEL_BGCOLOR))
        .with_children(key_legend);

        // Bottom side - inventory and stats
        p.spawn(Node {
            height: Val::Px(100.0 * UI_SCALE),
            width: Val::Percent(99.9),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(6.0),
            border: UiRect::all(Val::Px(1.0)),
            padding: UiRect::all(Val::Px(1.0)),
            ..Default::default()
        })
        .insert(colors::DEBUG_BCOLOR)
        .insert(BackgroundColor(colors::PANEL_BGCOLOR))
        .with_children(bottom_panel);
    };

    // Build UI
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Column,
            border: UiRect::all(Val::Px(1.0)),
            padding: UiRect::all(Val::Px(1.0)),
            ..default()
        })
        .insert(colors::DEBUG_BCOLOR)
        .insert(GameUI)
        .with_children(game_ui);
    info!("Game UI loaded");
}

pub fn setup_ui_evidence(parent: &mut ChildBuilder, handles: &GameAssets) {
    parent
        .spawn((
            Text::default(),
            TextFont {
                font: handles.fonts.chakra.w400_regular.clone(),
                font_size: 22.0 * FONT_SCALE,
                font_smoothing: bevy::text::FontSmoothing::AntiAliased,
            },
            TextColor(colors::INVENTORY_STATS_COLOR.with_alpha(1.0)),
            TextLayout::default(),
            Node::default(),
            EvidenceUI,
        ))
        .with_children(|parent| {
            parent
                .spawn(TextSpan::new("Freezing temps:"))
                .insert(TextFont {
                    font: handles.fonts.chakra.w400_regular.clone(),
                    font_size: 22.0 * FONT_SCALE,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                 })
                .insert(TextColor(colors::INVENTORY_STATS_COLOR.with_alpha(1.0)));
            parent
                .spawn(TextSpan::new(" [+] Evidence Found\n"))
                .insert(TextFont {
                    font: handles.fonts.victormono.w600_semibold.clone(),
                    font_size: 20.0 * FONT_SCALE,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                })
                .insert(TextColor(css::GREEN.with_alpha(1.0).into()));
            parent
                .spawn(TextSpan::new(
                    "The ghost and the breach will make the ambient colder.\nSome ghosts will make the temperature drop below 0.0ºC.",
                ))
                .insert(TextFont {
                    font: handles.fonts.chakra.w300_light.clone(),
                    font_size: 20.0 * FONT_SCALE,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                 })
                .insert(TextColor(colors::INVENTORY_STATS_COLOR));
        });
}

/// Sets up the UI elements for displaying information about the map item being
/// held by the player.
fn _setup_ui_held_object(parent: &mut ChildBuilder, handles: &GameAssets) {
    parent
        .spawn(Text::new("Object Name"))
        .insert(TextFont {
            font: handles.fonts.victormono.w600_semibold.clone(),
            font_size: 20.0 * FONT_SCALE,
            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
        })
        .insert(TextColor(colors::INVENTORY_STATS_COLOR))
        .insert(ElementObjectUI::Name);

    // --- Object Description ---
    parent
        .spawn(Text::new("Object Description"))
        .insert(TextFont {
            font: handles.fonts.chakra.w300_light.clone(),
            font_size: 16.0 * FONT_SCALE,
            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
        })
        .insert(TextColor(colors::INVENTORY_STATS_COLOR))
        .insert(ElementObjectUI::Description);

    // --- Control Actions ---
    parent
        .spawn(Text::new("[Drop]: Drop Object\n[Grab]: Move Object"))
        .insert(TextFont {
            font: handles.fonts.chakra.w300_light.clone(),
            font_size: 16.0 * FONT_SCALE,
            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
        })
        .insert(TextColor(colors::INVENTORY_STATS_COLOR))
        .insert(ElementObjectUI::Grab);
}

/// Manages the UI for the "Visual Holding" system.
///
/// This system dynamically shows or hides the UI elements related to holding
/// objects.It displays the held object's name and provides instructions for
/// dropping or moving the object. When the player is not holding an object, the UI
/// reverts to displaying the player's gear information.
#[allow(clippy::type_complexity)]
pub fn toggle_held_object_ui(
    // mut held_object_ui: Query<
    //     (&mut Visibility, &mut Node),
    //     (With<HeldObjectUI>, Without<RightSideGearUI>),
    // >,
    // mut right_hand_ui: Query<
    //     (&mut Visibility, &mut Node),
    //     (With<RightSideGearUI>, Without<HeldObjectUI>),
    // >,
    mut text_query: Query<(&mut Text, &mut TextColor, &ElementObjectUI)>,
    players: Query<&PlayerGear, With<PlayerSprite>>,
    objects: Query<&Behavior>,
) {
    // let is_holding_object = players
    //     .iter()
    //     .any(|player_gear| player_gear.held_item.is_some());

    // TODO: For now I'm disabling the Held Object UI - it will clash with Shift to look at the left gear.
    // // --- Toggle Held Object UI ---
    // for (mut visibility, mut style) in held_object_ui.iter_mut() {
    //     *visibility = if is_holding_object {
    //         Visibility::Inherited
    //     } else {
    //         Visibility::Hidden
    //     };
    //     style.display = if is_holding_object {
    //         Display::Flex
    //     } else {
    //         Display::None
    //     };
    // }

    // // --- Toggle Right-Hand Gear UI ---
    // for (mut visibility, mut style) in right_hand_ui.iter_mut() {
    //     *visibility = if is_holding_object {
    //         Visibility::Hidden
    //     } else {
    //         Visibility::Inherited
    //     };
    //     style.display = if is_holding_object {
    //         Display::None
    //     } else {
    //         Display::Flex
    //     };
    // }

    // --- Retrieve Object Data ---
    if let Ok(player_gear) = players.get_single() {
        if let Some(held_object) = &player_gear.held_item {
            if let Ok(behavior) = objects.get(held_object.entity) {
                // --- Set Object Name ---
                for (mut text, _, _) in text_query
                    .iter_mut()
                    .filter(|(_, _, e)| **e == ElementObjectUI::Name)
                {
                    text.0.clone_from(&behavior.p.object.name);
                }

                // --- Set Object Description ---
                for (mut text, _, _) in text_query
                    .iter_mut()
                    .filter(|(_, _, e)| **e == ElementObjectUI::Description)
                {
                    text.0 = "Object Description".into();
                }

                // --- Dynamic "Move" Action ---
                for (mut text, mut color, _) in text_query
                    .iter_mut()
                    .filter(|(_, _, e)| **e == ElementObjectUI::Grab)
                {
                    if behavior.p.object.movable {
                        text.0 = "[Grab]: Move Object".into();
                        color.0 = colors::INVENTORY_STATS_COLOR;
                    } else {
                        text.0 = "[Grab]: -".into();
                        color.0 = colors::INVENTORY_STATS_COLOR.with_alpha(0.3);
                    }
                }
            }
        }
    }
}

pub fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(AppState::InGame), setup_ui)
        .add_systems(OnExit(AppState::InGame), cleanup)
        .add_systems(OnEnter(GameState::None), resume)
        .add_systems(OnExit(GameState::None), pause)
        .add_systems(
            Update,
            toggle_held_object_ui.run_if(in_state(GameState::None)),
        );
}

use bevy::{prelude::*, render::view::RenderLayers};

use crate::{
    behavior::Behavior,
    colors,
    game::evidence,
    gear::{self, playergear::PlayerGear},
    platform::plt::UI_SCALE,
    player::PlayerSprite,
    root,
};

#[derive(Component)]
pub struct GCameraUI;

#[derive(Component, Debug)]
pub struct GameUI;

#[derive(Component, Debug, PartialEq, Eq)]
pub enum ElementObjectUI {
    Name,
    Description,
    Grab,
}

#[derive(Component, Debug)]
pub struct DamageBackground {
    pub exp: f32,
}

impl DamageBackground {
    pub fn new(exp: f32) -> Self {
        Self { exp }
    }
}

#[derive(Component, Debug)]
pub struct HeldObjectUI;

#[derive(Component, Debug)]
pub struct RightSideGearUI;

pub fn setup(mut commands: Commands, qc2: Query<Entity, With<GCameraUI>>) {
    // Despawn old camera if exists
    for cam in qc2.iter() {
        commands.entity(cam).despawn_recursive();
    }
    // 2D orthographic camera - UI
    let cam = Camera2dBundle {
        camera_2d: Camera2d,
        camera: Camera {
            // renders after / on top of the main camera
            order: 1,
            ..default()
        },
        ..default()
    };
    commands
        .spawn(cam)
        .insert(GCameraUI)
        .insert(RenderLayers::from_layers(&[2, 3]));
}

pub fn cleanup(
    mut commands: Commands,
    qc2: Query<Entity, With<GCameraUI>>,
    qg: Query<Entity, With<GameUI>>,
) {
    // Despawn old camera if exists
    for cam in qc2.iter() {
        commands.entity(cam).despawn_recursive();
    }
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

pub fn setup_ui(mut commands: Commands, handles: Res<root::GameAssets>) {
    commands
        .spawn(NodeBundle {
            background_color: Color::NONE.into(),
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        })
        .insert(GameUI)
        .insert(DamageBackground::new(4.0));
    commands
        .spawn(ImageBundle {
            background_color: Color::BLACK.into(),
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            image: handles.images.vignette.clone().into(),
            ..default()
        })
        .insert(GameUI)
        .insert(DamageBackground::new(0.7));
    // Spawn game UI
    type Cb<'a, 'b> = &'b mut ChildBuilder<'a>;

    let key_legend = |p: Cb| {
        // For now a reminder of the keys:
        let text_bundle = TextBundle::from_section(
            "[WASD]: Movement\n[E]: Interact\n[F]: Grab/Move\n[G]: Drop",
            TextStyle {
                font: handles.fonts.chakra.w300_light.clone(),
                font_size: 16.0 * UI_SCALE,
                color: colors::INVENTORY_STATS_COLOR,
            },
        )
        .with_style(Style {
            // height: Val::Percent(100.0),
            align_self: AlignSelf::End,
            justify_self: JustifySelf::End,
            justify_content: JustifyContent::End,
            margin: UiRect::new(
                Val::Px(4.0 * UI_SCALE),
                Val::Px(4.0 * UI_SCALE),
                Val::Px(5.0 * UI_SCALE),
                Val::Px(-5.0 * UI_SCALE),
            ),
            ..default()
        });

        p.spawn(text_bundle);
    };

    let evidence = |p: Cb| evidence::setup_ui_evidence(p, &handles);

    let inv_left = |p: Cb| gear::ui::setup_ui_gear_inv_left(p, &handles);
    let inv_right = |p: Cb| gear::ui::setup_ui_gear_inv_right(p, &handles);

    let bottom_panel = |p: Cb| {
        // Split for the bottom side in three regions

        // Leftmost side - Inventory left
        p.spawn(NodeBundle {
            border_color: colors::DEBUG_BCOLOR,
            background_color: BackgroundColor(colors::PANEL_BGCOLOR),
            style: Style {
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(1.0)),
                flex_grow: 0.0,
                flex_shrink: 0.0,
                width: Val::Px(100.0 * UI_SCALE),
                max_width: Val::Percent(20.0),
                align_items: AlignItems::Center, // Vertical alignment
                align_content: AlignContent::Start, // Horizontal alignment - start from the left.
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(inv_left);

        // Left side
        p.spawn(NodeBundle {
            border_color: colors::DEBUG_BCOLOR,
            background_color: BackgroundColor(colors::PANEL_BGCOLOR),
            style: Style {
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(6.0 * UI_SCALE)),
                flex_grow: 0.0,
                min_width: Val::Px(200.0 * UI_SCALE),
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(key_legend);

        // Mid side
        p.spawn(NodeBundle {
            border_color: colors::DEBUG_BCOLOR,
            background_color: BackgroundColor(colors::PANEL_BGCOLOR),
            style: Style {
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(8.0 * UI_SCALE)),
                flex_grow: 1.0,
                max_width: Val::Percent(60.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(evidence);

        // Right side
        p.spawn(NodeBundle {
            border_color: colors::DEBUG_BCOLOR,
            background_color: BackgroundColor(colors::PANEL_BGCOLOR),
            style: Style {
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(1.0)),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                max_width: Val::Percent(33.3),
                align_items: AlignItems::Start,
                align_content: AlignContent::Center,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|p| {
            p.spawn(NodeBundle {
                border_color: colors::DEBUG_BCOLOR,
                style: Style {
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::all(Val::Px(1.0)),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    align_content: AlignContent::Center,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(RightSideGearUI)
            .with_children(inv_right);

            p.spawn(NodeBundle {
                border_color: colors::DEBUG_BCOLOR,
                style: Style {
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::all(Val::Px(1.0)),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    display: Display::None, // Initially hidden
                    ..Default::default()
                },
                visibility: Visibility::Hidden, // Initially hidden
                ..Default::default()
            })
            .insert(HeldObjectUI)
            .with_children(|parent| setup_ui_held_object(parent, &handles));
        });
    };

    let game_ui = |p: Cb| {
        // Top row (Game title)
        p.spawn(NodeBundle {
            border_color: colors::DEBUG_BCOLOR,

            style: Style {
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(1.0)),
                width: Val::Percent(20.0),
                height: Val::Percent(5.0),
                min_width: Val::Px(0.0),
                min_height: Val::Px(16.0),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // logo
            parent.spawn(ImageBundle {
                style: Style {
                    aspect_ratio: Some(130.0 / 17.0),
                    width: Val::Percent(80.0),
                    height: Val::Auto,
                    max_width: Val::Percent(80.0),
                    max_height: Val::Percent(100.0),
                    flex_shrink: 1.0,
                    ..default()
                },
                image: handles.images.title.clone().into(),
                ..default()
            });
        });

        // Main game viewport - middle
        p.spawn(NodeBundle {
            border_color: colors::DEBUG_BCOLOR,
            style: Style {
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(1.0)),
                flex_grow: 1.0,
                min_height: Val::Px(2.0),
                ..Default::default()
            },
            ..Default::default()
        });

        // Bottom side - inventory and stats
        p.spawn(NodeBundle {
            border_color: colors::DEBUG_BCOLOR,
            background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4)),
            style: Style {
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(1.0)),
                height: Val::Px(100.0 * UI_SCALE),
                width: Val::Percent(99.9),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(6.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(bottom_panel);
    };

    // Build UI
    commands
        .spawn(NodeBundle {
            border_color: colors::DEBUG_BCOLOR,

            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            ..default()
        })
        .insert(GameUI)
        .with_children(game_ui);

    info!("Game UI loaded");
}

/// Sets up the UI elements for displaying information about the map item being held by the player.
fn setup_ui_held_object(parent: &mut ChildBuilder, handles: &root::GameAssets) {
    parent
        .spawn(TextBundle::from_section(
            "Object Name",
            TextStyle {
                font: handles.fonts.victormono.w600_semibold.clone(),
                font_size: 20.0 * UI_SCALE,
                color: colors::INVENTORY_STATS_COLOR,
            },
        ))
        .insert(ElementObjectUI::Name);

    // --- Object Description ---
    parent
        .spawn(TextBundle::from_section(
            "Object Description",
            TextStyle {
                font: handles.fonts.chakra.w300_light.clone(),
                font_size: 16.0 * UI_SCALE,
                color: colors::INVENTORY_STATS_COLOR,
            },
        ))
        .insert(ElementObjectUI::Description);

    // --- Control Actions ---
    parent
        .spawn(TextBundle::from_sections([
            TextSection {
                value: "[Drop]: Drop Object\n".into(),
                style: TextStyle {
                    font: handles.fonts.chakra.w300_light.clone(),
                    font_size: 16.0 * UI_SCALE,
                    color: colors::INVENTORY_STATS_COLOR,
                },
            },
            TextSection {
                value: "[Grab]: Move Object".into(), // Placeholder (will be dynamic)
                style: TextStyle {
                    font: handles.fonts.chakra.w300_light.clone(),
                    font_size: 16.0 * UI_SCALE,
                    color: colors::INVENTORY_STATS_COLOR,
                },
            },
        ]))
        .insert(ElementObjectUI::Grab);
}

/// Manages the UI for the "Visual Holding" system.
///
/// This system dynamically shows or hides the UI elements related to holding objects.  
/// It displays the held object's name and provides instructions for dropping or moving the object.
/// When the player is not holding an object, the UI reverts to displaying the player's gear information.
#[allow(clippy::type_complexity)]
pub fn toggle_held_object_ui(
    mut held_object_ui: Query<
        (&mut Visibility, &mut Style),
        (With<HeldObjectUI>, Without<RightSideGearUI>),
    >,
    mut right_hand_ui: Query<
        (&mut Visibility, &mut Style),
        (With<RightSideGearUI>, Without<HeldObjectUI>),
    >,
    mut text_query: Query<(&mut Text, &ElementObjectUI)>,
    players: Query<&PlayerGear, With<PlayerSprite>>,
    objects: Query<&Behavior>,
) {
    let is_holding_object = players
        .iter()
        .any(|player_gear| player_gear.held_item.is_some());

    // --- Toggle Held Object UI ---
    for (mut visibility, mut style) in held_object_ui.iter_mut() {
        *visibility = if is_holding_object {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };

        style.display = if is_holding_object {
            Display::Flex
        } else {
            Display::None
        };
    }

    // --- Toggle Right-Hand Gear UI ---
    for (mut visibility, mut style) in right_hand_ui.iter_mut() {
        *visibility = if is_holding_object {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        style.display = if is_holding_object {
            Display::None
        } else {
            Display::Flex
        };
    }

    // --- Retrieve Object Data ---
    if let Ok(player_gear) = players.get_single() {
        if let Some(held_object) = &player_gear.held_item {
            if let Ok(behavior) = objects.get(held_object.entity) {
                // --- Set Object Name ---
                for (mut text, _) in text_query
                    .iter_mut()
                    .filter(|(_, e)| **e == ElementObjectUI::Name)
                {
                    text.sections[0].value.clone_from(&behavior.p.object.name);
                }

                // --- Set Object Description ---
                for (mut text, _) in text_query
                    .iter_mut()
                    .filter(|(_, e)| **e == ElementObjectUI::Description)
                {
                    text.sections[0].value = "Object Description".into();
                }

                // --- Dynamic "Move" Action ---
                for (mut text, _) in text_query
                    .iter_mut()
                    .filter(|(_, e)| **e == ElementObjectUI::Grab)
                {
                    if behavior.p.object.movable {
                        text.sections[0].value = "[Grab]: Move Object".into();
                        text.sections[0].style.color = colors::INVENTORY_STATS_COLOR;
                    } else {
                        text.sections[0].value = "[Grab]: -".into();
                        text.sections[0].style.color = colors::INVENTORY_STATS_COLOR.with_alpha(0.3);
                    }
                }
            }
        }
    }
}

pub fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(root::State::InGame), setup)
        .add_systems(OnEnter(root::State::InGame), setup_ui)
        .add_systems(OnExit(root::State::InGame), cleanup)
        .add_systems(OnEnter(root::GameState::None), resume)
        .add_systems(OnExit(root::GameState::None), pause)
        .add_systems(
            Update,
            toggle_held_object_ui.run_if(in_state(root::GameState::None)),
        );
}

use bevy::{prelude::*, render::view::RenderLayers};

use crate::{gear, root};

#[derive(Component)]
pub struct GCameraUI;

#[derive(Component, Debug)]
pub struct GameUI;

#[derive(Component, Debug)]
pub struct DamageBackground;

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
    const DEBUG_BCOLOR: BorderColor = BorderColor(Color::rgba(0.0, 1.0, 1.0, 0.0003));
    const INVENTORY_STATS_COLOR: Color = Color::rgba(0.7, 0.7, 0.7, 0.6);
    const PANEL_BGCOLOR: Color = Color::rgba(0.1, 0.1, 0.1, 0.3);
    commands
        .spawn(NodeBundle {
            background_color: Color::rgba(0.0, 0.05, 0.2, 0.3).into(),
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        })
        .insert(GameUI)
        .insert(DamageBackground);
    // Spawn game UI

    let key_legend = |parent: &mut ChildBuilder| {
        // For now a reminder of the keys:
        let text_bundle = TextBundle::from_section(
            "Movement: WASD - Interact: E\nToggle Aux: T - Toggle Main: R\nCycle Inv: Q - Swap: TAB\nChange Evidence: C",
            TextStyle {
                font: handles.fonts.chakra.w300_light.clone(),
                font_size: 18.0,
                color: INVENTORY_STATS_COLOR,
            },
        );

        parent.spawn(text_bundle);
    };

    let evidence = |parent: &mut ChildBuilder| {
        let text_bundle = TextBundle::from_sections([
            TextSection{value: "Freezing temps:".into(), 
                style: TextStyle {
                    font: handles.fonts.chakra.w400_regular.clone(),
                    font_size: 22.0,
                    color: INVENTORY_STATS_COLOR.with_a(1.0),
                },
            },
            TextSection{value: " [+] Evidence Found\n".into(), 
                style: TextStyle {
                    font: handles.fonts.victormono.w600_semibold.clone(),
                    font_size: 20.0,
                    color: Color::GREEN.with_a(0.4),
                },
            },
            TextSection{value: "The ghost and the breach will make the ambient colder.\nSome ghosts will make the temperature drop below 0.0ºC.".into(), 
                style: TextStyle {
                    font: handles.fonts.chakra.w300_light.clone(),
                    font_size: 20.0,
                    color: INVENTORY_STATS_COLOR,
                },
            },
        ]);
        parent.spawn(text_bundle);
    };

    let inventory = |parent: &mut ChildBuilder| {
        // Right side panel - inventory
        parent
            .spawn(AtlasImageBundle {
                image: UiImage {
                    texture: handles.images.gear.clone(),
                    flip_x: false,
                    flip_y: false,
                },
                texture_atlas: TextureAtlas {
                    index: gear::GearSpriteID::Flashlight2 as usize,
                    layout: handles.images.gear_atlas.clone(),
                },
                ..default()
            })
            .insert(gear::playergear::Inventory::new_left());
        parent
            .spawn(AtlasImageBundle {
                image: UiImage {
                    texture: handles.images.gear.clone(),
                    flip_x: false,
                    flip_y: false,
                },
                texture_atlas: TextureAtlas {
                    index: gear::GearSpriteID::IonMeter2 as usize,
                    layout: handles.images.gear_atlas.clone(),
                },
                ..default()
            })
            .insert(gear::playergear::Inventory::new_right());
        let mut text_bundle = TextBundle::from_section(
            "-",
            TextStyle {
                font: handles.fonts.victormono.w600_semibold.clone(),
                font_size: 20.0,
                color: INVENTORY_STATS_COLOR,
            },
        );
        text_bundle.style = Style {
            flex_grow: 1.0,
            ..Default::default()
        };
        parent
            .spawn(text_bundle)
            .insert(gear::playergear::InventoryStats);
    };

    let bottom_panel = |parent: &mut ChildBuilder| {
        // Split for the bottom side in three regions

        // Left side
        parent
            .spawn(NodeBundle {
                border_color: DEBUG_BCOLOR,
                background_color: BackgroundColor(PANEL_BGCOLOR),
                style: Style {
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::all(Val::Px(6.0)),
                    flex_grow: 0.0,
                    min_width: Val::Px(200.0),
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                ..Default::default()
            })
            .with_children(key_legend);

        // Mid side
        parent
            .spawn(NodeBundle {
                border_color: DEBUG_BCOLOR,
                background_color: BackgroundColor(PANEL_BGCOLOR),
                style: Style {
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::all(Val::Px(8.0)),
                    flex_grow: 1.0,
                    max_width: Val::Percent(60.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .with_children(evidence);

        // Right side
        parent
            .spawn(NodeBundle {
                border_color: DEBUG_BCOLOR,
                background_color: BackgroundColor(PANEL_BGCOLOR),
                style: Style {
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::all(Val::Px(1.0)),
                    flex_grow: 1.0,
                    max_width: Val::Percent(33.3),
                    align_items: AlignItems::Center, // Vertical alignment
                    align_content: AlignContent::Start, // Horizontal alignment - start from the left.
                    ..Default::default()
                },
                ..Default::default()
            })
            .with_children(inventory);
    };

    let game_ui = |parent: &mut ChildBuilder| {
        // Top row (Game title)
        parent
            .spawn(NodeBundle {
                border_color: DEBUG_BCOLOR,

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
        parent.spawn(NodeBundle {
            border_color: DEBUG_BCOLOR,
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
        parent
            .spawn(NodeBundle {
                border_color: DEBUG_BCOLOR,
                background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.4)),
                style: Style {
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::all(Val::Px(1.0)),
                    height: Val::Px(100.0),
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
            border_color: DEBUG_BCOLOR,

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

pub fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(root::State::InGame), setup)
        .add_systems(OnEnter(root::State::InGame), setup_ui)
        .add_systems(OnExit(root::State::InGame), cleanup)
        .add_systems(OnEnter(root::GameState::None), resume)
        .add_systems(OnExit(root::GameState::None), pause);
}

use bevy::prelude::*;

use crate::{
    behavior::component::NpcHelpDialog,
    colors,
    materials::{self, UIPanelMaterial},
    platform::plt::UI_SCALE,
    root,
};

#[derive(Debug, Component)]
pub struct NpcUI;

#[derive(Debug, Component)]
pub struct NpcDialogText;

#[derive(Debug, Resource, Default)]
pub struct NpcUIData {
    pub dialog: String,
}

#[derive(Clone, Debug, Event)]
pub struct NpcHelpEvent {
    pub entity: Entity,
}

impl NpcHelpEvent {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}

pub fn keyboard(
    game_state: Res<State<root::GameState>>,
    mut game_next_state: ResMut<NextState<root::GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if *game_state.get() != root::GameState::NpcHelp {
        return;
    }
    if keyboard_input.just_pressed(KeyCode::Escape) || keyboard_input.just_pressed(KeyCode::KeyE) {
        game_next_state.set(root::GameState::None);
    }
}

pub fn cleanup(mut commands: Commands, qtui: Query<Entity, With<NpcUI>>) {
    for e in qtui.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn setup_ui(
    mut commands: Commands,
    mut materials: ResMut<Assets<materials::UIPanelMaterial>>,
    handles: Res<root::GameAssets>,
    npcdata: Res<NpcUIData>,
) {
    const MARGIN_PERCENT: f32 = 0.5;
    const MARGIN: UiRect = UiRect::percent(
        MARGIN_PERCENT,
        MARGIN_PERCENT,
        MARGIN_PERCENT,
        MARGIN_PERCENT,
    );
    commands
        .spawn(NodeBundle {
            background_color: colors::TRUCKUI_BGCOLOR.into(),

            style: Style {
                position_type: PositionType::Absolute,
                min_width: Val::Percent(50.0),
                min_height: Val::Percent(30.0),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Percent(MARGIN_PERCENT),
                padding: MARGIN,
                margin: MARGIN,
                ..default()
            },
            ..default()
        })
        .insert(NpcUI)
        .with_children(|parent| {
            // Mid content
            parent
                .spawn(MaterialNodeBundle {
                    material: materials.add(UIPanelMaterial {
                        color: colors::TRUCKUI_PANEL_BGCOLOR,
                    }),

                    style: Style {
                        border: UiRect::all(Val::Px(1.0)),
                        padding: UiRect::all(Val::Px(1.0)),
                        min_width: Val::Px(10.0),
                        min_height: Val::Px(10.0),
                        justify_content: JustifyContent::FlexStart,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Percent(MARGIN_PERCENT),
                        flex_grow: 1.0,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|mid_blk| {
                    let title = TextBundle::from_section(
                        "Stranger says:",
                        TextStyle {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 35.0 * UI_SCALE,
                            color: colors::TRUCKUI_ACCENT_COLOR,
                        },
                    )
                    .with_style(Style {
                        height: Val::Px(40.0 * UI_SCALE),
                        ..default()
                    });

                    mid_blk.spawn(title);

                    mid_blk.spawn(NodeBundle {
                        border_color: colors::TRUCKUI_ACCENT_COLOR.into(),
                        style: Style {
                            border: UiRect::top(Val::Px(1.50)),
                            height: Val::Px(0.0),
                            ..default()
                        },
                        ..default()
                    });

                    mid_blk
                        .spawn(
                            TextBundle::from_section(
                                npcdata.dialog.clone(),
                                TextStyle {
                                    font: handles.fonts.syne.w400_regular.clone(),
                                    font_size: 21.0 * UI_SCALE,
                                    color: colors::DIALOG_TEXT_COLOR,
                                },
                            )
                            .with_style(Style {
                                margin: UiRect::all(Val::Px(8.0 * UI_SCALE)),
                                max_width: Val::Vw(50.0),
                                ..default()
                            }),
                        )
                        .insert(NpcDialogText);

                    mid_blk.spawn(NodeBundle {
                        style: Style {
                            flex_grow: 1.0,
                            ..default()
                        },
                        ..default()
                    });

                    mid_blk.spawn(
                        TextBundle::from_section(
                            "Close: [ESC] or [E]",
                            TextStyle {
                                font: handles.fonts.chakra.w300_light.clone(),
                                font_size: 25.0 * UI_SCALE,
                                color: colors::TRUCKUI_TEXT_COLOR,
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(4.0)),
                            align_content: AlignContent::End,
                            align_items: AlignItems::End,
                            align_self: AlignSelf::End,
                            justify_content: JustifyContent::End,
                            justify_items: JustifyItems::End,
                            justify_self: JustifySelf::End,
                            ..default()
                        })
                        .with_text_justify(JustifyText::Right),
                    );

                    // ----
                    mid_blk.spawn(NodeBundle {
                        style: Style {
                            justify_content: JustifyContent::FlexStart,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Percent(MARGIN_PERCENT),
                            flex_grow: 1.0,
                            ..default()
                        },
                        ..default()
                    });
                });
        });

    // ---
}

pub fn npchelp_event(
    mut ev_npc: EventReader<NpcHelpEvent>,
    npc: Query<(Entity, &NpcHelpDialog)>,
    mut res_npc: ResMut<NpcUIData>,
    mut game_next_state: ResMut<NextState<root::GameState>>,
) {
    let Some(ev_npc) = ev_npc.read().next() else {
        return;
    };
    let Some(npcd) = npc
        .iter()
        .find(|(e, _)| *e == ev_npc.entity)
        .map(|(_, n)| n)
    else {
        warn!("Wrong entity for npchelp_event?");
        return;
    };
    res_npc.dialog = npcd.dialog.clone();
    game_next_state.set(root::GameState::NpcHelp);
    // warn!(npcd.dialog);
}

pub fn app_setup(app: &mut App) {
    app.add_event::<NpcHelpEvent>()
        .init_resource::<NpcUIData>()
        .add_systems(Update, npchelp_event)
        .add_systems(OnEnter(root::GameState::NpcHelp), setup_ui)
        .add_systems(OnExit(root::GameState::NpcHelp), cleanup)
        .add_systems(Update, keyboard);
}

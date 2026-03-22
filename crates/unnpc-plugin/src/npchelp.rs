use bevy::prelude::*;
use unbehavior_core::behavior::Behavior;
use unbehavior_core::behavior::{Interactive, NpcHelpDialog};
use unfoundation_core::colors;
use unfoundation_core::platform::plt::{FONT_SCALE, UI_SCALE};
use unnpc_core::events::NpcHelpEvent;
use unplayer_core::components::MainPlayer;
use unrender_std::materials::UIPanelMaterial;
use unspatial_core::direction::Direction;
use unspatial_core::position::Position;
use untypes_core::states::GameState;
use unui_core::assets::UiAssets;

#[derive(Debug, Component)]
pub(crate) struct NpcUI;
#[derive(Debug, Component)]
pub(crate) struct NpcDialogText;

#[derive(Debug, Resource, Default)]
pub(crate) struct NpcUIData {
    pub dialog: String,
}

pub(crate) fn keyboard(
    game_state: Res<State<GameState>>,
    mut game_next_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if *game_state.get() != GameState::NpcHelp {
        return;
    }
    if keyboard_input.just_pressed(KeyCode::Escape) || keyboard_input.just_pressed(KeyCode::KeyE) {
        game_next_state.set(GameState::Running);
    }
}

pub(crate) fn cleanup(mut commands: Commands, qtui: Query<Entity, With<NpcUI>>) {
    for e in qtui.iter() {
        commands.entity(e).despawn();
    }
}

pub(crate) fn setup_ui(
    mut commands: Commands,
    mut materials: ResMut<Assets<UIPanelMaterial>>,
    ui_assets: Res<UiAssets>,
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
        .spawn(Node {
            position_type: PositionType::Absolute,
            min_width: Val::Percent(50.0),
            min_height: Val::Percent(30.0),
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Row,
            column_gap: Val::Percent(MARGIN_PERCENT),
            padding: MARGIN,
            margin: MARGIN,
            ..default()
        })
        .insert(BackgroundColor(colors::TRUCKUI_BGCOLOR))
        .insert(NpcUI)
        .with_children(|parent| {
            // Mid content
            parent
                .spawn(MaterialNode(materials.add(UIPanelMaterial {
                    color: colors::TRUCKUI_PANEL_BGCOLOR.into(),
                })))
                .insert(Node {
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::all(Val::Px(1.0)),
                    min_width: Val::Px(10.0),
                    min_height: Val::Px(10.0),
                    justify_content: JustifyContent::FlexStart,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Percent(MARGIN_PERCENT),
                    flex_grow: 1.0,
                    ..default()
                })
                .with_children(|mid_blk| {
                    mid_blk
                        .spawn(Text::new("Stranger says:"))
                        .insert(TextFont {
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 35.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(colors::TRUCKUI_ACCENT_COLOR))
                        .insert(Node {
                            height: Val::Px(40.0 * UI_SCALE),
                            ..default()
                        });
                    mid_blk
                        .spawn(Node {
                            border: UiRect::top(Val::Px(1.50)),
                            height: Val::Px(0.0),
                            ..default()
                        })
                        .insert(BorderColor::all(colors::TRUCKUI_ACCENT_COLOR));
                    mid_blk
                        .spawn(Text::new(npcdata.dialog.clone()))
                        .insert(TextFont {
                            font: ui_assets.font_syne_regular.clone(),
                            font_size: 21.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(colors::DIALOG_TEXT_COLOR))
                        .insert(Node {
                            margin: UiRect::all(Val::Px(8.0 * UI_SCALE)),
                            max_width: Val::Vw(50.0),
                            ..default()
                        })
                        .insert(NpcDialogText);
                    mid_blk.spawn(Node {
                        flex_grow: 1.0,
                        ..default()
                    });
                    mid_blk
                        .spawn(Text::new("Close: [ESC] or [E]"))
                        .insert(TextFont {
                            font: ui_assets.font_chakra_light.clone(),
                            font_size: 25.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(colors::TRUCKUI_TEXT_COLOR))
                        .insert(Node {
                            margin: UiRect::all(Val::Px(4.0)),
                            align_content: AlignContent::End,
                            align_items: AlignItems::End,
                            align_self: AlignSelf::End,
                            justify_content: JustifyContent::End,
                            justify_items: JustifyItems::End,
                            justify_self: JustifySelf::End,
                            ..default()
                        });

                    // ---
                    mid_blk.spawn(Node {
                        justify_content: JustifyContent::FlexStart,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Percent(MARGIN_PERCENT),
                        flex_grow: 1.0,
                        ..default()
                    });
                });
        });
    // ---
}

pub(crate) fn npchelp_event(
    mut ev_npc: MessageReader<NpcHelpEvent>,
    mut npc: Query<(Entity, &mut NpcHelpDialog)>,
    mut res_npc: ResMut<NpcUIData>,
    mut game_next_state: ResMut<NextState<GameState>>,
) {
    let Some(ev_npc) = ev_npc.read().next() else {
        return;
    };
    let Some(mut npcd) = npc
        .iter_mut()
        .find(|(e, _)| *e == ev_npc.entity)
        .map(|(_, n)| n)
    else {
        warn!("Wrong entity for npchelp_event?");
        return;
    };
    npcd.seen = true;
    res_npc.dialog.clone_from(&npcd.dialog);
    game_next_state.set(GameState::NpcHelp);
    // warn!(npcd.dialog);
}

/// NPCs will call the player by distance & time if haven't spoken yet.
pub(crate) fn auto_call_npchelp(
    time: Res<Time>,
    q_player: Query<(&Position, &Direction), With<MainPlayer>>,
    mut interactables: Query<(
        Entity,
        &Position,
        &Interactive,
        &Behavior,
        &mut NpcHelpDialog,
    )>,
    mut ev_npc: MessageWriter<NpcHelpEvent>,
) {
    let Ok((pos, dir)) = q_player.single() else {
        return;
    };
    if dir.distance() > 79.5 {
        // If the player is walking fast, do not trigger auto-help.
        return;
    }
    let dt = time.delta_secs();
    for (entity, item_pos, _, _, mut npc) in interactables.iter_mut() {
        if npc.seen {
            continue;
        }
        let dist = pos.distance_taxicab(item_pos);
        if dist < 4.5 {
            npc.trigger += dt;
            if npc.trigger > 1.0 {
                ev_npc.write(NpcHelpEvent::new(entity));
            }
        } else {
            npc.trigger = 0.0;
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_message::<NpcHelpEvent>()
        .init_resource::<NpcUIData>()
        .add_systems(Update, npchelp_event)
        .add_systems(OnEnter(GameState::NpcHelp), setup_ui)
        .add_systems(OnExit(GameState::NpcHelp), cleanup)
        .add_systems(Update, keyboard)
        .add_systems(Update, auto_call_npchelp);
}

pub mod activity;
pub mod journal;
pub mod journalui;
pub mod sanity;
pub mod sensors;
pub mod ui;
pub mod uibutton;

use bevy::app::App;
use bevy::prelude::*;

use crate::game::GameConfig;
use crate::gear::playergear::PlayerGear;
use crate::player::PlayerSprite;
use crate::{ghost_definitions::GhostType, root};

#[derive(Component, Debug)]
pub struct TruckUI;

#[derive(Clone, Debug, Event)]
pub enum TruckUIEvent {
    EndMission,
    ExitTruck,
    CraftRepellent,
}

#[derive(Component, Debug)]
pub struct TruckUIGhostGuess;

#[derive(Debug, Resource, Default)]
pub struct GhostGuess {
    pub ghost_type: Option<GhostType>,
}

pub fn cleanup(mut commands: Commands, qtui: Query<Entity, With<TruckUI>>) {
    for e in qtui.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn show_ui(mut qtui: Query<&mut Visibility, With<TruckUI>>) {
    for mut v in qtui.iter_mut() {
        *v = Visibility::Inherited;
    }
}

pub fn hide_ui(mut qtui: Query<&mut Visibility, With<TruckUI>>) {
    for mut v in qtui.iter_mut() {
        *v = Visibility::Hidden;
    }
}

pub fn keyboard(
    game_state: Res<State<root::GameState>>,
    mut game_next_state: ResMut<NextState<root::GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if *game_state.get() != root::GameState::Truck {
        return;
    }
    if keyboard_input.just_pressed(KeyCode::Escape) {
        game_next_state.set(root::GameState::None);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn truckui_event_handle(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_truckui: EventReader<TruckUIEvent>,
    mut next_state: ResMut<NextState<root::State>>,
    mut game_next_state: ResMut<NextState<root::GameState>>,
    gg: Res<GhostGuess>,
    gc: Res<GameConfig>,
    mut q_gear: Query<(&PlayerSprite, &mut PlayerGear)>,
) {
    for ev in ev_truckui.read() {
        match ev {
            TruckUIEvent::EndMission => {
                game_next_state.set(root::GameState::None);
                next_state.set(root::State::Summary);
            }
            TruckUIEvent::ExitTruck => game_next_state.set(root::GameState::None),
            TruckUIEvent::CraftRepellent => {
                for (player, mut gear) in q_gear.iter_mut() {
                    if player.id == gc.player_id {
                        if let Some(ghost_type) = gg.ghost_type {
                            gear.craft_repellent(ghost_type);
                            commands.spawn(AudioBundle {
                                source: asset_server.load("sounds/effects-dingdingding.ogg"),
                                settings: PlaybackSettings {
                                    mode: bevy::audio::PlaybackMode::Despawn,
                                    volume: bevy::audio::Volume::new(1.0),
                                    speed: 1.0,
                                    paused: false,
                                    spatial: false,
                                    spatial_scale: None,
                                },
                            });
                        }
                    }
                }
            }
        }
    }
}

pub fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(root::State::InGame), ui::setup_ui)
        .add_systems(OnExit(root::State::InGame), cleanup)
        .add_systems(OnEnter(root::GameState::Truck), show_ui)
        .add_systems(OnExit(root::GameState::Truck), hide_ui)
        .add_event::<TruckUIEvent>()
        .init_resource::<GhostGuess>()
        .add_systems(Update, keyboard)
        .add_systems(Update, journal::ghost_guess_system)
        .add_systems(
            FixedUpdate,
            (journal::button_system, sanity::update_sanity)
                .run_if(in_state(root::GameState::Truck)),
        )
        .add_systems(Update, truckui_event_handle);
}

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use bevy_replicon::prelude::Remote;
use bevy_seedling::prelude::*;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use ungear_core::messages::{TruckLoadoutAction, TruckLoadoutMessage};
use ungearitems_core::events::RequestCraftRepellent;
use uninput_core::states::InGameUiState;
use uninvestigation_core::components::ghost_guess::GhostGuess;
use unmission_core::resources::MissionEndRequested;
use unmission_core::types::MissionEvent;
use unplayer_core::components::MainPlayer;
use unreplicon_core::components::MissionGoalEntity;
use unreplicon_core::messages::{MissionEndReason, RequestEndMission};
use unreplicon_core::repellent_tracker::RepellentCraftTracker;
use unreplicon_core::resources::{AuthorityRole, LobbyPresenceRole, LocalPlayerRole};
use unsettings_core::audio::AudioSettings;
use untruck_core::components::in_truck::InTruck;
use untruck_core::events::truck::TruckUIEvent;

// Initialize the repellent craft tracker when entering a mission
pub(crate) fn init_repellent_tracker(
    q_missiongoal: Query<Entity, (Added<MissionGoalEntity>, Without<Remote>)>,
    difficulty: Res<CurrentDifficulty>,
    mut commands: Commands,
) {
    for e in q_missiongoal {
        let craft_limit = difficulty.0.repellent_craft_limit();
        info!(
            "Setting MissionGoal RepellentCraftTracker to {craft_limit} from {:?}",
            difficulty.0
        );
        commands
            .entity(e)
            .insert(RepellentCraftTracker::new(craft_limit));
    }
}

#[derive(SystemParam)]
struct TruckNetParams<'w> {
    mission_end_requested: Res<'w, MissionEndRequested>,
}

fn truckui_event_handle(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_truckui: MessageReader<TruckUIEvent>,
    q_gg: Query<&GhostGuess, With<MissionGoalEntity>>,
    audio_settings: Res<Persistent<AudioSettings>>,
    mut ev_craft_req: MessageWriter<RequestCraftRepellent>,
    mut ev_loadout: MessageWriter<TruckLoadoutMessage>,
    mut ev_end_mission: MessageWriter<RequestEndMission>,
    mut ev_mission: MessageWriter<MissionEvent>,
    net_params: TruckNetParams,
    authority: Option<Res<AuthorityRole>>,
    local_player_role: Option<Res<LocalPlayerRole>>,
    lobby_presence: Option<Res<LobbyPresenceRole>>,
    q_player: Query<Entity, (With<MainPlayer>, With<InTruck>)>,
) {
    if ev_truckui.is_empty() {
        return;
    }
    let Ok(gg) = q_gg.single() else {
        error!("TruckUI: truckui_event_handle running but MissionGoalEntity is missing!");
        return;
    };

    for ev in ev_truckui.read() {
        match ev {
            TruckUIEvent::EndMission => {
                if !net_params.mission_end_requested.0 {
                    continue;
                }
                if lobby_presence.is_none() {
                    // TODO(multiplayer-first): remove once single-player uses a local in-memory transport.
                    // Offline single-player: no network transport exists, so RequestEndMission
                    // (a client message) would never be delivered. Write MissionEvent::End directly.
                    info!(
                        "EndMission: offline single-player path, writing MissionEvent::End directly"
                    );
                    ev_mission.write(MissionEvent::End);
                } else {
                    ev_end_mission.write(RequestEndMission {
                        reason: MissionEndReason::TruckExitInitiated,
                    });
                }
            }
            TruckUIEvent::ExitTruck => {
                for entity in q_player.iter() {
                    commands.entity(entity).remove::<InTruck>();
                }
            }
            TruckUIEvent::CraftRepellent => {
                if let Some(ghost_type) = gg.ghost_type {
                    let in_truck_main_players: Vec<Entity> = q_player.iter().collect();

                    debug!(
                        "REPELLENT: TruckUIEvent::CraftRepellent received authority={} local_player_role={} main_players_in_truck={:?} ghost_type={:?}",
                        authority.is_some(),
                        local_player_role.is_some(),
                        in_truck_main_players,
                        ghost_type,
                    );

                    if authority.is_none() {
                        warn!(
                            "REPELLENT: Craft repellent requested on non-authority node; sending TruckLoadoutMessage::CraftRepellent for ghost_type={:?}",
                            ghost_type
                        );
                        ev_loadout.write(TruckLoadoutMessage {
                            action: TruckLoadoutAction::CraftRepellent(ghost_type),
                        });
                    } else if let Some(player_entity) = in_truck_main_players.first().copied() {
                        debug!(
                            "REPELLENT: Craft repellent requested on authority node; writing local RequestCraftRepellent for ghost_type={:?}",
                            ghost_type
                        );
                        ev_craft_req.write(RequestCraftRepellent {
                            ghost_type,
                            player_entity,
                        });
                    } else {
                        warn!(
                            "REPELLENT: Craft repellent requested on authority node, but no MainPlayer found in truck!"
                        );
                    }

                    commands.spawn((
                        SamplePlayer::new(asset_server.load("sounds/effects-dingdingding.ogg")),
                        sample_effects![VolumeNode {
                            volume: Volume::Linear(
                                1.0 * audio_settings.volume_master.as_f32()
                                    * audio_settings.volume_effects.as_f32(),
                            ),
                            ..default()
                        }],
                        DefaultPool,
                    ));
                } else {
                    warn!(
                        "REPELLENT: CraftRepellent requested but no ghost type selected in journal"
                    );
                }
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        truckui_event_handle.run_if(in_state(InGameUiState::Truck)),
    );
}

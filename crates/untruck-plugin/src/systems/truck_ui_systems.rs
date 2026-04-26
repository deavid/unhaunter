use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use ungear_core::messages::{TruckLoadoutAction, TruckLoadoutMessage};
use ungearitems_core::events::RequestCraftRepellent;
use uninput_core::states::InGameUiState;
use uninvestigation_core::resources::ghost_guess::GhostGuess;
use unmission_core::resources::MissionEndRequested;
use unmission_core::types::MissionEvent;
use unplayer_core::components::MainPlayer;
use unreplicon_core::messages::{
    MissionEndReason, PlayPositionalSoundBroadcast, RequestEndMission,
};
use unreplicon_core::resources::{AuthorityRole, LobbyPresenceRole, LocalPlayerRole};
use unsettings_core::audio::AudioSettings;
use untruck_core::components::in_truck::InTruck;
use untruck_core::events::truck::TruckUIEvent;
use untruck_core::types::repellent_tracker::RepellentCraftTracker;

// Initialize the repellent craft tracker when entering a mission
pub(crate) fn init_repellent_tracker(
    mut craft_tracker: ResMut<RepellentCraftTracker>,
    difficulty: Res<CurrentDifficulty>,
) {
    craft_tracker.reset(difficulty.0.repellent_craft_limit());
}

// Reset the repellent craft tracker when leaving the game
pub(crate) fn reset_repellent_tracker(mut craft_tracker: ResMut<RepellentCraftTracker>) {
    craft_tracker.reset(0);
}

#[derive(SystemParam)]
struct TruckNetParams<'w, 's> {
    mission_end_requested: Res<'w, MissionEndRequested>,
    ev_end_mission: MessageWriter<'w, RequestEndMission>,
    ev_mission: MessageWriter<'w, MissionEvent>,
    ev_broadcast: MessageWriter<'w, bevy_replicon::prelude::ToClients<PlayPositionalSoundBroadcast>>,
    authority: Option<Res<'w, AuthorityRole>>,
    lobby_presence: Option<Res<'w, LobbyPresenceRole>>,
    q_van_entry: Query<'w, 's, &'static unspatial_core::position::Position, With<unboard_core::components::spawning::VanEntryPoint>>,
}

fn truckui_event_handle(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_truckui: MessageReader<TruckUIEvent>,
    gg: Res<GhostGuess>,
    audio_settings: Res<Persistent<AudioSettings>>,
    mut craft_tracker: ResMut<RepellentCraftTracker>,
    mut ev_craft_req: MessageWriter<RequestCraftRepellent>,
    mut ev_loadout: MessageWriter<TruckLoadoutMessage>,
    mut net_params: TruckNetParams,
    local_player_role: Option<Res<LocalPlayerRole>>,
    q_player: Query<Entity, (With<MainPlayer>, With<InTruck>)>,
    q_player_gear: Query<&ungear_core::components::playergear::PlayerGear, With<MainPlayer>>,
    q_repellent_flask: Query<&ungearitems_core::components::repellentflask::RepellentFlask>,
) {
    for ev in ev_truckui.read() {
        match ev {
            TruckUIEvent::EndMission => {
                if !net_params.mission_end_requested.0 {
                    continue;
                }
                if net_params.lobby_presence.is_none() {
                    // TODO(multiplayer-first): remove once single-player uses a local in-memory transport.
                    // Offline single-player: no network transport exists, so RequestEndMission
                    // (a client message) would never be delivered. Write MissionEvent::End directly.
                    info!(
                        "EndMission: offline single-player path, writing MissionEvent::End directly"
                    );
                    net_params.ev_mission.write(MissionEvent::End);
                } else {
                    net_params.ev_end_mission.write(RequestEndMission {
                        reason: MissionEndReason::TruckExitInitiated,
                    });
                }
                // Broadcast sound on authority
                if net_params.authority.is_some() {
                    let pos = net_params.q_van_entry
                        .iter()
                        .next()
                        .cloned()
                        .unwrap_or_default()
                        .to_vec3();
                    net_params.ev_broadcast.write(bevy_replicon::prelude::ToClients {
                        mode: bevy_replicon::prelude::SendMode::Broadcast,
                        message: PlayPositionalSoundBroadcast {
                            sound_path: "sounds/effects-dingdingding.ogg".to_string(),
                            position: pos.into(),
                            volume: 1.0,
                        },
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
                    let before_remaining = craft_tracker.remaining_crafts();

                    debug!(
                        "REPELLENT: TruckUIEvent::CraftRepellent received authority={} local_player_role={} main_players_in_truck={:?} ghost_type={:?} remaining_before={}",
                        net_params.authority.is_some(),
                        local_player_role.is_some(),
                        in_truck_main_players,
                        ghost_type,
                        before_remaining
                    );

                    let mut free_swap = false;
                    // B06: Check if we can do a free swap
                    if let Ok(p_gear) = q_player_gear.single() {
                        let entities = [p_gear.left_hand, p_gear.right_hand]
                            .into_iter()
                            .flatten()
                            .chain(p_gear.inventory.iter().copied());

                        for entity in entities {
                            if let Ok(flask) = q_repellent_flask.get(entity) {
                                if flask.qty == ungearitems_core::components::repellentflask::RepellentFlask::MAX_QTY && !flask.active {
                                    // It's a full, unopened flask.
                                    if flask.liquid_content == Some(ghost_type) {
                                        // Already the correct type, no need to do anything, but let's treat it as a "success" to exit truck.
                                        free_swap = true;
                                    } else {
                                        // Different type, we can swap for free.
                                        free_swap = true;
                                    }
                                    break;
                                }
                            }
                        }
                    }

                    if net_params.authority.is_none() {
                        warn!(
                            "REPELLENT: Craft repellent requested on non-authority node; sending TruckLoadoutMessage::CraftRepellent for ghost_type={:?}",
                            ghost_type
                        );
                        ev_loadout.write(TruckLoadoutMessage {
                            action: TruckLoadoutAction::CraftRepellent(ghost_type),
                        });
                    } else {
                        debug!(
                            "REPELLENT: Craft repellent requested on authority node; writing local RequestCraftRepellent for ghost_type={:?}",
                            ghost_type
                        );
                        ev_craft_req.write(RequestCraftRepellent { ghost_type });

                        // Broadcast sound on authority
                        let pos = net_params.q_van_entry
                            .iter()
                            .next()
                            .cloned()
                            .unwrap_or_default()
                            .to_vec3();
                        net_params.ev_broadcast.write(bevy_replicon::prelude::ToClients {
                            mode: bevy_replicon::prelude::SendMode::Broadcast,
                            message: PlayPositionalSoundBroadcast {
                                sound_path: "sounds/effects-dingdingding.ogg".to_string(),
                                position: pos.into(),
                                volume: 1.0,
                            },
                        });
                    }
                    if !free_swap {
                        craft_tracker.craft();
                    }

                    debug!(
                        "REPELLENT: Craft request dispatched for ghost_type={:?}; remaining_after={}",
                        ghost_type,
                        craft_tracker.remaining_crafts()
                    );

                    if net_params.authority.is_none() || net_params.lobby_presence.is_none() {
                        // Offline or pure client: play local sound immediately
                        commands
                            .spawn(AudioPlayer::new(
                                asset_server.load("sounds/effects-dingdingding.ogg"),
                            ))
                            .insert(PlaybackSettings {
                                mode: bevy::audio::PlaybackMode::Despawn,
                                volume: bevy::audio::Volume::Linear(
                                    1.0 * audio_settings.volume_master.as_f32()
                                        * audio_settings.volume_effects.as_f32(),
                                ),
                                speed: 1.0,
                                paused: false,
                                spatial: false,
                                spatial_scale: None,
                                ..Default::default()
                            });
                    }

                    // Automatically exit the truck after crafting repellent
                    for entity in in_truck_main_players {
                        debug!(
                            "REPELLENT: Removing InTruck from MainPlayer entity {:?} after craft request",
                            entity
                        );
                        commands.entity(entity).remove::<InTruck>();
                    }
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
        (truckui_event_handle).run_if(in_state(InGameUiState::Truck)),
    );
}

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use ungearitems_core::events::RequestCraftRepellent;
use uninput_core::states::InGameUiState;
use uninvestigation_core::resources::ghost_guess::GhostGuess;
use unmission_core::resources::MissionEndRequested;
use unmission_core::types::MissionEvent;
use unplayer_core::components::MainPlayer;
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
struct TruckNetParams<'w> {
    mission_end_requested: Res<'w, MissionEndRequested>,
}

fn truckui_event_handle(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_truckui: MessageReader<TruckUIEvent>,
    gg: Res<GhostGuess>,
    audio_settings: Res<Persistent<AudioSettings>>,
    mut craft_tracker: ResMut<RepellentCraftTracker>,
    mut ev_craft_req: MessageWriter<RequestCraftRepellent>,
    mut ev_mission: MessageWriter<MissionEvent>,
    net_params: TruckNetParams,
    q_player: Query<Entity, (With<MainPlayer>, With<InTruck>)>,
) {
    for ev in ev_truckui.read() {
        match ev {
            TruckUIEvent::EndMission => {
                if !net_params.mission_end_requested.0 {
                    continue;
                }
                ev_mission.write(MissionEvent::End);
            }
            TruckUIEvent::ExitTruck => {
                for entity in q_player.iter() {
                    commands.entity(entity).remove::<InTruck>();
                }
            }
            TruckUIEvent::CraftRepellent => {
                if let Some(ghost_type) = gg.ghost_type {
                    ev_craft_req.write(RequestCraftRepellent { ghost_type });
                    craft_tracker.craft();

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

                    // Automatically exit the truck after crafting repellent
                    for entity in q_player.iter() {
                        commands.entity(entity).remove::<InTruck>();
                    }
                } else {
                    debug!("CraftRepellent requested but no ghost type selected in journal");
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

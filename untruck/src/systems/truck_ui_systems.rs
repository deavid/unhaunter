use crate::craft_repellent::craft_repellent;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use uncore::components::game_config::GameConfig;
use uncore::components::player_sprite::PlayerSprite;
use uncore::components::truck::TruckUI;
use uncore::components::truck_ui_button::TruckUIButton;
use uncore::events::truck::TruckUIEvent;
use uncore::resources::board_data::BoardData;
use uncore::resources::ghost_guess::GhostGuess;
use uncore::resources::summary_data::SummaryData;
use uncore::states::{AppState, GameState};
use uncore::types::truck_button::TruckButtonType;
use ungear::components::playergear::PlayerGear;
use unprofile::data::PlayerProfileData;
use unsettings::audio::AudioSettings;

// Component to mark the progress bar for hold buttons
#[derive(Component)]
pub struct ProgressIndicator;

// Entity resource to track the audio player for the hold sound
#[derive(Resource, Default)]
pub struct HoldSoundEntity(pub Option<Entity>);

fn cleanup(mut commands: Commands, qtui: Query<Entity, With<TruckUI>>) {
    for e in qtui.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn show_ui(mut qtui: Query<&mut Visibility, With<TruckUI>>) {
    for mut v in qtui.iter_mut() {
        *v = Visibility::Inherited;
    }
}

fn hide_ui(mut qtui: Query<&mut Visibility, With<TruckUI>>) {
    for mut v in qtui.iter_mut() {
        *v = Visibility::Hidden;
    }
}

fn keyboard(
    game_state: Res<State<GameState>>,
    mut game_next_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if *game_state.get() != GameState::Truck {
        return;
    }
    if keyboard_input.just_pressed(KeyCode::Escape) {
        game_next_state.set(GameState::None);
    }
}

/// Handles the "click and hold" mechanic for buttons in the truck UI.
///
/// This system manages buttons that require being held down for a specific duration
/// before triggering their action. It provides both visual feedback (a progress bar)
/// and audio feedback (a looping sound).
///
/// # Functionality
///
/// - Tracks buttons that are being actively held
/// - Creates a progress bar when a button is first held down
/// - Updates the progress bar width as the hold time increases
/// - Plays a looping sound during the hold
/// - Triggers the appropriate event when the hold duration is reached
/// - Cleans up resources when buttons are no longer held
///
/// # Progress Bar
///
/// The progress bar is a colored horizontal bar that grows from 0% to 100% width
/// during the hold duration. It's positioned at the bottom of the button and uses
/// a high z-index to ensure visibility.
///
/// # Sound
///
/// Plays "sounds/fadein-progress-1000ms.ogg" while the button is being held.
/// The sound is stopped when the hold is cancelled.
///
/// # Events
///
/// When a hold is completed, this system sends the appropriate event based on the
/// button type (e.g., `TruckUIEvent::CraftRepellent` or `TruckUIEvent::EndMission`).
fn hold_button_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    audio_settings: Res<Persistent<AudioSettings>>,
    mut interaction_query: Query<
        (&Interaction, &mut TruckUIButton, &Children, Entity),
        With<Button>,
    >,
    mut node_query: Query<&mut Node>,
    progress_query: Query<(Entity, &Parent), With<ProgressIndicator>>,
    mut ev_truckui: EventWriter<TruckUIEvent>,
    mut hold_sound: Local<Option<Entity>>,
) {
    // Track which buttons are currently being held
    let mut active_buttons = Vec::new();

    // Handle buttons that need hold interaction
    for (interaction, mut button, _children, button_entity) in &mut interaction_query {
        // Skip buttons that don't require holding
        if button.hold_duration.is_none() {
            continue;
        }

        // Skip disabled buttons
        if button.disabled {
            continue;
        }

        // Keep track of buttons that are being actively held
        if *interaction == Interaction::Pressed && button.holding {
            active_buttons.push(button_entity);
        }

        // Extract values we need before mutable borrows
        let hold_duration = button.hold_duration.unwrap();
        let button_class = button.class.clone(); // Clone the enum to avoid borrowing issues

        match *interaction {
            Interaction::Pressed => {
                if !button.holding {
                    // Start holding
                    button.holding = true;
                    button.hold_timer = Some(0.0);

                    info!("Button hold started: {:?}", button_class);

                    // Only spawn a new progress bar if none exists for this button
                    let has_progress_bar = progress_query
                        .iter()
                        .any(|(_, parent)| parent.get() == button_entity);

                    if !has_progress_bar {
                        // Create progress bar with very distinctive appearance
                        let progress_entity = commands
                            .spawn((
                                ProgressIndicator,
                                Node {
                                    position_type: PositionType::Absolute,
                                    bottom: Val::Px(0.0),
                                    left: Val::Px(0.7),
                                    width: Val::Percent(0.0), // Start at 0%
                                    height: Val::Px(20.0),    // Much taller for visibility
                                    ..default()
                                },
                                // Bright yellow for maximum visibility
                                BackgroundColor(Color::srgba(1.0, 1.0, 0.0, 0.2)),
                                ZIndex(999),
                            ))
                            .id();

                        // Add progress bar directly to button
                        commands.entity(button_entity).add_child(progress_entity);
                        info!(
                            "Added progress bar: {:?} to button: {:?}",
                            progress_entity, button_entity
                        );
                    }

                    // Play sound
                    let sound_entity = commands
                        .spawn(AudioPlayer::new(
                            asset_server.load("sounds/fadein-progress-1000ms.ogg"),
                        ))
                        .insert(PlaybackSettings {
                            mode: bevy::audio::PlaybackMode::Despawn,
                            volume: bevy::audio::Volume::new(
                                1.0 * audio_settings.volume_master.as_f32()
                                    * audio_settings.volume_effects.as_f32(),
                            ),
                            ..default()
                        })
                        .id();

                    // Store sound entity to stop it later
                    *hold_sound = Some(sound_entity);
                }

                // Update timer
                if let Some(hold_timer) = &mut button.hold_timer {
                    let delta = time.delta_secs();
                    *hold_timer += delta;

                    // Update all progress bars for this button
                    let progress = (*hold_timer / hold_duration).clamp(0.0, 1.0);

                    for (progress_entity, parent) in &progress_query {
                        if parent.get() == button_entity {
                            if let Ok(mut node) = node_query.get_mut(progress_entity) {
                                // We only cover up to 99% to avoid overflowing the button due to the borders.
                                node.width = Val::Percent(progress.abs().sqrt() * 99.0);
                            }
                        }
                    }

                    // Check if hold is complete
                    if *hold_timer >= hold_duration {
                        info!("Button hold complete: {:?}", button_class);

                        // Trigger action
                        match button_class {
                            TruckButtonType::CraftRepellent => {
                                button.disabled = true; // Disable button to prevent multiple triggers
                                ev_truckui.send(TruckUIEvent::CraftRepellent);
                                info!("Sent CraftRepellent event");
                            }
                            TruckButtonType::EndMission => {
                                ev_truckui.send(TruckUIEvent::EndMission);
                                info!("Sent EndMission event");
                            }
                            _ => {}
                        }

                        // Reset button state
                        button.holding = false;
                        button.hold_timer = None;
                    }
                }
            }
            _ => {
                // Button is no longer pressed, reset state
                if button.holding {
                    info!("Button hold canceled: {:?}", button_class);
                    button.holding = false;
                    button.hold_timer = None;

                    // Stop sound
                    if let Some(entity) = hold_sound.take() {
                        if let Some(cmd_e) = commands.get_entity(entity) {
                            cmd_e.despawn_recursive();
                        }
                    }
                }
            }
        }
    }

    // Clean up progress bars for buttons that are no longer being held or are disabled
    for (entity, parent) in progress_query.iter() {
        let button_entity = parent.get();
        let button_is_active = active_buttons.contains(&button_entity);

        // Also get the button to check if it's disabled
        let button_is_disabled = interaction_query
            .iter()
            .find(|(_, _, _, e)| *e == button_entity)
            .map(|(_, button, _, _)| button.disabled)
            .unwrap_or(false);

        if !button_is_active || button_is_disabled {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn truckui_event_handle(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_truckui: EventReader<TruckUIEvent>,
    mut next_state: ResMut<NextState<AppState>>,
    mut game_next_state: ResMut<NextState<GameState>>,
    gg: Res<GhostGuess>,
    gc: Res<GameConfig>,
    mut q_gear: Query<(&PlayerSprite, &mut PlayerGear)>,
    audio_settings: Res<Persistent<AudioSettings>>,
    mut summary_data: ResMut<SummaryData>,
    board_data: Res<BoardData>,
    mut player_profile: ResMut<Persistent<PlayerProfileData>>,
) {
    for ev in ev_truckui.read() {
        match ev {
            TruckUIEvent::EndMission => {
                // Debug: Log the current state of board_data.map_path
                info!(
                    "[EndMission] Current board_data.map_path: '{}'",
                    board_data.map_path
                );

                let initial_deposit_held = player_profile.progression.insurance_deposit;

                player_profile.progression.bank += initial_deposit_held;
                player_profile.progression.insurance_deposit = 0;

                if let Err(e) = player_profile.persist() {
                    panic!("Failed to persist PlayerProfileData: {:?}", e);
                }

                // Set summary_data.current_mission_id from board_data.map_path
                summary_data.map_path = board_data.map_path.clone();

                // Debug: Log the updated value of summary_data.current_mission_id
                info!(
                    "[EndMission] Set summary_data.current_mission_id to: '{}'",
                    summary_data.map_path
                );

                summary_data.deposit_originally_held = initial_deposit_held;
                summary_data.deposit_returned_to_bank = initial_deposit_held;
                summary_data.costs_deducted_from_deposit = 0;
                summary_data.money_earned = 0;

                if summary_data.ghosts_unhaunted == summary_data.ghost_types.len() as u32 {
                    // All ghosts were unhaunted, successful completion
                    summary_data.mission_successful = true;
                } else {
                    summary_data.mission_successful = false;
                }
                // grade_achieved is now determined in the summary screen based on mission_successful

                game_next_state.set(GameState::None);
                next_state.set(AppState::Summary);
            }
            TruckUIEvent::ExitTruck => game_next_state.set(GameState::None),
            TruckUIEvent::CraftRepellent => {
                for (player, mut gear) in q_gear.iter_mut() {
                    if player.id == gc.player_id {
                        if let Some(ghost_type) = gg.ghost_type {
                            craft_repellent(&mut gear, ghost_type);
                            commands
                                .spawn(AudioPlayer::new(
                                    asset_server.load("sounds/effects-dingdingding.ogg"),
                                ))
                                .insert(PlaybackSettings {
                                    mode: bevy::audio::PlaybackMode::Despawn,
                                    volume: bevy::audio::Volume::new(
                                        1.0 * audio_settings.volume_master.as_f32()
                                            * audio_settings.volume_effects.as_f32(),
                                    ),
                                    speed: 1.0,
                                    paused: false,
                                    spatial: false,
                                    spatial_scale: None,
                                });
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(OnExit(AppState::InGame), cleanup);
    app.add_systems(OnEnter(GameState::Truck), show_ui);
    app.add_systems(OnExit(GameState::Truck), hide_ui);
    app.add_systems(Update, keyboard);
    app.add_systems(
        Update,
        (
            hold_button_system,
            truckui_event_handle.after(hold_button_system),
        )
            .run_if(in_state(GameState::Truck)),
    );
}

use bevy::prelude::*;
use bevy_persistent::Persistent;
use uncommon_states_core::UIContextState;
use uninput_core::states::InGameUiState;
use unreplicon_core::components::MissionGoalEntity;
use unreplicon_core::repellent_tracker::RepellentCraftTracker;
use unsettings_core::audio::AudioSettings;
use untruck_core::components::truck_ui_button::TruckUIButton;
use untruck_core::components::truck_ui_markers::TruckUI;
use untruck_core::events::truck::TruckUIEvent;
use untruck_core::types::truck_button::TruckButtonType;

// Component to mark the progress bar for hold buttons
#[derive(Component)]
pub(crate) struct ProgressIndicator;

fn cleanup(mut commands: Commands, qtui: Query<Entity, With<TruckUI>>) {
    for e in qtui.iter() {
        commands.entity(e).despawn();
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
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut ev_truckui: MessageWriter<TruckUIEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        ev_truckui.write(TruckUIEvent::ExitTruck);
    }
}

fn truck_button_cooldown_system(time: Res<Time>, mut q_button: Query<&mut TruckUIButton>) {
    for mut button in &mut q_button {
        if button.cooldown_timer > 0.0 {
            button.cooldown_timer = (button.cooldown_timer - time.delta_secs()).max(0.0);
        }
    }
}

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
    progress_query: Query<(Entity, &ChildOf), With<ProgressIndicator>>,
    mut ev_truckui: MessageWriter<TruckUIEvent>,
    mut hold_sound: Local<Option<Entity>>,
    q_craft_tracker: Query<&RepellentCraftTracker, With<MissionGoalEntity>>,
) {
    let Ok(craft_tracker) = q_craft_tracker.single() else {
        warn_once!("Craft Tracker component not found");
        return;
    };
    // Track which buttons are currently being held
    let mut active_buttons = Vec::new();

    // Handle buttons that need hold interaction
    for (interaction, mut button, _children, button_entity) in &mut interaction_query {
        // Skip buttons that don't require holding
        if button.hold_duration.is_none() {
            continue;
        }

        // Gating: reset require_release when button is not pressed
        if *interaction != Interaction::Pressed {
            button.require_release = false;
        }

        // Skip disabled buttons
        if button.disabled {
            continue;
        }

        // Check if this is a craft repellent button and we've reached the limit
        if matches!(button.class, TruckButtonType::CraftRepellent) && !craft_tracker.can_craft() {
            button.disabled = true;
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
                // Cooldown and gating check
                if button.cooldown_timer > 0.0 || button.require_release {
                    if button.holding {
                        button.holding = false;
                        button.hold_timer = None;
                        if let Some(entity) = hold_sound.take()
                            && let Ok(mut cmd_e) = commands.get_entity(entity)
                        {
                            cmd_e.despawn();
                        }
                    }
                    continue;
                }

                if !button.holding {
                    // Start holding
                    button.holding = true;
                    button.hold_timer = Some(0.0);

                    info!("Button hold started: {:?}", button_class);

                    // Only spawn a new progress bar if none exists for this button
                    let has_progress_bar = progress_query
                        .iter()
                        .any(|(_, parent)| parent.parent() == button_entity);

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
                            volume: bevy::audio::Volume::Linear(
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
                        if parent.parent() == button_entity
                            && let Ok(mut node) = node_query.get_mut(progress_entity)
                        {
                            // We only cover up to 99% to avoid overflowing the button due to the borders.
                            node.width = Val::Percent(progress.abs().sqrt() * 99.0);
                        }
                    }

                    // Check if hold is complete
                    if *hold_timer >= hold_duration {
                        info!("Button hold complete: {:?}", button_class);

                        // Trigger action
                        match button_class {
                            TruckButtonType::CraftRepellent => {
                                // Check if we can still craft
                                if craft_tracker.can_craft() {
                                    button.disabled = true; // Disable button to prevent multiple triggers
                                    ev_truckui.write(TruckUIEvent::CraftRepellent);
                                    info!("Sent CraftRepellent event");
                                } else {
                                    info!("Craft repellent limit reached!");
                                    // Optimization for net clients: if the UI haven't updated yet, don't let them click.
                                }
                            }
                            TruckButtonType::EndMission => {
                                ev_truckui.write(TruckUIEvent::EndMission);
                                info!("Sent EndMission event");
                            }
                            _ => {}
                        }

                        // Activate cooldown and require release
                        button.cooldown_timer = 1.0;
                        button.require_release = true;

                        // Stop sound
                        if let Some(entity) = hold_sound.take()
                            && let Ok(mut cmd_e) = commands.get_entity(entity)
                        {
                            cmd_e.despawn();
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
                    if let Some(entity) = hold_sound.take()
                        && let Ok(mut cmd_e) = commands.get_entity(entity)
                    {
                        cmd_e.despawn();
                    }
                }
            }
        }
    }

    // Clean up progress bars for buttons that are no longer being held or are disabled
    for (entity, parent) in progress_query.iter() {
        let button_entity = parent.parent();
        let button_is_active = active_buttons.contains(&button_entity);

        // Also get the button to check if it's disabled
        let button_is_disabled = interaction_query
            .iter()
            .find(|(_, _, _, e)| *e == button_entity)
            .map(|(_, button, _, _)| button.disabled)
            .unwrap_or(false);

        if !button_is_active || button_is_disabled {
            commands.entity(entity).despawn();
        }
    }
}

fn update_craft_button_text(
    mut q_button: Query<(&mut TruckUIButton, &Children), With<Button>>,
    mut q_text: Query<&mut Text>,
    q_craft_tracker: Query<Ref<RepellentCraftTracker>, With<MissionGoalEntity>>,
) {
    let Ok(craft_tracker) = q_craft_tracker.single() else {
        warn_once!("Craft Tracker component not found");
        return;
    };

    // Only update when the resource has changed
    if !craft_tracker.is_changed() {
        return;
    }

    for (mut button, children) in &mut q_button {
        if matches!(button.class, TruckButtonType::CraftRepellent) {
            let remaining = craft_tracker.remaining_crafts();
            let can_craft = craft_tracker.can_craft();

            // Update button disabled state
            button.disabled = !can_craft;

            // Find the text child and update text
            for &child in children {
                if let Ok(mut text) = q_text.get_mut(child) {
                    if remaining > 0 {
                        text.0 = format!("Craft Repellent ({})", remaining);
                    } else {
                        text.0 = "End Mission - No More Repellents".to_string();
                    }
                    break;
                }
            }
            break;
        }
    }
}

fn update_end_mission_button_status(
    mission_end_req: Res<unmission_core::resources::MissionEndRequested>,
    mut q_button: Query<&mut TruckUIButton, With<Button>>,
) {
    if !mission_end_req.is_changed() {
        return;
    }
    for mut button in &mut q_button {
        if matches!(button.class, TruckButtonType::EndMission) {
            button.disabled = !mission_end_req.0;
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(OnExit(UIContextState::InGame), cleanup);
    app.add_systems(OnEnter(InGameUiState::Truck), show_ui);
    app.add_systems(OnExit(InGameUiState::Truck), hide_ui);
    app.add_systems(Update, keyboard.run_if(in_state(InGameUiState::Truck)));
    app.add_systems(
        Update,
        (
            truck_button_cooldown_system,
            hold_button_system,
            update_craft_button_text,
            update_end_mission_button_status,
        )
            .run_if(in_state(InGameUiState::Truck)),
    );
}

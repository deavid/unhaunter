use bevy::prelude::Resource;
use bevy::time::Stopwatch;

#[derive(Resource, Default, Debug)]
pub struct InTruckCraftPromptTimer {
    pub delay_timer: Option<Stopwatch>, // Changed from prompt_at_time: Option<f32>
}

use bevy::prelude::*;
use uncore::{resources::ghost_guess::GhostGuess, types::gear_kind::GearKind};
use ungear::components::playergear::PlayerGear;
use ungearitems::components::repellentflask::RepellentFlask;
use unwalkiecore::{events::WalkieEvent, resources::WalkiePlay};

pub fn trigger_in_truck_craft_prompt_system(
    mut timer: ResMut<InTruckCraftPromptTimer>,
    ghost_guess: Res<GhostGuess>,
    player_query: Query<&PlayerGear>,
    mut walkie_play: ResMut<WalkiePlay>,
    time: Res<Time>,
) {
    const CRAFT_PROMPT_DELAY_SECONDS: f32 = 12.0;

    let initial_highlight_state = walkie_play.highlight_craft_button;
    // Default to false. It will be set to true ONLY if the walkie event successfully fires.
    // If the event doesn't fire (e.g., due to cooldown) AND the timer is still active (meaning we intend to prompt),
    // then we might want to keep the highlight if it was already on.
    walkie_play.highlight_craft_button = false;

    if let Some(identified_ghost) = ghost_guess.ghost_type {
        let mut has_correct_repellent = false;
        if let Ok(player_gear) = player_query.get_single() {
            for (gear, _epos) in player_gear.as_vec() {
                if gear.kind == GearKind::RepellentFlask {
                    if let Some(rep_data_dyn) = gear.data.as_ref() {
                        if let Some(flask) = <dyn std::any::Any>::downcast_ref::<RepellentFlask>(
                            rep_data_dyn.as_ref(),
                        ) {
                            if flask.liquid_content == Some(identified_ghost) && flask.qty > 0 {
                                has_correct_repellent = true;
                                break;
                            }
                        }
                    }
                }
            }
        }

        if !has_correct_repellent {
            if timer.delay_timer.is_none() {
                let new_timer = Stopwatch::new();
                // Start as paused to accumulate the exact delay we want
                timer.delay_timer = Some(new_timer);
                // info!("TruckCraftPrompt: Timer started for ghost {:?}. Will prompt in {}s.", identified_ghost.name(), CRAFT_PROMPT_DELAY_SECONDS);
            }
        } else {
            if timer.delay_timer.is_some() {
                // info!("TruckCraftPrompt: Player has correct repellent for {:?}. Clearing timer.", identified_ghost.name());
            }
            timer.delay_timer = None;
            // highlight_craft_button remains false as condition (no repellent) is not met
        }
    } else {
        if timer.delay_timer.is_some() {
            // info!("TruckCraftPrompt: No ghost identified. Clearing timer.");
        }
        timer.delay_timer = None;
        // highlight_craft_button remains false
    }

    if let Some(stopwatch) = timer.delay_timer.as_mut() {
        // Tick the timer forward
        stopwatch.tick(time.delta());

        if stopwatch.elapsed_secs() >= CRAFT_PROMPT_DELAY_SECONDS {
            // Re-check conditions before firing, especially if player could have crafted/picked up repellent
            let should_prompt = if let Some(identified_ghost_recheck) = ghost_guess.ghost_type {
                let mut has_correct_repellent_recheck = false;
                if let Ok(player_gear_recheck) = player_query.get_single() {
                    for (gear_recheck, _epos_recheck) in player_gear_recheck.as_vec() {
                        if gear_recheck.kind == GearKind::RepellentFlask {
                            if let Some(rep_data_dyn_recheck) = gear_recheck.data.as_ref() {
                                if let Some(flask_recheck) =
                                    <dyn std::any::Any>::downcast_ref::<RepellentFlask>(
                                        rep_data_dyn_recheck.as_ref(),
                                    )
                                {
                                    if flask_recheck.liquid_content
                                        == Some(identified_ghost_recheck)
                                        && flask_recheck.qty > 0
                                    {
                                        has_correct_repellent_recheck = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                !has_correct_repellent_recheck
            } else {
                false // No ghost ID'd, so shouldn't prompt
            };

            if should_prompt {
                // info!("TruckCraftPrompt: Attempting to trigger JournalPointsToOneGhostNoCraft walkie event.");
                if walkie_play.set(
                    WalkieEvent::JournalPointsToOneGhostNoCraft,
                    time.elapsed_secs_f64(),
                ) {
                    // info!("TruckCraftPrompt: Walkie event set. Highlighting craft button.");
                    walkie_play.highlight_craft_button = true;
                    timer.delay_timer = None; // Clear timer after successful prompt
                } else {
                    // info!("TruckCraftPrompt: Walkie event NOT set (e.g., cooldown). Timer remains. Highlight: {}", initial_highlight_state);
                    // If walkie couldn't be set (e.g. cooldown), retain original highlight state if it was already true
                    // AND the timer is still active (prompt_at_time will not be None yet).
                    // This ensures if it was highlighted and is still pending, it remains highlighted.
                    walkie_play.highlight_craft_button =
                        initial_highlight_state && timer.delay_timer.is_some();
                }
            } else {
                // info!("TruckCraftPrompt: Conditions for prompt changed (e.g., player crafted repellent). Clearing timer.");
                timer.delay_timer = None; // Conditions changed (e.g., player crafted repellent), clear timer
                // highlight_craft_button remains false
            }
        } else {
            // Timer is set, but not yet time to fire.
            // If it was already highlighted and timer is still running, keep it highlighted.
            // Otherwise, it's false.
            if initial_highlight_state && timer.delay_timer.is_some() {
                walkie_play.highlight_craft_button = true;
                //  info!("TruckCraftPrompt: Timer running ({}s left), highlight maintained.", CRAFT_PROMPT_DELAY_SECONDS - stopwatch.elapsed_secs());
            } else {
                // info!("TruckCraftPrompt: Timer running ({}s left), highlight false.", CRAFT_PROMPT_DELAY_SECONDS - stopwatch.elapsed_secs());
            }
        }
    }
}

use bevy::app::App;

use crate::triggers::base1;
use crate::triggers::basic_gear_usage;
use crate::triggers::consumables_and_defense;
use crate::triggers::environmental_awareness;
use crate::triggers::evidence_gathering_logic;
use crate::triggers::ghost_behavior_and_hunting;
use crate::triggers::locomotion_interaction;
use crate::triggers::mission_progression_and_truck;
use crate::triggers::player_wellbeing;
use crate::triggers::potential_id_prompt;
use crate::triggers::repellent_expulsion;
use crate::triggers::repellent_feedback;
use crate::triggers::truck_craft_prompt;
use crate::triggers::tutorial_gear_explanations;
use crate::triggers::tutorial_introductions;
use crate::triggers::tutorial_specific;

pub(crate) fn app_setup(app: &mut App) {
    base1::app_setup(app);
    basic_gear_usage::app_setup(app);
    consumables_and_defense::app_setup(app);
    environmental_awareness::app_setup(app);
    evidence_gathering_logic::app_setup(app);
    ghost_behavior_and_hunting::app_setup(app);
    locomotion_interaction::app_setup(app);
    mission_progression_and_truck::app_setup(app);
    player_wellbeing::app_setup(app);
    potential_id_prompt::app_setup(app);
    repellent_expulsion::app_setup(app);
    repellent_feedback::app_setup(app);
    truck_craft_prompt::app_setup(app);
    tutorial_gear_explanations::app_setup(app);
    tutorial_introductions::app_setup(app);
    tutorial_specific::app_setup(app);
}

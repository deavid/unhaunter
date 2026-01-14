use bevy::app::App;

pub(crate) mod base1;
pub(crate) mod basic_gear_usage;
pub(crate) mod consumables_and_defense;
pub(crate) mod environmental_awareness;
pub(crate) mod evidence_gathering_logic;
pub(crate) mod ghost_behavior_and_hunting;
pub(crate) mod locomotion_interaction;
pub(crate) mod mission_progression_and_truck;
pub(crate) mod player_wellbeing;
pub(crate) mod potential_id_prompt;
pub(crate) mod repellent_expulsion;
pub(crate) mod repellent_feedback;
pub(crate) mod truck_craft_prompt;
pub(crate) mod tutorial_gear_explanations;
pub(crate) mod tutorial_introductions;
pub(crate) mod tutorial_specific;

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

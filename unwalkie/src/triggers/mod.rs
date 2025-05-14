use bevy::app::App;

pub mod base1;
pub mod basic_gear_usage;
pub mod environmental_awareness;
pub mod evidence_gathering_logic;
pub mod ghost_behavior_hunting;
pub mod locomotion_interaction;
pub mod mission_progression_truck;
pub mod player_wellbeing;
pub mod repellent_expulsion;
pub mod tutorial_specific;

pub(crate) fn app_setup(app: &mut App) {
    base1::app_setup(app);
    basic_gear_usage::app_setup(app);
    environmental_awareness::app_setup(app);
    evidence_gathering_logic::app_setup(app);
    ghost_behavior_hunting::app_setup(app);
    locomotion_interaction::app_setup(app);
    mission_progression_truck::app_setup(app);
    player_wellbeing::app_setup(app);
    repellent_expulsion::app_setup(app);
    tutorial_specific::app_setup(app);
}

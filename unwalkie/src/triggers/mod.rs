use bevy::app::App;

pub mod base1;
pub mod basic_gear_usage;
pub mod consumables_and_defense;
pub mod environmental_awareness;
pub mod evidence_gathering_logic;
pub mod ghost_behavior_and_hunting;
pub mod locomotion_interaction;
pub mod mission_progression_and_truck;
pub mod player_wellbeing;
pub mod potential_id_prompt;
pub mod repellent_expulsion;
pub mod truck_craft_prompt;
pub mod tutorial_gear_explanations;
pub mod tutorial_introductions;
pub mod tutorial_specific;

pub(crate) fn app_setup(app: &mut App) {
    app.add_plugins((
        tutorial_introductions::TutorialIntroductionsTriggerPlugin,
        tutorial_gear_explanations::TutorialGearExplanationsTriggerPlugin,
    ));
    base1::app_setup(app);
    basic_gear_usage::app_setup(app);
    environmental_awareness::app_setup(app);
    evidence_gathering_logic::app_setup(app);
    ghost_behavior_and_hunting::app_setup(app);
    locomotion_interaction::app_setup(app);
    mission_progression_and_truck::app_setup(app);
    player_wellbeing::app_setup(app);
    repellent_expulsion::app_setup(app);
    tutorial_specific::app_setup(app);
    consumables_and_defense::app_setup(app);
    potential_id_prompt::app_setup(app);
    truck_craft_prompt::app_setup(app);
}

pub use truck_craft_prompt::InTruckCraftPromptTimer;

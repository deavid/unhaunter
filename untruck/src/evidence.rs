use super::uibutton::{TruckButtonState, TruckButtonType, TruckUIButton};
use bevy::prelude::*;
use uncore::components::game_ui::EvidenceUI;
use uncore::components::{game_config::GameConfig, player_sprite::PlayerSprite};
use uncore::resources::looking_gear::LookingGear;
use uncore::states::{AppState, GameState};
use uncore::types::evidence::Evidence;
use uncore::types::evidence_status::EvidenceStatus;
use ungear::components::playergear::PlayerGear;

pub fn update_evidence_ui(
    gc: Res<GameConfig>,
    q_gear: Query<(&PlayerSprite, &PlayerGear)>,
    mut qs: Query<Entity, With<EvidenceUI>>,
    interaction_query: Query<&TruckUIButton, With<Button>>,
    mut writer: TextUiWriter,
    looking_gear: Res<LookingGear>,
) {
    for (ps, playergear) in q_gear.iter() {
        if gc.player_id == ps.id {
            for txt_entity in qs.iter_mut() {
                let o_evidence =
                    Evidence::try_from(&playergear.get_hand(&looking_gear.hand()).kind).ok();
                let ev_state = match o_evidence {
                    Some(ev) => interaction_query
                        .iter()
                        .find(|t| t.class == TruckButtonType::Evidence(ev))
                        .map(|t| t.status)
                        .unwrap_or(TruckButtonState::Off),
                    None => TruckButtonState::Off,
                };
                let status = EvidenceStatus::from_gearkind(o_evidence, ev_state);
                if let Some((_entity, _depth, mut text, _font, _color)) = writer.get(txt_entity, 1)
                {
                    if *text != status.title {
                        *text = status.title;
                    }
                }
                if let Some((_entity, _depth, mut text, _font, mut color)) =
                    writer.get(txt_entity, 2)
                {
                    if *text != status.status_game {
                        *text = status.status_game;
                        *color = TextColor(status.status_color);
                    }
                }
                if let Some((_entity, _depth, mut text, _font, _color)) = writer.get(txt_entity, 3)
                {
                    if *text != status.help_text {
                        *text = status.help_text;
                    }
                }
            }
        }
    }
}

pub fn keyboard_evidence(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    gc: Res<GameConfig>,
    players: Query<(&PlayerSprite, &PlayerGear)>,
    mut interaction_query: Query<&mut TruckUIButton, With<Button>>,
    looking_gear: Res<LookingGear>,
) {
    for (player, playergear) in &players {
        if gc.player_id != player.id {
            continue;
        }
        let Ok(evidence) = Evidence::try_from(&playergear.get_hand(&looking_gear.hand()).kind)
        else {
            continue;
        };
        if keyboard_input.just_pressed(player.controls.change_evidence) {
            for mut t in &mut interaction_query {
                if t.class == TruckButtonType::Evidence(evidence) {
                    t.pressed();
                }
            }
        }
    }
}

pub fn app_setup(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        update_evidence_ui.run_if(in_state(GameState::None).and(in_state(AppState::InGame))),
    )
    .add_systems(
        Update,
        keyboard_evidence.run_if(in_state(GameState::None).and(in_state(AppState::InGame))),
    );
}

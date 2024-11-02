use bevy::prelude::*;

use crate::root;

use super::ManualPage;

#[derive(Component)]
pub struct PrePlayManualUI;


pub fn preplay_manual_system(
    mut _current_page: ResMut<ManualPage>,
    // difficulty: Res<CurrentDifficulty>,
    _keyboard_input: Res<ButtonInput<KeyCode>>,
    mut _interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    mut _next_state: ResMut<NextState<root::State>>,
    // mut game_next_state: ResMut<NextState<root::GameState>>,
    // Query for Text components
    _text_query: Query<&Text>,
    mut _button_query: Query<(&Children, &mut Visibility), With<Button>>,
) {
}
use bevy::prelude::*;
use uncore::colors;
use uncore::components::hint_ui::{HintBoxText, HintBoxUIRoot};
use uncore::events::hint::OnScreenHintEvent;
use uncore::resources::hint_ui_state::{HintAnimationPhase, HintUiState};
use uncore::states::AppState;
use uncore::types::root::game_assets::GameAssets; // For font

const HINT_TEXT_FONT_SIZE: f32 = 24.0;
const HINT_PADDING: Val = Val::Px(10.0);
const HINT_BORDER_RADIUS: Val = Val::Px(15.0);
const HINT_BACKGROUND_COLOR: Color = Color::rgba(0.9, 0.9, 0.9, 0.9);
const HINT_TEXT_COLOR: Color = Color::BLACK;
const HINT_OFFSCREEN_LEFT: Val = Val::Percent(-100.0); // Start off-screen to the left
const HINT_ONSCREEN_LEFT: Val = Val::Px(20.0); // Target on-screen position

/// System to set up the hint UI elements when entering the InGame state.
fn setup_hint_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // game_assets: Res<GameAssets>, // If using a preloaded font from GameAssets
) {
    commands
        .spawn((
            Node {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: HINT_OFFSCREEN_LEFT,
                    bottom: Val::Px(20.0),
                    width: Val::Auto,
                    height: Val::Auto,
                    padding: UiRect::all(HINT_PADDING),
                    border: UiRect::all(Val::Px(2.0)), // Optional: if you want a border around the bg
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: HINT_BACKGROUND_COLOR.into(),
                border_radius: BorderRadius::all(HINT_BORDER_RADIUS),
                ..default()
            },
            HintBoxUIRoot,
            Name::new("HintBoxUIRoot"),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text {
                    sections: vec![TextSection::new(
                        "", // Initial text is empty
                        TextStyle {
                            // font: game_assets.fonts.victormono.w400_regular.clone(), // Example using GameAssets
                            font: asset_server
                                .load("fonts/victor_mono/static/VictorMono-Regular.ttf"), // Direct load
                            font_size: HINT_TEXT_FONT_SIZE,
                            color: HINT_TEXT_COLOR,
                            ..default()
                        },
                    )],
                    justify: JustifyText::Center,
                    ..default()
                },
                HintBoxText,
                Name::new("HintBoxText"),
            ));
        });
}

/// System to handle OnScreenHintEvents and manage hint animation.
fn handle_hint_events_and_animation(
    mut hint_events: EventReader<OnScreenHintEvent>,
    mut hint_ui_state: ResMut<HintUiState>,
    mut q_hint_text: Query<&mut Text, With<HintBoxText>>,
    mut q_hint_root_style: Query<&mut Style, With<HintBoxUIRoot>>,
    time: Res<Time>,
) {
    // Check for new hint events first
    if let Some(event) = hint_events.read().last() {
        hint_ui_state.current_text = event.hint_text.clone();
        hint_ui_state.phase = HintAnimationPhase::AnimatingIn;
        hint_ui_state
            .animation_timer
            .set_duration(hint_ui_state.slide_in_duration);
        hint_ui_state.animation_timer.reset();

        if let Ok(mut text_component) = q_hint_text.get_single_mut() {
            if !text_component.sections.is_empty() {
                text_component.sections[0].value = hint_ui_state.current_text.clone();
            }
        }
    }

    // Animate based on current phase
    hint_ui_state.animation_timer.tick(time.delta());

    if let Ok(mut root_style) = q_hint_root_style.get_single_mut() {
        match hint_ui_state.phase {
            HintAnimationPhase::AnimatingIn => {
                let fraction = hint_ui_state.animation_timer.fraction();
                root_style.left = Val::Px(
                    HINT_OFFSCREEN_LEFT.evaluate(0.0).unwrap_or(-300.0) * (1.0 - fraction)
                        + HINT_ONSCREEN_LEFT.evaluate(0.0).unwrap_or(20.0) * fraction,
                );
                if hint_ui_state.animation_timer.finished() {
                    hint_ui_state.phase = HintAnimationPhase::Visible;
                    hint_ui_state
                        .animation_timer
                        .set_duration(hint_ui_state.visible_duration);
                    hint_ui_state.animation_timer.reset();
                    root_style.left = HINT_ONSCREEN_LEFT; // Ensure it's exactly in place
                }
            }
            HintAnimationPhase::Visible => {
                if hint_ui_state.animation_timer.finished() {
                    hint_ui_state.phase = HintAnimationPhase::AnimatingOut;
                    hint_ui_state
                        .animation_timer
                        .set_duration(hint_ui_state.slide_out_duration);
                    hint_ui_state.animation_timer.reset();
                }
            }
            HintAnimationPhase::AnimatingOut => {
                let fraction = hint_ui_state.animation_timer.fraction();
                root_style.left = Val::Px(
                    HINT_ONSCREEN_LEFT.evaluate(0.0).unwrap_or(20.0) * (1.0 - fraction)
                        + HINT_OFFSCREEN_LEFT.evaluate(0.0).unwrap_or(-300.0) * fraction,
                );
                if hint_ui_state.animation_timer.finished() {
                    hint_ui_state.phase = HintAnimationPhase::Idle;
                    root_style.left = HINT_OFFSCREEN_LEFT; // Ensure it's fully off-screen
                }
            }
            HintAnimationPhase::Idle => {
                // Do nothing, wait for a new event
            }
        }
    }
}

/// Plugin to add the hint UI systems to the game.
pub struct HintUiPlugin;

impl Plugin for HintUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup_hint_ui)
            .add_systems(
                Update,
                handle_hint_events_and_animation.run_if(in_state(AppState::InGame)),
            );
    }
}

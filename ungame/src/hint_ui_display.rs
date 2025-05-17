use bevy::prelude::*;
use uncore::{
    components::hint_ui::{HintBoxText, HintBoxUIRoot},
    events::hint::OnScreenHintEvent,
    platform::plt, // For FONT_SCALE
    resources::hint_ui_state::{HintAnimationPhase, HintUiState},
    states::AppState,
    types::root::game_assets::GameAssets, // For font handles
};

// --- Constants for Animation and Styling ---
// We will use the durations from HintUiState resource
const HINT_BOX_WIDTH_PX: f32 = 450.0;
const HINT_BOX_MARGIN_LEFT_PX: f32 = 20.0;
const HINT_BOX_OFFSCREEN_LEFT_PX: f32 = -(HINT_BOX_WIDTH_PX + HINT_BOX_MARGIN_LEFT_PX);
const HINT_BOX_ONSCREEN_LEFT_PX: f32 = HINT_BOX_MARGIN_LEFT_PX;
const HINT_BOX_BOTTOM_PX: f32 = 170.0;

// Styling constants
const HINT_BOX_BACKGROUND_COLOR: Color = Color::WHITE;
const HINT_BOX_TEXT_COLOR: Color = Color::BLACK;
const HINT_BOX_BORDER_RADIUS_VAL: f32 = 12.0;
const HINT_BOX_PADDING_PX: f32 = 15.0;
const HINT_BOX_PADDING_UIVAL: UiRect = UiRect::all(Val::Px(HINT_BOX_PADDING_PX));
const HINT_TEXT_FONT_SIZE_SCALE: f32 = 18.0 * plt::FONT_SCALE;

// --- Easing Functions ---
fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start * (1.0 - t) + end * t
}

fn ease_out_quad(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

fn ease_in_quad(t: f32) -> f32 {
    t * t
}

/// Sets up the on-screen hint UI elements.
pub fn setup_hint_ui_system(mut commands: Commands, handles: Res<GameAssets>) {
    let font_size = HINT_TEXT_FONT_SIZE_SCALE;
    let font_handle = handles.fonts.overlock.w400_regular.clone();

    commands
        .spawn((
            HintBoxUIRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(HINT_BOX_OFFSCREEN_LEFT_PX),
                bottom: Val::Px(HINT_BOX_BOTTOM_PX),
                width: Val::Px(HINT_BOX_WIDTH_PX),
                height: Val::Auto,
                padding: HINT_BOX_PADDING_UIVAL,
                ..default()
            },
            BackgroundColor(HINT_BOX_BACKGROUND_COLOR),
            BorderRadius::all(Val::Px(HINT_BOX_BORDER_RADIUS_VAL)),
            Visibility::Hidden,
            ZIndex(200),
        ))
        .with_children(|parent| {
            parent.spawn((
                HintBoxText,   // Marker for the text entity
                Text::new(""), // Initial empty text using Text::new()
                TextFont {
                    font: font_handle,
                    font_size,
                    ..default()
                },
                TextColor(HINT_BOX_TEXT_COLOR),
                // Node for text layout (e.g., max_width for wrapping)
                Node {
                    max_width: Val::Px(HINT_BOX_WIDTH_PX - (2.0 * HINT_BOX_PADDING_PX)),
                    ..default()
                },
            ));
        });
}

/// Handles `OnScreenHintEvent`s and manages hint animations.
pub fn hint_ui_event_and_animation_system(
    mut events: EventReader<OnScreenHintEvent>,
    mut ui_state: ResMut<HintUiState>,
    mut hint_box_query: Query<(&mut Node, &mut Visibility), With<HintBoxUIRoot>>,
    text_entity_query: Query<Entity, With<HintBoxText>>, // Query for the Entity with HintBoxText
    mut text_query: Query<&mut Text, With<HintBoxText>>, // For direct text update
    time: Res<Time>,
) {
    let Ok((mut hint_node, mut visibility)) = hint_box_query.get_single_mut() else {
        if ui_state.phase != HintAnimationPhase::Idle {
            *ui_state = HintUiState::default();
        }
        return;
    };

    // Get the text entity
    let text_entity = text_entity_query.get_single().ok();

    if let Some(event) = events.read().last() {
        // Store durations locally before modifying ui_state
        let slide_in_duration = ui_state.slide_in_duration;

        ui_state.current_text = event.hint_text.clone();
        ui_state.phase = HintAnimationPhase::AnimatingIn;
        ui_state.animation_timer.set_duration(slide_in_duration);
        ui_state.animation_timer.reset();

        // Update text content - using direct query instead of TextUiWriter
        if let Some(entity) = text_entity {
            if let Ok(mut text) = text_query.get_mut(entity) {
                text.0 = ui_state.current_text.clone();
            }
        }
        events.clear(); // Clear events after processing the last one
    }

    if ui_state.phase == HintAnimationPhase::Idle {
        if *visibility != Visibility::Hidden {
            *visibility = Visibility::Hidden;
        }
        return;
    }

    if *visibility == Visibility::Hidden {
        *visibility = Visibility::Visible;
    }

    ui_state.animation_timer.tick(time.delta());
    let progress = ui_state.animation_timer.fraction();

    match ui_state.phase {
        HintAnimationPhase::AnimatingIn => {
            hint_node.left = Val::Px(lerp(
                HINT_BOX_OFFSCREEN_LEFT_PX,
                HINT_BOX_ONSCREEN_LEFT_PX,
                ease_out_quad(progress),
            ));
            if ui_state.animation_timer.finished() {
                // Store durations locally before modifying ui_state
                let visible_duration = ui_state.visible_duration;

                ui_state.phase = HintAnimationPhase::Visible;
                ui_state.animation_timer.set_duration(visible_duration);
                ui_state.animation_timer.reset();
                hint_node.left = Val::Px(HINT_BOX_ONSCREEN_LEFT_PX); // Ensure exact position
            }
        }
        HintAnimationPhase::Visible => {
            if ui_state.animation_timer.finished() {
                // Store durations locally before modifying ui_state
                let slide_out_duration = ui_state.slide_out_duration;

                ui_state.phase = HintAnimationPhase::AnimatingOut;
                ui_state.animation_timer.set_duration(slide_out_duration);
                ui_state.animation_timer.reset();
            }
        }
        HintAnimationPhase::AnimatingOut => {
            hint_node.left = Val::Px(lerp(
                HINT_BOX_ONSCREEN_LEFT_PX,
                HINT_BOX_OFFSCREEN_LEFT_PX,
                ease_in_quad(progress),
            ));
            if ui_state.animation_timer.finished() {
                ui_state.phase = HintAnimationPhase::Idle;
                *visibility = Visibility::Hidden;
                hint_node.left = Val::Px(HINT_BOX_OFFSCREEN_LEFT_PX); // Ensure exact position
                if let Some(entity) = text_entity {
                    if let Ok(mut text) = text_query.get_mut(entity) {
                        text.0 = "".to_string(); // Clear text when hiding
                    }
                }
            }
        }
        HintAnimationPhase::Idle => { /* Already handled by the early return */ }
    }
}

/// Cleans up the on-screen hint UI elements when leaving the in-game state.
pub fn cleanup_hint_ui_system(mut commands: Commands, query: Query<Entity, With<HintBoxUIRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// Registers the hint UI systems with the app.
pub fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(AppState::InGame), setup_hint_ui_system)
        .add_systems(
            Update,
            hint_ui_event_and_animation_system.run_if(in_state(AppState::InGame)),
        )
        .add_systems(OnExit(AppState::InGame), cleanup_hint_ui_system);
}

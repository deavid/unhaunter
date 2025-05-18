use bevy::{
    prelude::*,
    text::TextLayoutInfo, // Added for text layout if needed, TextAlignment
};
use std::time::Duration;
use uncore::{
    components::hint_ui::{HintBoxText, HintBoxUIRoot},
    events::hint::OnScreenHintEvent,
    platform::plt,
    resources::hint_ui_state::{HintAnimationPhase, HintUIState},
    states::AppState, // Added for OnEnter/OnExit AppState::InGame
};

// --- Constants as per the plan ---
const ANIMATION_DURATION_SECS: f32 = 0.3;
const VISIBLE_DURATION_SECS: f32 = 7.0;
const HINT_BOX_WIDTH_PX: f32 = 350.0;
const HINT_BOX_MARGIN_LEFT_PX: f32 = 20.0; // Renamed from HINT_BOX_ONSCREEN_LEFT_PX for clarity in offscreen calculation
const HINT_BOX_OFFSCREEN_LEFT_PX: f32 = -(HINT_BOX_WIDTH_PX + HINT_BOX_MARGIN_LEFT_PX); // Initial off-screen X
const HINT_BOX_ONSCREEN_LEFT_PX: f32 = HINT_BOX_MARGIN_LEFT_PX; // Target on-screen X
const HINT_BOX_BOTTOM_PX: f32 = 70.0; // Y position from bottom

// Styling constants
const HINT_BOX_BACKGROUND_COLOR: Color = Color::WHITE;
const HINT_BOX_TEXT_COLOR: Color = Color::BLACK;
const HINT_BOX_BORDER_RADIUS_VAL: f32 = 10.0; // Changed to f32 for BorderRadius::all
const HINT_BOX_PADDING_PX: f32 = 15.0;
const HINT_BOX_PADDING_UIVAL: UiRect = UiRect::all(Val::Px(HINT_BOX_PADDING_PX));
// Using Overlock as suggested, ensure this path is correct relative to asset root
const HINT_TEXT_FONT_PATH: &str = "fonts/overlock/Overlock-Regular.ttf";

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
/// Renamed from setup_hint_ui to match plan.
pub fn setup_hint_ui_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_handle: Handle<Font> = asset_server.load(HINT_TEXT_FONT_PATH);
    let text_font_size = 18.0 * plt::FONT_SCALE;

    commands
        .spawn((
            HintBoxUIRoot,
            NodeBundle {
                // NodeBundle is still used as a base for UI nodes
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(HINT_BOX_OFFSCREEN_LEFT_PX),
                    bottom: Val::Px(HINT_BOX_BOTTOM_PX),
                    width: Val::Px(HINT_BOX_WIDTH_PX),
                    min_height: Val::Px(50.0), // Set a min height, auto will also work
                    height: Val::Auto,
                    padding: HINT_BOX_PADDING_UIVAL,
                    align_items: AlignItems::Center, // Vertically center content
                    justify_content: JustifyContent::FlexStart, // Horizontally align to start
                    flex_wrap: FlexWrap::Wrap,       // Allow text to wrap
                    ..default()
                },
                background_color: BackgroundColor(HINT_BOX_BACKGROUND_COLOR),
                visibility: Visibility::Hidden, // Start hidden
                z_index: ZIndex::Local(100),    // Ensure it's on top
                ..default()
            },
            BorderRadius::all(Val::Px(HINT_BOX_BORDER_RADIUS_VAL)), // BorderRadius is a component
        ))
        .with_children(|parent| {
            parent.spawn((
                HintBoxText,
                TextBundle::from_sections([TextSection::new(
                    "", // Initial empty text
                    TextStyle {
                        font: font_handle,
                        font_size: text_font_size,
                        color: HINT_BOX_TEXT_COLOR,
                    },
                )])
                // Align text within its own bounds if TextBundle takes full width of parent padding box
                .with_text_justify(JustifyText::Left) // Use JustifyText for Bevy 0.12+
                .with_style(Style {
                    // Max width for text wrapping, considering parent's padding
                    max_width: Val::Px(HINT_BOX_WIDTH_PX - (2.0 * HINT_BOX_PADDING_PX)),
                    ..default()
                }),
                // Required for text wrapping calculations if not automatically handled by Flexbox
                // TextLayoutInfo, // Usually added automatically by TextBundle
            ));
        });
}

/// Handles `OnScreenHintEvent`s and manages hint animations.
/// Renamed from handle_hint_events_and_animate to match plan.
#[allow(clippy::too_many_arguments)]
pub fn hint_ui_event_and_animation_system(
    mut events: EventReader<OnScreenHintEvent>,
    mut ui_state: ResMut<HintUIState>,
    // Query for Style and Visibility separately as they are components
    mut hint_box_query: Query<(&mut Style, &mut Visibility), With<HintBoxUIRoot>>,
    mut text_query: Query<&mut Text, With<HintBoxText>>,
    time: Res<Time>,
) {
    // It's safer to use get_single_mut for entities expected to be unique.
    if let Ok((mut hint_style, mut visibility)) = hint_box_query.get_single_mut() {
        if let Ok(mut text_component) = text_query.get_single_mut() {
            // Event Handling
            if let Some(event) = events.read().last() {
                ui_state.text = event.hint_text.clone();
                ui_state.animation_phase = HintAnimationPhase::AnimatingIn;
                ui_state
                    .timer
                    .set_duration(Duration::from_secs_f32(ANIMATION_DURATION_SECS));
                ui_state.timer.reset();

                // Update text content
                if text_component.sections.is_empty() {
                    // This should not happen if setup is correct and text_component always has one section
                    text_component.sections.push(TextSection::new(
                        ui_state.text.clone(),
                        // Infer style from existing or default if truly empty, but setup should prevent this
                        TextStyle {
                            font_size: 18.0 * plt::FONT_SCALE,
                            color: HINT_BOX_TEXT_COLOR,
                            font: Default::default(), /* Needs actual font handle */
                        },
                    ));
                } else {
                    text_component.sections[0].value = ui_state.text.clone();
                }
                // events.clear(); // Not strictly necessary with .last()
            }

            // Animation Logic
            if ui_state.animation_phase == HintAnimationPhase::Idle {
                // If visibility is hidden and we are idle, ensure position is offscreen.
                // This handles cases where the game might have started with a hint already processed to Idle.
                if *visibility == Visibility::Hidden {
                    hint_style.left = Val::Px(HINT_BOX_OFFSCREEN_LEFT_PX);
                }
                return;
            }

            // Ensure visibility is on if we are animating
            if *visibility == Visibility::Hidden
                && ui_state.animation_phase != HintAnimationPhase::Idle
            {
                *visibility = Visibility::Visible;
            }

            ui_state.timer.tick(time.delta());
            let progress = ui_state.timer.fraction_remaining(); // For animating out, often fraction_remaining is useful, or 1.0 - fraction for animating in.
            // Let's stick to `fraction()` and adjust lerp if needed.
            let mut current_progress = ui_state.timer.fraction();

            match ui_state.animation_phase {
                HintAnimationPhase::AnimatingIn => {
                    let eased_progress = ease_out_quad(current_progress);
                    hint_style.left = Val::Px(lerp(
                        HINT_BOX_OFFSCREEN_LEFT_PX,
                        HINT_BOX_ONSCREEN_LEFT_PX,
                        eased_progress,
                    ));
                    if ui_state.timer.finished() {
                        ui_state.animation_phase = HintAnimationPhase::Visible;
                        ui_state
                            .timer
                            .set_duration(Duration::from_secs_f32(VISIBLE_DURATION_SECS));
                        ui_state.timer.reset();
                        hint_style.left = Val::Px(HINT_BOX_ONSCREEN_LEFT_PX); // Ensure final position
                    }
                }
                HintAnimationPhase::Visible => {
                    if ui_state.timer.finished() {
                        ui_state.animation_phase = HintAnimationPhase::AnimatingOut;
                        ui_state
                            .timer
                            .set_duration(Duration::from_secs_f32(ANIMATION_DURATION_SECS));
                        ui_state.timer.reset();
                    }
                }
                HintAnimationPhase::AnimatingOut => {
                    // For animating out, progress goes from 0 to 1.
                    // If using ease_in_quad, it correctly accelerates the movement towards offscreen.
                    let eased_progress = ease_in_quad(current_progress);
                    hint_style.left = Val::Px(lerp(
                        HINT_BOX_ONSCREEN_LEFT_PX,
                        HINT_BOX_OFFSCREEN_LEFT_PX,
                        eased_progress,
                    ));
                    if ui_state.timer.finished() {
                        ui_state.animation_phase = HintAnimationPhase::Idle;
                        *visibility = Visibility::Hidden;
                        if !text_component.sections.is_empty() {
                            text_component.sections[0].value.clear(); // Clear text
                        }
                        hint_style.left = Val::Px(HINT_BOX_OFFSCREEN_LEFT_PX); // Ensure final position
                    }
                }
                HintAnimationPhase::Idle => { /* Already handled by early return */ }
            }
        } else {
            // Error: HintBoxText component not found. This might indicate a despawn or setup issue.
            // Optionally, reset ui_state to prevent inconsistent states.
            // warn!("HintBoxText not found, resetting HintUIState.");
            // *ui_state = HintUIState::default();
        }
    } else {
        // Error: HintBoxUIRoot (Style or Visibility) not found.
        // warn!("HintBoxUIRoot not found, resetting HintUIState.");
        // *ui_state = HintUIState::default();
    }
}

/// Cleans up the on-screen hint UI elements.
pub fn cleanup_hint_ui_system(mut commands: Commands, query: Query<Entity, With<HintBoxUIRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    // As per plan: Optional: commands.remove_resource::<HintUIState>()
    // The plan states HintUIState is init_resource'd in UnhaunterCorePlugin,
    // so it should persist unless explicitly removed or the app state managing it is exited.
    // If it's meant to be truly per-InGame session and re-initialized, then removal is fine.
    // The current ungame/plugin.rs also does init_resource<HintUIState>, which might be redundant
    // if uncore/plugin.rs does it. It's safer to have it initialized once in the core plugin.
}

use bevy::prelude::*;

use uncore::{
    components::hint_ui::{HintBoxText, HintBoxUIRoot},
    events::hint::OnScreenHintEvent,
    platform::plt,
    resources::hint_ui_state::{HintAnimationPhase, HintUiState},
    states::AppState,
};

const HINT_BOX_WIDTH_PX: f32 = 350.0;
const HINT_BOX_MARGIN_LEFT_PX: f32 = 20.0;
const HINT_BOX_OFFSCREEN_LEFT_PX: f32 = -(HINT_BOX_WIDTH_PX + HINT_BOX_MARGIN_LEFT_PX);
const HINT_BOX_ONSCREEN_LEFT_PX: f32 = HINT_BOX_MARGIN_LEFT_PX;
const HINT_BOX_BOTTOM_PX: f32 = 170.0;

const HINT_BOX_BACKGROUND_COLOR: Color = Color::WHITE;
const HINT_BOX_TEXT_COLOR: Color = Color::BLACK;
const HINT_BOX_BORDER_RADIUS_VAL: f32 = 10.0;
const HINT_BOX_PADDING_PX: f32 = 15.0;
const HINT_BOX_PADDING_UIVAL: UiRect = UiRect::all(Val::Px(HINT_BOX_PADDING_PX));
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

pub(crate) fn app_setup(app: &mut App) {
    app.init_resource::<HintUiState>()
        .add_systems(OnEnter(AppState::InGame), setup_hint_ui_system)
        .add_systems(
            Update,
            hint_ui_event_and_animation_system.run_if(in_state(AppState::InGame)),
        )
        .add_systems(OnExit(AppState::InGame), cleanup_hint_ui_system);
}

/// Sets up the on-screen hint UI elements.
/// Renamed from setup_hint_ui to match plan.
fn setup_hint_ui_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_handle: Handle<Font> = asset_server.load(HINT_TEXT_FONT_PATH);
    let text_font_size = 18.0 * plt::FONT_SCALE;

    commands
        .spawn((
            HintBoxUIRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(HINT_BOX_OFFSCREEN_LEFT_PX),
                bottom: Val::Px(HINT_BOX_BOTTOM_PX),
                width: Val::Px(HINT_BOX_WIDTH_PX),
                min_height: Val::Px(50.0),
                padding: HINT_BOX_PADDING_UIVAL,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                flex_wrap: FlexWrap::Wrap,
                ..default()
            },
            BackgroundColor(HINT_BOX_BACKGROUND_COLOR),
            BorderRadius::all(Val::Px(HINT_BOX_BORDER_RADIUS_VAL)),
            Visibility::Hidden,
            ZIndex(100),
        ))
        .with_children(|parent| {
            parent.spawn((
                HintBoxText,
                Text("".to_string()),
                TextFont {
                    font: font_handle.clone(),
                    font_size: text_font_size,
                    ..default()
                },
                TextColor(HINT_BOX_TEXT_COLOR),
                Node {
                    max_width: Val::Px(HINT_BOX_WIDTH_PX - (2.0 * HINT_BOX_PADDING_PX)),
                    ..default()
                },
            ));
        });
}

/// Handles `OnScreenHintEvent`s and manages hint animations.
fn hint_ui_event_and_animation_system(
    mut events: EventReader<OnScreenHintEvent>,
    mut ui_state: ResMut<HintUiState>,
    mut hint_box_query: Query<(&mut Node, &mut Visibility), With<HintBoxUIRoot>>,
    mut text_query: Query<&mut Text, With<HintBoxText>>,
    time: Res<Time>,
) {
    if let Ok((mut hint_style, mut visibility)) = hint_box_query.single_mut() {
        if let Ok(mut text_component) = text_query.single_mut() {
            // Event Handling
            if let Some(event) = events.read().last() {
                let slide_in_duration = ui_state.slide_in_duration;
                ui_state.current_text = event.hint_text.clone();
                ui_state.phase = HintAnimationPhase::AnimatingIn;
                ui_state.animation_timer.set_duration(slide_in_duration);
                ui_state.animation_timer.reset();

                text_component.0 = ui_state.current_text.clone();
            }

            // Animation Logic
            if ui_state.phase == HintAnimationPhase::Idle {
                if *visibility == Visibility::Hidden {
                    hint_style.left = Val::Px(HINT_BOX_OFFSCREEN_LEFT_PX);
                }
                return;
            }

            if *visibility == Visibility::Hidden && ui_state.phase != HintAnimationPhase::Idle {
                *visibility = Visibility::Visible;
            }

            ui_state.animation_timer.tick(time.delta());
            let current_progress = ui_state.animation_timer.fraction();

            match ui_state.phase {
                HintAnimationPhase::AnimatingIn => {
                    let eased_progress = ease_out_quad(current_progress);
                    hint_style.left = Val::Px(lerp(
                        HINT_BOX_OFFSCREEN_LEFT_PX,
                        HINT_BOX_ONSCREEN_LEFT_PX,
                        eased_progress,
                    ));
                    if ui_state.animation_timer.finished() {
                        hint_style.left = Val::Px(HINT_BOX_ONSCREEN_LEFT_PX);
                        let visible_duration = ui_state.visible_duration;
                        ui_state.phase = HintAnimationPhase::Visible;
                        ui_state.animation_timer.set_duration(visible_duration);
                        ui_state.animation_timer.reset();
                    }
                }
                HintAnimationPhase::Visible => {
                    if ui_state.animation_timer.finished() {
                        let slide_out_duration = ui_state.slide_out_duration;
                        ui_state.phase = HintAnimationPhase::AnimatingOut;
                        ui_state.animation_timer.set_duration(slide_out_duration);
                        ui_state.animation_timer.reset();
                    }
                }
                HintAnimationPhase::AnimatingOut => {
                    let eased_progress = ease_in_quad(current_progress);
                    hint_style.left = Val::Px(lerp(
                        HINT_BOX_ONSCREEN_LEFT_PX,
                        HINT_BOX_OFFSCREEN_LEFT_PX,
                        eased_progress,
                    ));
                    if ui_state.animation_timer.finished() {
                        hint_style.left = Val::Px(HINT_BOX_OFFSCREEN_LEFT_PX);
                        *visibility = Visibility::Hidden;
                        ui_state.phase = HintAnimationPhase::Idle;
                        text_component.0.clear();
                    }
                }
                HintAnimationPhase::Idle => { /* Already handled */ }
            }
        } else {
            // warn!("HintBoxText not found");
        }
    } else {
        // warn!("HintBoxUIRoot not found");
    }
}

/// Cleans up the on-screen hint UI elements.
fn cleanup_hint_ui_system(mut commands: Commands, query: Query<Entity, With<HintBoxUIRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

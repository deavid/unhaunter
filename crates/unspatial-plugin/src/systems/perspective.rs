use bevy::diagnostic::{Diagnostic, DiagnosticPath as DP, RegisterDiagnostic};
use bevy::prelude::*;
use unmetrics_core::metrics::SendMetric;
use unrender_std::components::sprite_layer::SpriteLayer;
use unspatial_core::lerp_position::LerpPosition;
use unspatial_core::perspective;
use unspatial_core::position::Position;

const APPLY_PERSPECTIVE: DP = DP::const_new("unboard/systems/apply_perspective");

/// Converts logical `Position` (and optional `LerpPosition`) to a Bevy screen-space
/// `Transform`, applying the isometric projection and any `SpriteLayer` Z-offset.
///
/// Runs in `PostUpdate` so all game-logic Position mutations from `Update` are visible
/// before the Bevy transform propagation pass.
fn apply_perspective(
    mut q: Query<
        (
            &Position,
            Option<&LerpPosition>,
            &mut Transform,
            Option<&SpriteLayer>,
        ),
        Or<(
            Changed<Position>,
            Changed<LerpPosition>,
            Changed<SpriteLayer>,
        )>,
    >,
) {
    let measure = APPLY_PERSPECTIVE.time_measure();

    for (pos, lerp, mut transform, layer) in q.iter_mut() {
        let effective_pos = if let Some(l) = lerp { &l.current } else { pos };
        let mut translation = perspective::to_screen_coord(*effective_pos);
        if let Some(layer) = layer {
            translation.z += layer.0;
        }
        transform.translation = translation;
    }

    measure.end_ms();
}

pub(crate) fn app_setup(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(APPLY_PERSPECTIVE).with_suffix("ms"));
    app.add_systems(PostUpdate, apply_perspective);
}

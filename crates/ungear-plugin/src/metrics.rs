use bevy::{
    app::App,
    diagnostic::{Diagnostic, DiagnosticPath as DP, RegisterDiagnostic},
};

pub(crate) const UPDATE_DEPLOYED_GEAR_SPRITES: DP =
    DP::const_new("ungear/systems/update_deployed_gear_sprites");
pub(crate) const SOUND_PLAYBACK: DP = DP::const_new("ungear/systems/sound_playback");
pub(crate) const UPDATE_GEAR_UI: DP = DP::const_new("ungear/systems/update_gear_ui");

pub(crate) fn register_all(app: &mut App) {
    app.register_diagnostic(Diagnostic::new(UPDATE_DEPLOYED_GEAR_SPRITES).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(SOUND_PLAYBACK).with_suffix("ms"))
        .register_diagnostic(Diagnostic::new(UPDATE_GEAR_UI).with_suffix("ms"));
}
